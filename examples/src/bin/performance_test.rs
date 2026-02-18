use cell_dt_core::{
    SimulationManager, SimulationConfig,
    components::{CentriolePair, CellCycleState, Phase},
};
use centriole_module::CentrioleModule;
use rand::Rng;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("=== Cell DT Performance Test ===\n");
    
    // Тестируем с разным количеством клеток
    let cell_counts = [100, 1000, 10000];
    
    for &num_cells in &cell_counts {
        println!("\n📊 Testing with {} cells", num_cells);
        println!("{}", "=".repeat(50));
        
        // Тест с параллельной обработкой клеток
        let time_parallel = test_with_config(num_cells, true)?;
        println!("  ⚡ Parallel cells:   {:.3}s", time_parallel);
        
        // Тест без параллельной обработки
        let time_sequential = test_with_config(num_cells, false)?;
        println!("  🐢 Sequential cells: {:.3}s", time_sequential);
        
        if time_sequential > 0.0 {
            let speedup = time_sequential / time_parallel;
            println!("  📈 Speedup:         {:.2}x", speedup);
        }
    }
    
    Ok(())
}

fn test_with_config(num_cells: usize, parallel_cells: bool) -> Result<f64, Box<dyn std::error::Error>> {
    let config = SimulationConfig {
        max_steps: 50,
        dt: 0.1,
        checkpoint_interval: 1000,
        num_threads: Some(8),
        seed: Some(42),
        parallel_modules: false,  // Модули выполняем последовательно
    };
    
    let mut sim = SimulationManager::new(config);
    
    // Создаем модуль центриоли с настройкой параллельной обработки клеток
    let centriole_module = CentrioleModule::with_parallel(parallel_cells);
    sim.register_module(Box::new(centriole_module))?;
    
    // Инициализируем клетки
    initialize_cells(&mut sim, num_cells)?;
    
    // Запускаем симуляцию и замеряем время
    let start = Instant::now();
    sim.run()?;
    let duration = start.elapsed().as_secs_f64();
    
    Ok(duration)
}

fn initialize_cells(sim: &mut SimulationManager, count: usize) -> Result<(), cell_dt_core::SimulationError> {
    let world = sim.world_mut();
    let mut rng = rand::thread_rng();
    
    for _ in 0..count {
        world.spawn((
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
    }
    
    Ok(())
}
