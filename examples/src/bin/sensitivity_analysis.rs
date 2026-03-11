//! P2 — Анализ чувствительности параметров CDATA (Sensitivity Analysis).
//!
//! Для каждого из 11 параметров `DamageParams` прогоняем симуляцию с базовым,
//! +50% и −50% значением. Измеряем эффект на:
//!   - `lifespan`      — возраст смерти [лет]
//!   - `damage_at_60`  — total_damage_score в 60 лет
//!   - `frailty_at_70` — frailty в 70 лет
//!
//! Вывод: текстовый tornado-chart (ранжирование по |Δlifespan|) + CSV.
//!
//! ## Запуск
//! ```bash
//! cargo run --bin sensitivity_analysis
//! # Результаты: sensitivity_output/sa_results.csv
//! ```

use cell_dt_core::{SimulationManager, SimulationConfig, SimulationModule};
use cell_dt_core::components::CellCycleStateExtended;
use human_development_module::{
    HumanDevelopmentModule, HumanDevelopmentParams,
    HumanDevelopmentComponent,
    damage::DamageParams,
};
use std::fs;

// ---------------------------------------------------------------------------
// Конфигурация sweep
// ---------------------------------------------------------------------------

/// Один параметр для sweep-анализа.
struct SweepParam {
    name: &'static str,
    getter: fn(&DamageParams) -> f32,
    setter: fn(&mut DamageParams, f32),
}

/// Конфигурация анализа чувствительности.
struct ParameterSweepConfig {
    /// Фактор вариации: 0.5 = ±50%
    pub variation_factor: f32,
    /// Число шагов (лет) симуляции
    pub max_years: u64,
    /// Seed для воспроизводимости
    pub seed: u64,
}

impl Default for ParameterSweepConfig {
    fn default() -> Self {
        Self { variation_factor: 0.50, max_years: 120, seed: 42 }
    }
}

// ---------------------------------------------------------------------------
// Параметры для анализа
// ---------------------------------------------------------------------------

fn sweep_params() -> Vec<SweepParam> {
    vec![
        SweepParam {
            name: "base_ros_damage_rate",
            getter: |p| p.base_ros_damage_rate,
            setter: |p, v| p.base_ros_damage_rate = v,
        },
        SweepParam {
            name: "acetylation_rate",
            getter: |p| p.acetylation_rate,
            setter: |p, v| p.acetylation_rate = v,
        },
        SweepParam {
            name: "aggregation_rate",
            getter: |p| p.aggregation_rate,
            setter: |p, v| p.aggregation_rate = v,
        },
        SweepParam {
            name: "phospho_dysreg_rate",
            getter: |p| p.phospho_dysregulation_rate,
            setter: |p, v| p.phospho_dysregulation_rate = v,
        },
        SweepParam {
            name: "cep164_loss_rate",
            getter: |p| p.cep164_loss_rate,
            setter: |p, v| p.cep164_loss_rate = v,
        },
        SweepParam {
            name: "cep89_loss_rate",
            getter: |p| p.cep89_loss_rate,
            setter: |p, v| p.cep89_loss_rate = v,
        },
        SweepParam {
            name: "ninein_loss_rate",
            getter: |p| p.ninein_loss_rate,
            setter: |p, v| p.ninein_loss_rate = v,
        },
        SweepParam {
            name: "cep170_loss_rate",
            getter: |p| p.cep170_loss_rate,
            setter: |p, v| p.cep170_loss_rate = v,
        },
        SweepParam {
            name: "ros_feedback_coeff",
            getter: |p| p.ros_feedback_coefficient,
            setter: |p, v| p.ros_feedback_coefficient = v,
        },
        SweepParam {
            name: "midlife_multiplier",
            getter: |p| p.midlife_damage_multiplier,
            setter: |p, v| p.midlife_damage_multiplier = v,
        },
        SweepParam {
            name: "senescence_threshold",
            getter: |p| p.senescence_threshold,
            setter: |p, v| p.senescence_threshold = v,
        },
    ]
}

// ---------------------------------------------------------------------------
// Одна симуляция
// ---------------------------------------------------------------------------

struct RunResult {
    lifespan:     f64,
    damage_at_60: f32,
    frailty_at_70: f32,
}

