//! P1 — NichePool: демографический дрейф и CHIP-подобная клональная экспансия.
//!
//! ## Сценарий
//! Пул из 20 HSC-ниш (Blood). Ёмкость пула = 20. При гибели ниши (повреждения
//! превышают порог) здоровый клон симметрично делится и заполняет пустой слот.
//! Поскольку более молодые/здоровые клоны делятся чаще, постепенно они занимают
//! всё больший процент пула — клональная экспансия без положительного отбора.
//!
//! ## Вывод
//! Каждые 10 лет — таблица клонального состава пула (CHIP-анализ):
//! - clone_id, count, доля (%), поколение
//! - Порог CHIP: один клон занимает > 10% пула (Jaiswal et al. 2014)

use cell_dt_core::{SimulationManager, SimulationConfig};
use cell_dt_core::components::{CentriolePair, CellCycleStateExtended, ClonalState, CentriolarDamageState};
use centriole_module::CentrioleModule;
use cell_cycle_module::{CellCycleModule, CellCycleParams};
use human_development_module::{
    HumanDevelopmentModule, HumanDevelopmentParams, HumanDevelopmentComponent,
};
use myeloid_shift_module::MyeloidShiftModule;
use asymmetric_division_module::{AsymmetricDivisionModule, AsymmetricDivisionParams};
use std::collections::HashMap;
use std::io::Write;

const POOL_SIZE: usize = 20;
const CHIP_THRESHOLD: f32 = 0.10; // 10% пула = CHIP по Jaiswal 2014

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Cell DT Platform — NichePool + Клональный дрейф (CHIP) ===");
    println!();
    println!("Теория: CDATA-старение → дифференциальное истощение ниш → клональная экспансия");
    println!("Параметры: {} HSC-ниш, ёмкость пула = {}, порог CHIP = {}%",
        POOL_SIZE, POOL_SIZE, (CHIP_THRESHOLD * 100.0) as u32);
    println!();

    let config = SimulationConfig {
        max_steps: 40_000,
        dt: 1.0,
        checkpoint_interval: 3650,
        num_threads: Some(4),
        seed: None,          // случайный seed → разные траектории каждый запуск
        parallel_modules: false,
        cleanup_dead_interval: None,
    };

    let mut sim = SimulationManager::new(config);

    // Модули в правильном порядке
    sim.register_module(Box::new(CentrioleModule::with_parallel(true)))?;
    sim.register_module(Box::new(CellCycleModule::with_params(CellCycleParams {
        base_cycle_time:           24.0,
        growth_factor_sensitivity: 0.3,
        stress_sensitivity:        0.2,
        checkpoint_strictness:     0.1,
        enable_apoptosis:          true,
        nutrient_availability:     0.9,
        growth_factor_level:       0.8,
        random_variation:          0.2,
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
            mother_bias:                0.5,  // тканевой bias применится автоматически (Blood=0.65)
            age_bias_coefficient:       0.0,
            ptm_exhaustion_scale:       0.001,
            de_novo_centriole_division: 4,
            meiotic_elimination_enabled: true,
            // Ключевой параметр для NichePool:
            // без шума все ниши стареют синхронно → нет дрейфа.
            // noise_scale=0.20 → σ_year ≈ 0.20×√(1/365.25)×√(365.25) = 0.20 per year
            // → смерти разбросаны по ±20 лет вокруг средней продолжительности жизни.
            noise_scale:                0.20,
        }
    )))?;
    sim.register_module(Box::new(MyeloidShiftModule::new()))?;

    // AsymmetricDivisionModule с NichePool
    sim.register_module(Box::new(AsymmetricDivisionModule::with_params(
        AsymmetricDivisionParams {
            asymmetric_division_probability: 0.3,
            symmetric_renewal_probability:   0.4,
            symmetric_diff_probability:      0.3,
            stem_cell_niche_capacity:        10,
            max_niches:                      100,
            spindle_failure_threshold:       0.3,
            max_entities:                    200,    // 20 ниш + запас для замен
            enable_daughter_spawn:           false,  // ← выключен: не нужен для NichePool
            niche_pool_capacity:             POOL_SIZE, // ← ёмкость пула
            enable_niche_competition:        true,   // ← конкуренция включена
            niche_check_interval:            30,     // проверка раз в 30 шагов
        }
    )))?;

    // Создаём POOL_SIZE ниш
    println!("Создаём {} стволовых HSC-ниш...", POOL_SIZE);
    {
        let world = sim.world_mut();
        for _ in 0..POOL_SIZE {
            world.spawn((
                CentriolePair::default(),
                CellCycleStateExtended::new(),
            ));
        }
    }
    println!("Ниши созданы.\n");

    sim.initialize()?;

    // Заголовок
    println!("{:<5} {:>6} {:>6}  {:<40}  {:>6}",
        "Год", "Живых", "Итого", "Топ-5 клонов (id:доля%:пок)", "CHIP?");
    println!("{}", "─".repeat(80));

    for year in 0usize..80 {
        for _ in 0..365 {
            sim.step()?;
        }
        if year % 10 == 0 || year == 79 {
            print_year(year, &sim);
            std::io::stdout().flush()?;
        }
    }

    println!();
    println!("=== Финальный клональный состав ===");
    print_final(&sim);

    Ok(())
}

