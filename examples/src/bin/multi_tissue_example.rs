//! P7 — Многотканевая модель: 5 тканей × 5 ниш = 25 HSC-подобных сущностей.
//!
//! ## Сценарий
//! Симулируем один организм с пятью тканевыми компартментами:
//! Blood (HSC), Neural, Epithelial (Gut), Muscle, Skin.
//! Каждая ткань представлена 5 нишами. OrganismState агрегирует
//! все ниши и определяет момент гибели организма.
//!
//! ## P7-механизмы
//! - **Системный SASP**: сенесцентные клетки одной ткани ускоряют старение других
//!   через ros_boost (лаг 1 шаг, масштаб 5%)
//! - **Ось IGF-1/GH**: пик в 20 лет, снижение до 0.3 к 90 годам → регенерация ↓
//!
//! ## Вывод
//! Каждые 10 лет — состояние тканей + OrganismState.
//! В конце: последовательность отказа тканей.

use cell_dt_core::{SimulationManager, SimulationConfig};
use cell_dt_core::components::{CentriolePair, CellCycleStateExtended};
use centriole_module::CentrioleModule;
use cell_cycle_module::{CellCycleModule, CellCycleParams};
use human_development_module::{
    HumanDevelopmentModule, HumanDevelopmentParams, HumanDevelopmentComponent,
    HumanTissueType,
};
use myeloid_shift_module::MyeloidShiftModule;
use std::collections::HashMap;
use std::io::Write;

const NICHES_PER_TISSUE: usize = 5;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Cell DT Platform — Многотканевая модель организма (P7) ===");
    println!();
    println!("Ткани: Blood×{n}, Neural×{n}, Epithelial×{n}, Muscle×{n}, Skin×{n}",
        n = NICHES_PER_TISSUE);
    println!("P7-механизмы: системный SASP + ось IGF-1/GH");
    println!();

    let config = SimulationConfig {
        max_steps: 40_000,
        dt: 1.0,
        checkpoint_interval: 3650,
        num_threads: Some(4),
        seed: Some(42),
        parallel_modules: false,
        cleanup_dead_interval: None,
    };

    let mut sim = SimulationManager::new(config);

    sim.register_module(Box::new(CentrioleModule::with_parallel(true)))?;
    sim.register_module(Box::new(CellCycleModule::with_params(CellCycleParams {
        base_cycle_time:           24.0,
        growth_factor_sensitivity: 0.3,
        stress_sensitivity:        0.2,
        checkpoint_strictness:     0.1,
        enable_apoptosis:          true,
        nutrient_availability:     0.9,
        growth_factor_level:       0.8,
        random_variation:          0.1,
    })))?;
    sim.register_module(Box::new(HumanDevelopmentModule::with_params(
        HumanDevelopmentParams {
            time_acceleration:          1.0,
            enable_aging:               true,
            enable_morphogenesis:       true,
            tissue_detail_level:        3,
            mother_inducer_count:       10,
            daughter_inducer_count:     8,
            base_detach_probability:    0.0003,
            mother_bias:                0.5,
            age_bias_coefficient:       0.0,
            ptm_exhaustion_scale:       0.001,
            de_novo_centriole_division: 4,
            meiotic_elimination_enabled: true,
            noise_scale:                0.05, // небольшой шум для вариации
        }
    )))?;
    sim.register_module(Box::new(MyeloidShiftModule::new()))?;

    // Создаём 5 × 5 ниш с явным указанием ткани
    println!("Создаём {} ниш...", NICHES_PER_TISSUE * 5);
    {
        let world = sim.world_mut();
        let tissues = [
            HumanTissueType::Blood,
            HumanTissueType::Neural,
            HumanTissueType::Epithelial,
            HumanTissueType::Muscle,
            HumanTissueType::Skin,
        ];
        for tissue in &tissues {
            for _ in 0..NICHES_PER_TISSUE {
                // Ткань закодирована в HumanDevelopmentComponent — назначается
                // HumanDevelopmentModule.initialize() по cycle. Поскольку мы создаём
                // ровно 5×5 = 25 ниш и tissue_cycle имеет 5 элементов,
                // каждая ткань получит ровно 5 ниш автоматически.
                let _ = tissue; // явное указание будет при спавне ниже
                world.spawn((
                    CentriolePair::default(),
                    CellCycleStateExtended::new(),
                ));
            }
        }
    }
    println!("Ниши созданы.\n");

    sim.initialize()?;

    // Заголовок таблицы
    println!("{:<5} {:>6} {:>7} {:>7} {:>7} {:>7} {:>7}  {:>6} {:>6}  Смерть?",
        "Год", "Blood", "Neural", "Gut", "Muscle", "Skin", "Frailty",
        "IGF-1", "SASP");
    println!("{}", "─".repeat(90));

    // Отслеживаем время первого отказа каждой ткани
    let mut tissue_death_year: HashMap<String, usize> = HashMap::new();

    for year in 0usize..110 {
        for _ in 0..365 {
            sim.step()?;
        }
        if year % 10 == 0 || year == 109 {
            print_year(year, &sim, &mut tissue_death_year);
            std::io::stdout().flush()?;
        }

        // Проверяем OrganismState через get_params()
        let params = sim.get_module_params("human_development_module")?;
        let is_alive = params.get("organism_is_alive")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        if !is_alive {
            let cause = params.get("organism_death_cause")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let age = params.get("organism_age_years")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            println!();
            println!("▶ ОРГАНИЗМ УМЕР на году {} (возраст {:.1} лет): причина = {}",
                year, age, cause);
            break;
        }
    }

    // Последовательность отказа тканей
    println!();
    println!("=== Последовательность отказа тканей ===");
    if tissue_death_year.is_empty() {
        println!("  Ни одна ткань не достигла 50% потери FC за время симуляции.");
    } else {
        let mut seq: Vec<_> = tissue_death_year.iter().collect();
        seq.sort_by_key(|(_, y)| *y);
        for (tissue, year) in seq {
            println!("  {:12} — год {:>3}", tissue, year);
        }
    }

    Ok(())
}