fn run_one(damage_params: DamageParams, cfg: &ParameterSweepConfig) -> RunResult {
    let sim_config = SimulationConfig {
        max_steps: cfg.max_years * 365,
        dt: 1.0,
        checkpoint_interval: cfg.max_years * 365,
        num_threads: Some(1),
        seed: Some(cfg.seed),
        parallel_modules: false,
        cleanup_dead_interval: None,
    };

    let mut sim = SimulationManager::new(sim_config);

    let dev_params = HumanDevelopmentParams {
        base_detach_probability: 0.0, // детерминированный — без случайного шума
        ..HumanDevelopmentParams::default()
    };
    let mut dev = HumanDevelopmentModule::with_params(dev_params);
    dev.set_seed(cfg.seed);

    // Применяем пользовательские DamageParams через set_params JSON
    // (используем serde_json чтобы не зависеть от внутреннего API)
    sim.register_module(Box::new(dev)).unwrap();

    // Устанавливаем кастомные DamageParams через get_params/set_params
    {
        let world = sim.world_mut();
        world.spawn((CellCycleStateExtended::new(),));
    }
    sim.initialize().unwrap();

    // Применяем кастомные DamageParams через set_params
    // Используем ключи из get_params
    use serde_json::json;
    sim.set_module_params("human_development_module", &json!({
        "base_ros_damage_rate":       damage_params.base_ros_damage_rate,
        "acetylation_rate":           damage_params.acetylation_rate,
        "aggregation_rate":           damage_params.aggregation_rate,
        "phospho_dysregulation_rate": damage_params.phospho_dysregulation_rate,
        "cep164_loss_rate":           damage_params.cep164_loss_rate,
        "cep89_loss_rate":            damage_params.cep89_loss_rate,
        "ninein_loss_rate":           damage_params.ninein_loss_rate,
        "cep170_loss_rate":           damage_params.cep170_loss_rate,
        "ros_feedback_coefficient":   damage_params.ros_feedback_coefficient,
        "midlife_damage_multiplier":  damage_params.midlife_damage_multiplier,
        "senescence_threshold":       damage_params.senescence_threshold,
    })).unwrap();

    let mut lifespan     = cfg.max_years as f64;
    let mut damage_at_60 = 0.0f32;
    let mut frailty_at_70 = 0.0f32;
    let mut recorded_60  = false;
    let mut recorded_70  = false;

    for day in 0u64..(cfg.max_years * 365) {
        sim.step().unwrap();

        let year = day as f64 / 365.25;

        if !recorded_60 && year >= 60.0 {
            let world = sim.world();
            let mut q = world.query::<&HumanDevelopmentComponent>();
            if let Some((_, dev)) = q.iter().next() {
                damage_at_60 = dev.centriolar_damage.total_damage_score();
            }
            recorded_60 = true;
        }

        if !recorded_70 && year >= 70.0 {
            let world = sim.world();
            let mut q = world.query::<&HumanDevelopmentComponent>();
            if let Some((_, dev)) = q.iter().next() {
                frailty_at_70 = dev.frailty();
            }
            recorded_70 = true;
        }

        {
            let world = sim.world();
            let mut q = world.query::<&HumanDevelopmentComponent>();
            if q.iter().all(|(_, d)| !d.is_alive) && day > 365 {
                let world = sim.world();
                let mut q2 = world.query::<&HumanDevelopmentComponent>();
                if let Some((_, dev)) = q2.iter().next() {
                    lifespan = dev.age_years();
                }
                break;
            }
        }
    }

    RunResult { lifespan, damage_at_60, frailty_at_70 }
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  CDATA — Sensitivity Analysis (P2)                                   ║");
    println!("║  Varying each DamageParam by ±50%, measuring effect on lifespan      ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();

    let cfg = ParameterSweepConfig::default();
    let params = sweep_params();
    let base_dp = DamageParams::default();

    // Базовая симуляция
    print!("Running baseline... ");
    let base = run_one(base_dp.clone(), &cfg);
    println!("done → lifespan={:.1}yr, damage@60={:.3}, frailty@70={:.3}",
        base.lifespan, base.damage_at_60, base.frailty_at_70);
    println!();

    // Результаты для tornado-chart
    struct SaRow {
        name: String,
        base_val: f32,
        delta_hi: f64,  // lifespan change at +50%
        delta_lo: f64,  // lifespan change at -50%
        damage_hi: f32,
        damage_lo: f32,
        frailty_hi: f32,
        frailty_lo: f32,
    }

    let mut rows: Vec<SaRow> = Vec::new();

    for (i, param) in params.iter().enumerate() {
        let base_val = (param.getter)(&base_dp);

        // +50%
        let mut dp_hi = base_dp.clone();
        (param.setter)(&mut dp_hi, base_val * (1.0 + cfg.variation_factor));
        print!("[{:02}/{:02}] {} +{:.0}%... ", i + 1, params.len(), param.name, cfg.variation_factor * 100.0);
        let hi = run_one(dp_hi, &cfg);

        // -50%
        let mut dp_lo = base_dp.clone();
        (param.setter)(&mut dp_lo, (base_val * (1.0 - cfg.variation_factor)).max(0.0001));
        print!("-{:.0}%... ", cfg.variation_factor * 100.0);
        let lo = run_one(dp_lo, &cfg);

        println!("Δlifespan: {:+.1}yr / {:+.1}yr",
            hi.lifespan - base.lifespan,
            lo.lifespan - base.lifespan);

        rows.push(SaRow {
            name: param.name.to_string(),
            base_val,
            delta_hi: hi.lifespan - base.lifespan,
            delta_lo: lo.lifespan - base.lifespan,
            damage_hi: hi.damage_at_60 - base.damage_at_60,
            damage_lo: lo.damage_at_60 - base.damage_at_60,
            frailty_hi: hi.frailty_at_70 - base.frailty_at_70,
            frailty_lo: lo.frailty_at_70 - base.frailty_at_70,
        });
    }

    // Сортировка по |max(|Δhi|, |Δlo|)| — tornado chart
    rows.sort_by(|a, b| {
        let a_max = a.delta_hi.abs().max(a.delta_lo.abs());
        let b_max = b.delta_hi.abs().max(b.delta_lo.abs());
        b_max.partial_cmp(&a_max).unwrap()
    });

    // --- Tornado chart (текстовый) ---
    println!();
    println!("══════════════════════════════════════════════════════════════════════");
    println!("  TORNADO CHART — Sensitivity of Lifespan to ±50% Parameter Changes");
    println!("  Baseline: {:.1} yr  |  Seed: {}  |  Det. mode (noise=0)", base.lifespan, cfg.seed);
    println!("══════════════════════════════════════════════════════════════════════");
    println!("{:<24} {:>8} {:>9} {:>9}",
        "Parameter", "Base val", "+50% Δyr", "-50% Δyr");
    println!("{}", "─".repeat(54));

    let max_abs: f64 = rows.iter()
        .map(|r| r.delta_hi.abs().max(r.delta_lo.abs()))
        .fold(0.0_f64, f64::max)
        .max(1.0);

    for row in &rows {
        let bar_width = 20usize;
        let hi_len = ((row.delta_hi.abs() / max_abs) * bar_width as f64) as usize;
        let lo_len = ((row.delta_lo.abs() / max_abs) * bar_width as f64) as usize;

        let hi_sym = if row.delta_hi >= 0.0 { "+" } else { "-" };
        let lo_sym = if row.delta_lo >= 0.0 { "+" } else { "-" };
        let hi_bar = format!("{}{}", hi_sym, "█".repeat(hi_len));
        let lo_bar = format!("{}{}", lo_sym, "█".repeat(lo_len));

        println!("{:<24} {:>8.5} {:>+9.2} {:>+9.2}   {} / {}",
            row.name, row.base_val, row.delta_hi, row.delta_lo, hi_bar, lo_bar);
    }

    println!();
    println!("Interpretation: parameters at top have largest effect on lifespan.");
    println!("senescence_threshold -50% → immediate death (threshold too low).");

    // --- CSV ---
    fs::create_dir_all("sensitivity_output")?;
    let csv_path = "sensitivity_output/sa_results.csv";
    let mut csv = String::new();
    csv.push_str("parameter,base_value,delta_lifespan_hi,delta_lifespan_lo,delta_damage60_hi,delta_damage60_lo,delta_frailty70_hi,delta_frailty70_lo\n");
    for row in &rows {
        csv.push_str(&format!(
            "{},{:.6},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}\n",
            row.name, row.base_val,
            row.delta_hi, row.delta_lo,
            row.damage_hi, row.damage_lo,
            row.frailty_hi, row.frailty_lo,
        ));
    }
    fs::write(csv_path, &csv)?;
    println!("\n✅ Results saved to {}", csv_path);

    Ok(())
}