fn clone_frequency(sim: &SimulationManager) -> HashMap<u64, (u32, u32)> {
    // (clone_id → (count, max_generation))
    let world = sim.world();
    let mut freq: HashMap<u64, (u32, u32)> = HashMap::new();

    // Считаем только живые ниши (с минимальной spindle_fidelity)
    let damage_map: HashMap<_, _> = {
        let mut q = world.query::<&CentriolarDamageState>();
        q.iter().map(|(e, d)| (e, d.spindle_fidelity)).collect()
    };
    let dev_alive: std::collections::HashSet<_> = {
        let mut q = world.query::<&HumanDevelopmentComponent>();
        q.iter().filter(|(_, d)| d.is_alive).map(|(e, _)| e).collect()
    };

    let mut q = world.query::<&ClonalState>();
    for (entity, clonal) in q.iter() {
        let spindle = damage_map.get(&entity).copied().unwrap_or(0.0);
        let alive_in_dev = dev_alive.contains(&entity);
        // Ниша жива если: (есть HumanDevelopmentComponent и alive) ИЛИ (нет HumanDev, но spindle>0.05)
        let is_alive = if dev_alive.is_empty() {
            spindle > 0.05
        } else {
            alive_in_dev
        };
        if is_alive && clonal.clone_id > 0 {
            let entry = freq.entry(clonal.clone_id).or_insert((0, 0));
            entry.0 += 1;
            entry.1 = entry.1.max(clonal.generation);
        }
    }
    freq
}

fn print_year(year: usize, sim: &SimulationManager) {
    let freq = clone_frequency(sim);
    let total_alive: u32 = freq.values().map(|(c, _)| c).sum();
    let world_size = sim.world().len();

    if total_alive == 0 {
        println!("{:<5} {:>6} {:>6}  {:<40}  {:>6}",
            year, 0, world_size, "—нет живых клонов—", "—");
        return;
    }

    let total_f = total_alive as f32;

    // Топ-5 клонов по частоте
    let mut sorted: Vec<_> = freq.iter().collect();
    sorted.sort_by(|a, b| b.1.0.cmp(&a.1.0));

    let top5: Vec<String> = sorted.iter().take(5).map(|(id, (cnt, gen))| {
        let pct = *cnt as f32 / total_f * 100.0;
        format!("{}:{:.0}%:g{}", id, pct, gen)
    }).collect();

    let max_pct = sorted.first().map(|(_, (c, _))| *c as f32 / total_f * 100.0).unwrap_or(0.0);
    let chip = if max_pct >= CHIP_THRESHOLD * 100.0 { "CHIP!" } else { "" };

    println!("{:<5} {:>6} {:>6}  {:<40}  {:>6}",
        year, total_alive, world_size, top5.join(" "), chip);
}

fn print_final(sim: &SimulationManager) {
    let freq = clone_frequency(sim);
    let total_alive: u32 = freq.values().map(|(c, _)| c).sum();

    if total_alive == 0 {
        println!("  Все ниши мертвы.");
        return;
    }

    let total_f = total_alive as f32;
    let mut sorted: Vec<_> = freq.iter().collect();
    sorted.sort_by(|a, b| b.1.0.cmp(&a.1.0));

    println!("{:<10} {:>6} {:>8}  {:>6}  {:>6}",
        "clone_id", "кол-во", "доля%", "поколение", "CHIP?");
    println!("{}", "─".repeat(50));
    for (id, (cnt, gen)) in sorted.iter().take(10) {
        let pct = *cnt as f32 / total_f * 100.0;
        let chip = if pct >= CHIP_THRESHOLD * 100.0 { "← CHIP" } else { "" };
        println!("{:<10} {:>6} {:>7.1}%  {:>6}  {}",
            id, cnt, pct, gen, chip);
    }
    println!();
    println!("Всего живых ниш: {} / {}", total_alive, POOL_SIZE);
    let dominant_clones = sorted.iter()
        .filter(|(_, (c, _))| *c as f32 / total_f >= CHIP_THRESHOLD)
        .count();
    if dominant_clones > 0 {
        println!("CHIP-статус: {} доминирующих клонов (≥{}% пула) — CHIP ОБНАРУЖЕН",
            dominant_clones, (CHIP_THRESHOLD * 100.0) as u32);
    } else {
        println!("CHIP-статус: ни один клон не достиг {}% — CHIP не обнаружен",
            (CHIP_THRESHOLD * 100.0) as u32);
    }
}