fn print_year(
    year: usize,
    sim: &SimulationManager,
    tissue_death_year: &mut HashMap<String, usize>,
) {
    let world = sim.world();

    // Агрегируем FC по тканям
    let mut tissue_fc: HashMap<&str, (f32, u32)> = HashMap::new();
    for (_, comp) in world.query::<&HumanDevelopmentComponent>().iter() {
        if !comp.is_alive { continue; }
        let key = match comp.tissue_type {
            HumanTissueType::Blood      => "Blood",
            HumanTissueType::Neural     => "Neural",
            HumanTissueType::Epithelial => "Gut",
            HumanTissueType::Muscle     => "Muscle",
            HumanTissueType::Skin       => "Skin",
            _                           => "Other",
        };
        let e = tissue_fc.entry(key).or_insert((0.0, 0));
        e.0 += comp.tissue_state.functional_capacity;
        e.1 += 1;
    }

    let mut fc_str = |key: &str| -> String {
        if let Some((sum, cnt)) = tissue_fc.get(key) {
            if *cnt == 0 { return "  DEAD".to_string(); }
            let mean = sum / *cnt as f32;
            // Отмечаем первый отказ (FC < 0.5)
            if mean < 0.5 && !tissue_death_year.contains_key(key) {
                tissue_death_year.insert(key.to_string(), year);
            }
            format!("{:.3}", mean)
        } else {
            "  DEAD".to_string()
        }
    };

    // Читаем OrganismState из get_params()
    let (frailty, igf1, sasp, is_alive) =
        if let Ok(params) = sim.get_module_params("human_development_module") {
            (
                params.get("organism_frailty").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                params.get("organism_igf1_level").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                params.get("organism_systemic_sasp").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                params.get("organism_is_alive").and_then(|v| v.as_bool()).unwrap_or(true),
            )
        } else {
            (0.0, 1.0, 0.0, true)
        };

    let death_flag = if !is_alive { "☠ DEAD" } else { "" };

    println!("{:<5} {:>6} {:>7} {:>7} {:>7} {:>7} {:>7.3}  {:>6.3} {:>6.3}  {}",
        year,
        fc_str("Blood"),
        fc_str("Neural"),
        fc_str("Gut"),
        fc_str("Muscle"),
        fc_str("Skin"),
        frailty, igf1, sasp,
        death_flag,
    );
}
