use cell_dt_core::{
    SimulationManager,
    components::{CentriolePair, CellCycleStateExtended},
};
use centriole_module::CentrioleModule;
use cell_cycle_module::{CellCycleModule, CellCycleParams};
use cell_dt_io::{
    DataExporter,
    load_json_config, save_json_config,
    SimulationConfigFull, ModuleConfigs,
};
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Cell DT Platform - Data Export Example ===\n");
    
    // Create directories for output data
    std::fs::create_dir_all("io_output/data")?;
    std::fs::create_dir_all("io_output/configs")?;
    
    // Save example configuration
    let config_full = SimulationConfigFull {
        simulation: cell_dt_io::SimulationConfig {
            max_steps: 200,
            dt: 0.05,
            num_threads: Some(4),
            seed: Some(42),
        },
        modules: ModuleConfigs {
            centriole: Some(serde_json::json!({
                "acetylation_rate": 0.02,
                "parallel_cells": true,
            })),
            cell_cycle: Some(serde_json::json!({
                "base_cycle_time": 15.0,
                "checkpoint_strictness": 0.1,
            })),
        },
    };
    
    save_json_config("io_output/configs/config.json", &config_full)?;
    println!("✅ Saved configuration to io_output/configs/config.json");
    
    // Load configuration
    let loaded_config = load_json_config("io_output/configs/config.json")?;
    println!("📋 Loaded configuration: simulation steps = {}", loaded_config.simulation.max_steps);
    
    // Create simulation configuration
    let sim_config = cell_dt_core::SimulationConfig {
        max_steps: loaded_config.simulation.max_steps,
        dt: loaded_config.simulation.dt,
        checkpoint_interval: 100,
        num_threads: loaded_config.simulation.num_threads,
        seed: loaded_config.simulation.seed,
        parallel_modules: false,
    };
    
    // Initialize simulation
    let mut sim = SimulationManager::new(sim_config);
    
    // Register modules
    sim.register_module(Box::new(CentrioleModule::with_parallel(true)))?;
    
    let cell_cycle_params = CellCycleParams {
        base_cycle_time: 15.0,
        growth_factor_sensitivity: 0.3,
        stress_sensitivity: 0.2,
        checkpoint_strictness: 0.1,
        enable_apoptosis: true,
        nutrient_availability: 0.9,
        growth_factor_level: 0.8,
        random_variation: 0.2,
    };
    sim.register_module(Box::new(CellCycleModule::with_params(cell_cycle_params)))?;
    
    // Initialize cells
    initialize_cells(&mut sim, 10)?;
    
    // Create data exporter
    let mut exporter = DataExporter::new("io_output/data", "simulation");
    
    println!("\n🚀 Starting simulation with data export...");
    println!("   Data will be saved to io_output/data/\n");
    
    sim.initialize()?;
    
    for step in 0..sim.config().max_steps {
        sim.step()?;
        
        // Collect data every 10th step
        if step % 10 == 0 {
            exporter.collect_data(sim.world(), sim.current_step(), sim.current_time())?;
        }
        
        // Save data every 50th step
        if step % 50 == 0 && step > 0 {
            let path = exporter.save_snapshot(step)?;
            println!("   💾 Saved data: {}", path.display());
        }
        
        // Show progress
        if step % 50 == 0 {
            println!("   Step {}/{}", step, sim.config().max_steps);
        }
    }
    
    // Final export
    println!("\n📊 Performing final export...");
    let final_path = exporter.save_snapshot(sim.current_step())?;
    println!("   ✅ Final data saved to: {}", final_path.display());
    
    // Final statistics
    println!("\n=== Final Statistics ===");
    println!("Total steps: {}", sim.current_step());
    println!("Total cells: {}", sim.world().query::<()>().iter().count());
    
    Ok(())
}

fn initialize_cells(sim: &mut SimulationManager, count: usize) -> Result<(), cell_dt_core::SimulationError> {
    print!("Initializing {} cells...", count);
    std::io::stdout().flush()?;
    
    let world = sim.world_mut();
    
    for i in 0..count {
        let _entity = world.spawn((
            CentriolePair::default(),
            CellCycleStateExtended::new(),
        ));
        
        if i % 5 == 0 {
            print!(".");
            std::io::stdout().flush()?;
        }
    }
    println!(" done!");
    
    Ok(())
}
