use cell_dt_core::{
    SimulationManager, SimulationConfig,
    components::{CentriolePair, CellCycleStateExtended, Phase},
};
use centriole_module::CentrioleModule;
use cell_cycle_module::{CellCycleModule, CellCycleParams};
use cell_dt_viz::{
    VisualizationManager,
    ScatterPlotVisualizer,
    HeatmapVisualizer,
    TimeSeriesVisualizer,
};
use rand::Rng;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Cell DT Platform - Cell Cycle Module ===\n");
    
    std::fs::create_dir_all("cell_cycle_output")?;
    
    let config = SimulationConfig {
        max_steps: 1000,
        dt: 0.1,
        checkpoint_interval: 100,
        num_threads: Some(4),
        seed: Some(42),
        parallel_modules: false,
    };
    
    let max_steps = config.max_steps;
    
    let mut sim = SimulationManager::new(config);
    
    // Регистрируем модуль центриоли
    let centriole_module = CentrioleModule::with_parallel(true);
    sim.register_module(Box::new(centriole_module))?;
    
    // Регистрируем модуль клеточного цикла со ВСЕМИ полями
    let cell_cycle_params = CellCycleParams {
        base_cycle_time: 24.0,
        growth_factor_sensitivity: 1.0,
        stress_sensitivity: 0.8,
        checkpoint_strictness: 0.9,
        enable_apoptosis: true,
        nutrient_availability: 0.9,
        growth_factor_level: 0.8,
        random_variation: 0.2,
    };
    let cell_cycle_module = CellCycleModule::with_params(cell_cycle_params);
    sim.register_module(Box::new(cell_cycle_module))?;
    
    // Инициализируем клетки
    initialize_cells(&mut sim, 100)?;
    
    // Настраиваем визуализацию
    let mut viz_manager = VisualizationManager::new(10);
    viz_manager.add_visualizer(Box::new(ScatterPlotVisualizer::new("cell_cycle_output/scatter")));
    viz_manager.add_visualizer(Box::new(HeatmapVisualizer::new("cell_cycle_output/heatmap")));
    
    let data_history = viz_manager.data_history.clone();
    viz_manager.add_visualizer(Box::new(TimeSeriesVisualizer::new("cell_cycle_output/timeseries", data_history)));
    
    println!("\n📊 Starting simulation with real cell cycle biology...");
    println!("   Output will be saved to ./cell_cycle_output/\n");
    
    sim.initialize()?;
    
    for step in 0..max_steps {
        sim.step()?;
        
        viz_manager.update(sim.world(), sim.current_step(), sim.current_time())?;
        
        if step % 100 == 0 {
            print_progress(step, max_steps, &sim);
        }
    }
    
    println!("\n✅ Simulation completed!");
    print_final_stats(&sim);
    
    Ok(())
}

fn initialize_cells(sim: &mut SimulationManager, count: usize) -> Result<(), cell_dt_core::SimulationError> {
    println!("Initializing {} cells with cell cycle states...", count);
    
    let world = sim.world_mut();
    let mut rng = rand::thread_rng();
    
    for i in 0..count {
        let mut cell_cycle = CellCycleStateExtended::new();
        
        // Случайная начальная фаза
        cell_cycle.phase = match rng.gen_range(0..4) {
            0 => Phase::G1,
            1 => Phase::S,
            2 => Phase::G2,
            _ => Phase::M,
        };
        
        cell_cycle.progress = rng.gen::<f32>();
        
        let _entity = world.spawn((
            CentriolePair::default(),
            cell_cycle,
        ));
        
        if i % 20 == 0 {
            print!(".");
            std::io::stdout().flush()?;
        }
    }
    println!(" done!");
    
    Ok(())
}

fn print_progress(step: u64, max_steps: u64, sim: &SimulationManager) {
    let world = sim.world();
    let mut query = world.query::<&CellCycleStateExtended>();
    
    let mut phase_counts = [0; 4];
    let mut arrested = 0;
    let mut total_cycles = 0;
    let mut total_cells = 0;
    
    for (_, cycle) in query.iter() {
        total_cells += 1;
        match cycle.phase {
            Phase::G1 => phase_counts[0] += 1,
            Phase::S => phase_counts[1] += 1,
            Phase::G2 => phase_counts[2] += 1,
            Phase::M => phase_counts[3] += 1,
        }
        
        if cycle.current_checkpoint.is_some() {
            arrested += 1;
        }
        
        total_cycles += cycle.cycle_count;
    }
    
    let progress = step as f32 / max_steps as f32 * 100.0;
    println!("\n📊 Step {}/{} ({:.1}%)", step, max_steps, progress);
    println!("   Phases: G1:{} S:{} G2:{} M:{}", 
             phase_counts[0], phase_counts[1], phase_counts[2], phase_counts[3]);
    if total_cells > 0 {
        println!("   Arrested: {}, Avg cycles: {:.2}", 
                 arrested, total_cycles as f32 / total_cells as f32);
    }
}

fn print_final_stats(sim: &SimulationManager) {
    let world = sim.world();
    let mut query = world.query::<&CellCycleStateExtended>();
    
    let mut total_cells = 0;
    let mut total_cycles = 0;
    let mut max_cycles = 0;
    let mut phase_counts = [0; 4];
    let mut arrested = 0;
    
    for (_, cycle) in query.iter() {
        total_cells += 1;
        total_cycles += cycle.cycle_count;
        max_cycles = max_cycles.max(cycle.cycle_count);
        
        match cycle.phase {
            Phase::G1 => phase_counts[0] += 1,
            Phase::S => phase_counts[1] += 1,
            Phase::G2 => phase_counts[2] += 1,
            Phase::M => phase_counts[3] += 1,
        }
        
        if cycle.current_checkpoint.is_some() {
            arrested += 1;
        }
    }
    
    println!("\n=== Final Statistics ===");
    println!("Total cells: {}", total_cells);
    println!("Phase distribution: G1={}, S={}, G2={}, M={}", 
             phase_counts[0], phase_counts[1], phase_counts[2], phase_counts[3]);
    println!("Cells arrested: {}", arrested);
    if total_cells > 0 {
        println!("Average cycles completed: {:.2}", total_cycles as f32 / total_cells as f32);
    }
    println!("Maximum cycles: {}", max_cycles);
}
