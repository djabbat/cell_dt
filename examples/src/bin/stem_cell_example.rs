use cell_dt_core::{
    SimulationManager, SimulationConfig,
    components::{CentriolePair, CellCycleStateExtended},
};
use centriole_module::CentrioleModule;
use cell_cycle_module::{CellCycleModule, CellCycleParams};
use asymmetric_division_module::{AsymmetricDivisionModule, AsymmetricDivisionParams};
use stem_cell_hierarchy_module::{StemCellHierarchyModule, StemCellHierarchyParams, factories};
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Cell DT Platform - Stem Cell Biology Example ===\n");
    
    let config = SimulationConfig {
        max_steps: 200,
        dt: 0.1,
        checkpoint_interval: 50,
        num_threads: Some(4),
        seed: Some(42),
        parallel_modules: false,
    };
    
    let mut sim = SimulationManager::new(config);
    
    // 1. Модуль центриоли
    sim.register_module(Box::new(CentrioleModule::with_parallel(true)))?;
    println!("✅ Centriole module registered");
    
    // 2. Модуль клеточного цикла
    let cell_cycle_params = CellCycleParams {
        base_cycle_time: 20.0,
        ..Default::default()
    };
    sim.register_module(Box::new(CellCycleModule::with_params(cell_cycle_params)))?;
    println!("✅ Cell cycle module registered");
    
    // 3. Модуль асимметричного деления
    let asymmetric_params = AsymmetricDivisionParams {
        asymmetric_division_probability: 0.4,
        symmetric_renewal_probability: 0.4,
        symmetric_diff_probability: 0.2,
        stem_cell_niche_capacity: 5,
        max_niches: 10,
    };
    let mut asymmetric_module = AsymmetricDivisionModule::with_params(asymmetric_params);
    
    // Создаем ниши
    for i in 0..3 {
        let niche_id = asymmetric_module.create_niche(0.0, 0.0, (i * 10) as f32, 5.0);
        println!("  Created niche {} at position (0, 0, {})", niche_id, i * 10);
    }
    
    sim.register_module(Box::new(asymmetric_module))?;
    println!("✅ Asymmetric division module registered");
    
    // 4. Модуль иерархии стволовых клеток
    let hierarchy_params = StemCellHierarchyParams {
        initial_potency: stem_cell_hierarchy_module::PotencyLevel::Pluripotent,
        enable_plasticity: true,
        plasticity_rate: 0.01,
        differentiation_threshold: 0.7,
    };
    sim.register_module(Box::new(StemCellHierarchyModule::with_params(hierarchy_params)))?;
    println!("✅ Stem cell hierarchy module registered");
    
    // Инициализируем стволовые клетки
    initialize_stem_cells(&mut sim, 10)?;
    
    println!("\n🚀 Starting stem cell simulation...\n");
    
    sim.initialize()?;
    
    for step in 0..sim.config().max_steps {
        sim.step()?;
        
        if step % 50 == 0 {
            println!("   Step {}/{}", step, sim.config().max_steps);
        }
    }
    
    println!("\n✅ Simulation completed!");
    Ok(())
}

fn initialize_stem_cells(sim: &mut SimulationManager, count: usize) -> Result<(), cell_dt_core::SimulationError> {
    println!("\nInitializing {} stem cells...", count);
    
    let world = sim.world_mut();
    
    for i in 0..count {
        let hierarchy = if i < 3 {
            factories::create_embryonic_stem_cell()
        } else if i < 6 {
            factories::create_hematopoietic_stem_cell()
        } else {
            factories::create_neural_stem_cell()
        };
        
        let _entity = world.spawn((
            CentriolePair::default(),
            CellCycleStateExtended::new(),
            hierarchy,
        ));
        
        if i % 3 == 0 {
            print!(".");
            std::io::stdout().flush()?;
        }
    }
    println!(" done!");
    
    Ok(())
}
