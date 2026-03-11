//! P11 — Терапевтические интервенции: сравнение 4 стратегий × 100 лет.
//!
//! Демонстрирует применение InterventionSchedule в CDATA-симуляции:
//! — Контроль (без терапии)
//! — Сенолитики с 60 лет
//! — NAD⁺ пожизненно с 40 лет
//! — Калорийное ограничение + Активация TERT с 50 лет
//!
//! ## Метрики вывода
//! ```
//! Strategy             Age@Death  Healthspan  Damage@70  Frailty@70  Senescent@70
//! ```
//! Healthspan = годы с total_damage_score < 0.5.

use cell_dt_core::{SimulationManager, SimulationConfig, SimulationModule};
use cell_dt_core::components::CellCycleStateExtended;
use human_development_module::{
    HumanDevelopmentModule, HumanDevelopmentParams,
    HumanDevelopmentComponent,
    Intervention, InterventionKind,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  CDATA — Therapeutic Interventions (P11)                             ║");
    println!("║  Comparing 4 strategies × 100-year simulation                        ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();

    let strategies: Vec<(&str, Vec<Intervention>)> = vec![
        // 1. Контроль — никаких интервенций
        ("Control (no therapy)", vec![]),

        // 2. Сенолитики — начиная с 60 лет, clearance_rate 0.4/год
        //    Аналог Dasatinib+Quercetin: убирает ~40% сенесцентных клеток в год
        ("Senolytics from age 60", vec![
            Intervention {
                start_age_years: 60.0,
                end_age_years: None,
                kind: InterventionKind::Senolytics { clearance_rate: 0.4 },
            },
        ]),

        // 3. NAD⁺ (NMN) с 40 лет — активирует митофагию, снижает ROS и агрегацию
        ("NAD⁺ from age 40", vec![
            Intervention {
                start_age_years: 40.0,
                end_age_years: None,
                kind: InterventionKind::NadPlus { mitophagy_boost: 0.4 },
            },
        ]),

        // 4. Комбинация: калорийное ограничение + TERT-активация с 50 лет
        //    CR снижает ROS на 30%; TERT удлиняет теломеры
        ("CR + TERT from age 50", vec![
            Intervention {
                start_age_years: 50.0,
                end_age_years: None,
                kind: InterventionKind::CaloricRestriction { ros_factor: 0.7 },
            },
            Intervention {
                start_age_years: 50.0,
                end_age_years: None,
                kind: InterventionKind::TertActivation { elongation_per_div: 0.0003 },
            },
        ]),
    ];

    // --- Заголовок таблицы ---
    println!("{:<28} {:>10} {:>12} {:>11} {:>11} {:>13}",
        "Strategy", "Age@Death", "Healthspan", "Damage@70", "Frailty@70", "Senescent@70");
    println!("{}", "─".repeat(87));

    for (name, interventions) in strategies {
        let result = run_simulation(name, interventions)?;
        println!("{:<28} {:>9.1}yr {:>10.1}yr {:>11.3} {:>11.3} {:>13.3}",
            name,
            result.age_at_death,
            result.healthspan_years,
            result.damage_at_70,
            result.frailty_at_70,
            result.senescent_at_70,
        );
    }

    println!();
    println!("Healthspan = years with total_damage_score < 0.5");
    println!("Theory: CDATA (Centriolar Damage Accumulation Theory of Aging, Tkemaladze)");

    Ok(())
}

struct SimResult {
    age_at_death:     f64,
    healthspan_years: f64,
    damage_at_70:     f32,
    frailty_at_70:    f32,
    senescent_at_70:  f32,
}

fn run_simulation(
    _name: &str,
    interventions: Vec<Intervention>,
) -> Result<SimResult, Box<dyn std::error::Error>> {

    let config = SimulationConfig {
        max_steps: 43_800,  // 120 лет × 365 дней
        dt: 1.0,
        checkpoint_interval: 36500,
        num_threads: Some(1),
        seed: Some(42),
        parallel_modules: false,
        cleanup_dead_interval: None,
    };

    let mut sim = SimulationManager::new(config);

    // Модуль развития с P11-интервенциями
    let dev_params = HumanDevelopmentParams {
        time_acceleration:         1.0,
        enable_aging:              true,
        enable_morphogenesis:      true,
        tissue_detail_level:       3,
        mother_inducer_count:      10,
        daughter_inducer_count:    8,
        base_detach_probability:   0.0003,
        mother_bias:               0.5,
        age_bias_coefficient:      0.0,
        ptm_exhaustion_scale:      0.001,
        de_novo_centriole_division:   4,
        meiotic_elimination_enabled: true,
        noise_scale:               0.0,
    };

    let mut dev_module = HumanDevelopmentModule::with_params(dev_params);
    dev_module.set_seed(42);
    for iv in interventions {
        dev_module.add_intervention(iv);
    }

    sim.register_module(Box::new(dev_module))?;

    // Одна ниша — CellCycleStateExtended нужен для initialize() human_development_module
    {
        let world = sim.world_mut();
        let _ = world.spawn((
            CellCycleStateExtended::new(),
        ));
    }

    sim.initialize()?;

    let mut age_at_death:    f64 = 120.0;
    let mut damage_at_70:    f32 = 0.0;
    let mut frailty_at_70:   f32 = 0.0;
    let mut senescent_at_70: f32 = 0.0;
    let mut healthspan_days: f64 = 0.0;
    let mut recorded_70          = false;

    for day in 0u64..43_800 {
        sim.step()?;

        // Healthspan: считаем дни с damage < 0.5
        {
            let world = sim.world();
            let mut q = world.query::<&HumanDevelopmentComponent>();
            if let Some((_, dev)) = q.iter().find(|(_, d)| d.is_alive) {
                if dev.centriolar_damage.total_damage_score() < 0.5 {
                    healthspan_days += 1.0;
                }
            }
        }

        // Снимаем метрики в 70 лет
        let year = day as f64 / 365.25;
        if !recorded_70 && year >= 70.0 {
            let world = sim.world();
            let mut q = world.query::<&HumanDevelopmentComponent>();
            if let Some((_, dev)) = q.iter().next() {
                damage_at_70    = dev.centriolar_damage.total_damage_score();
                frailty_at_70   = dev.frailty();
                senescent_at_70 = dev.tissue_state.senescent_fraction;
            }
            recorded_70 = true;
        }

        // Проверяем смерть
        {
            let world = sim.world();
            let mut q = world.query::<&HumanDevelopmentComponent>();
            let all_dead = q.iter().all(|(_, d)| !d.is_alive);
            if all_dead && day > 365 {
                let world = sim.world();
                let mut q2 = world.query::<&HumanDevelopmentComponent>();
                if let Some((_, dev)) = q2.iter().next() {
                    age_at_death = dev.age_years();
                }
                break;
            }
        }
    }

    Ok(SimResult {
        age_at_death,
        healthspan_years: healthspan_days / 365.25,
        damage_at_70,
        frailty_at_70,
        senescent_at_70,
    })
}
