use cell_dt_core::{
    SimulationManager, SimulationConfig,
    components::{CentriolePair, CellCycleState, Phase},
};
use centriole_module::CentrioleModule;
use rand::Rng;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("=== Cell Differentiation Platform Simulation ===");
    println!("Simple example with centriole module\n");
    
    let config = SimulationConfig {
        max_steps: 100,
        dt: 0.1,
        checkpoint_interval: 20,
        num_threads: Some(4),
        seed: Some(42),
    };
    
    let mut sim = SimulationManager::new(config);
    let centriole_module = CentrioleModule::new();
    sim.register_module(Box::new(centriole_module))?;
    
    initialize_cells(&mut sim, 10)?;
    print_statistics(&sim);
    
    println!("\nStarting simulation...");
    sim.run()?;
    
    println!("\nSimulation finished!");
    print_statistics(&sim);
    
    Ok(())
}

fn initialize_cells(sim: &mut SimulationManager, count: usize) -> Result<(), cell_dt_core::SimulationError> {
    println!("Initializing {} cells...", count);
    
    let world = sim.world_mut();
    let mut rng = rand::thread_rng();
    
    for i in 0..count {
        let entity = world.spawn((
            CentriolePair::default(),
            CellCycleState {
                phase: match rng.gen_range(0..4) {
                    0 => Phase::G1,
                    1 => Phase::S,
                    2 => Phase::G2,
                    _ => Phase::M,
                },
                progress: rng.gen::<f32>(),
            },
        ));
        
        println!("  Created cell {} (entity {:?})", i, entity);
    }
    
    Ok(())
}

fn print_statistics(sim: &SimulationManager) {
    let world = sim.world();
    let mut query = world.query::<(&CentriolePair, &CellCycleState)>();
    
    let mut total_cells = 0;
    let mut total_mother_maturity = 0.0;
    let mut total_daughter_maturity = 0.0;
    let mut total_mtoc_activity = 0.0;
    let mut phase_counts = [0; 4];
    
    // Исправлено: убираем world из iter()
    for (_, (pair, cycle)) in query.iter() {
        total_cells += 1;
        total_mother_maturity += pair.mother.maturity;
        total_daughter_maturity += pair.daughter.maturity;
        total_mtoc_activity += pair.mtoc_activity;
        
        match cycle.phase {
            Phase::G1 => phase_counts[0] += 1,
            Phase::S => phase_counts[1] += 1,
            Phase::G2 => phase_counts[2] += 1,
            Phase::M => phase_counts[3] += 1,
        }
    }
    
    println!("\n=== Statistics at step {} ===", sim.current_step());
    println!("Total cells: {}", total_cells);
    println!("Time: {:.2}", sim.current_time());
    
    if total_cells > 0 {
        println!("\nCentriole statistics:");
        println!("  Average mother maturity: {:.3}", total_mother_maturity / total_cells as f32);
        println!("  Average daughter maturity: {:.3}", total_daughter_maturity / total_cells as f32);
        println!("  Average MTOC activity: {:.3}", total_mtoc_activity / total_cells as f32);
        
        println!("\nCell cycle phases:");
        println!("  G1: {} cells ({:.1}%)", phase_counts[0], 
                 phase_counts[0] as f32 / total_cells as f32 * 100.0);
        println!("  S:  {} cells ({:.1}%)", phase_counts[1],
                 phase_counts[1] as f32 / total_cells as f32 * 100.0);
        println!("  G2: {} cells ({:.1}%)", phase_counts[2],
                 phase_counts[2] as f32 / total_cells as f32 * 100.0);
        println!("  M:  {} cells ({:.1}%)", phase_counts[3],
                 phase_counts[3] as f32 / total_cells as f32 * 100.0);
    }
}
