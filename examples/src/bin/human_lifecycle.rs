//! # Симуляция полного жизненного цикла человека (CDATA)
//!
//! Запуск:
//!   cargo run --bin human_lifecycle
//!   cargo run --bin human_lifecycle -- --mode progeria
//!   cargo run --bin human_lifecycle -- --mode longevity
//!
//! Выводит почасовой/годовой снимок ключевых метрик CDATA:
//!   - Стадия развития
//!   - Повреждения центриоли по тканям
//!   - Размер пула стволовых клеток
//!   - Когниция / иммунитет / мышечная масса / дряхлость

use human_development_module::{
    HumanDevelopmentModule, HumanDevelopmentParams,
    damage::DamageParams,
    development::DevelopmentParams,
};
use cell_dt_core::{SimulationModule, components::DevelopmentalStage};

fn main() {
    env_logger::init();

    // --- Разбор аргументов командной строки ---
    let args: Vec<String> = std::env::args().collect();
    let mode = args.iter()
        .position(|a| a == "--mode")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("normal");

    let (damage_params, label) = match mode {
        "progeria"  => (DamageParams::progeria(),  "ПРОГЕРИЯ"),
        "longevity" => (DamageParams::longevity(),  "ДОЛГОЖИТЕЛЬ"),
        _           => (DamageParams::default(),    "НОРМА"),
    };

    let params = HumanDevelopmentParams {
        damage: damage_params,
        development: DevelopmentParams::default(),
        steps_per_year: 10,
        seed: 42,
    };

    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║    CDATA — Симуляция жизненного цикла человека                   ║");
    println!("║    Centriolar Damage Accumulation Theory of Aging                ║");
    println!("║    Режим: {:56}║", label);
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();

    let mut module = HumanDevelopmentModule::with_params(params.clone());

    // Инициализируем мировое состояние (пустой мир — organism-level симуляция)
    let mut world = cell_dt_core::hecs::World::new();
    module.initialize(&mut world).unwrap();

    print_header();

    // Шагаем по 0.1 года за шаг; печатаем каждые 5 лет (50 шагов)
    let total_steps = (params.development.max_lifespan_years * params.steps_per_year as f64) as u64;
    let print_every = params.steps_per_year * 5;  // каждые 5 лет
    let mut prev_stage = DevelopmentalStage::Zygote;

    for step in 0..total_steps {
        use cell_dt_core::SimulationModule;
        module.step(&mut world, 1.0).unwrap();

        let snap = module.snapshot();

        // Выводить при смене стадии или каждые 5 лет
        let stage_changed = snap.stage != prev_stage;
        let periodic = step % print_every == 0;

        if stage_changed || periodic {
            print_row(&snap);
            prev_stage = snap.stage;
        }

        if !snap.is_alive {
            println!();
            println!("  ✦ Смерть организма на {:.1} году жизни", snap.age_years);
            println!("  ✦ Стадия: {:?}", snap.stage);
            println!("  ✦ Индекс дряхлости: {:.3}", snap.frailty);
            break;
        }
    }

    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Финальный отчёт по тканям:");
    println!("═══════════════════════════════════════════════════════════════════");
    let final_snap = module.snapshot();
    for tissue in &final_snap.tissues {
        println!("  {:20} | пул={:.3}  регенерация={:.3}  сенесц.={:.3}  F={:.3}",
            format!("{:?}", tissue.tissue_type),
            tissue.stem_cell_pool,
            tissue.regeneration_tempo,
            tissue.senescent_fraction,
            tissue.functional_capacity,
        );
    }

    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Ключевые показатели CDATA:");
    println!("═══════════════════════════════════════════════════════════════════");
    for tissue in &module.tissues {
        let d = &tissue.damage;
        println!("  {:20} | карбонил={:.3}  ацетил={:.3}  агрег={:.3}",
            format!("{:?}", tissue.state.tissue_type),
            d.protein_carbonylation,
            d.tubulin_hyperacetylation,
            d.protein_aggregates,
        );
        println!("  {:20} | CEP164={:.3}  Ninein={:.3}  цилия={:.3}  веретено={:.3}",
            "",
            d.cep164_integrity,
            d.ninein_integrity,
            d.ciliary_function,
            d.spindle_fidelity,
        );
    }
}

fn print_header() {
    println!("{:>6}  {:16}  {:8}  {:8}  {:8}  {:8}  {:8}  {:8}",
        "Возраст", "Стадия", "Дряхл.", "Когниц.", "Иммун.", "Мышцы", "Inflamm", "Жив?");
    println!("{}", "─".repeat(80));
}

fn print_row(snap: &human_development_module::OrganismSnapshot) {
    let stage_str = format!("{:?}", snap.stage);
    println!("{:>6.1}  {:16}  {:.3}  {:.3}  {:.3}  {:.3}  {:.3}     {}",
        snap.age_years,
        &stage_str[..stage_str.len().min(16)],
        snap.frailty,
        snap.cognitive,
        snap.immune,
        snap.muscle,
        snap.inflammaging,
        if snap.is_alive { "+" } else { "✗" },
    );
}
