use crate::{
    SimulationError, SimulationModule, SimulationResult,
    hecs::World,
};
use std::collections::HashMap;
use std::time::Instant;
use log::{info, debug, warn};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub max_steps: u64,
    pub dt: f64,
    pub checkpoint_interval: u64,
    pub num_threads: Option<usize>,
    pub seed: Option<u64>,
    pub parallel_modules: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            max_steps: 1000,
            dt: 0.1,
            checkpoint_interval: 100,
            num_threads: None,
            seed: Some(42),
            parallel_modules: false,
        }
    }
}

pub struct SimulationManager {
    world: World,
    modules: HashMap<String, Box<dyn SimulationModule>>,
    config: SimulationConfig,
    current_step: u64,
    current_time: f64,
    module_execution_times: Arc<Mutex<HashMap<String, Vec<std::time::Duration>>>>,
}

impl SimulationManager {
    pub fn new(config: SimulationConfig) -> Self {
        if let Some(seed) = config.seed {
            info!("Using random seed: {}", seed);
        }
        
        if let Some(num_threads) = config.num_threads {
            rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build_global()
                .unwrap_or_else(|_| warn!("Failed to set Rayon thread pool"));
        }
        
        Self {
            world: World::new(),
            modules: HashMap::new(),
            config,
            current_step: 0,
            current_time: 0.0,
            module_execution_times: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn register_module(&mut self, module: Box<dyn SimulationModule>) -> SimulationResult<()> {
        let name = module.name().to_string();
        
        if self.modules.contains_key(&name) {
            return Err(SimulationError::ModuleError(
                format!("Module '{}' already registered", name)
            ));
        }
        
        info!("Registering module: {}", name);
        self.modules.insert(name, module);
        Ok(())
    }
    
    pub fn initialize(&mut self) -> SimulationResult<()> {
        info!("Initializing simulation with {} modules", self.modules.len());
        
        for (name, module) in self.modules.iter_mut() {
            debug!("Initializing module: {}", name);
            module.initialize(&mut self.world)?;
        }
        
        Ok(())
    }
    
    pub fn step(&mut self) -> SimulationResult<()> {
        if self.current_step >= self.config.max_steps {
            return Ok(());
        }
        
        let step_start = Instant::now();
        let dt = self.config.dt;
        
        for (name, module) in self.modules.iter_mut() {
            debug!("Executing module: {} at step {}", name, self.current_step);
            
            let module_start = Instant::now();
            module.step(&mut self.world, dt)?;
            
            let module_time = module_start.elapsed();
            
            if let Ok(mut times) = self.module_execution_times.lock() {
                times.entry(name.to_string())
                    .or_insert_with(Vec::new)
                    .push(module_time);
            }
            
            if module_time.as_millis() > 100 {
                warn!("Module {} took {:?}", name, module_time);
            }
        }
        
        self.current_step += 1;
        self.current_time += dt;
        
        let step_time = step_start.elapsed();
        debug!("Step {} completed in {:?}", self.current_step, step_time);
        
        Ok(())
    }
    
    pub fn run(&mut self) -> SimulationResult<()> {
        self.initialize()?;
        
        info!(
            "Starting simulation: {} steps, dt = {}", 
            self.config.max_steps, 
            self.config.dt,
        );
        
        let start_time = Instant::now();
        
        while self.current_step < self.config.max_steps {
            self.step()?;
            
            if self.config.checkpoint_interval > 0 && 
               self.current_step % self.config.checkpoint_interval == 0 {
                info!("Checkpoint at step {}", self.current_step);
            }
        }
        
        let total_time = start_time.elapsed();
        info!("Simulation completed in {:?}. Final time: {}", total_time, self.current_time);
        
        self.print_performance_stats();
        
        Ok(())
    }
    
    fn print_performance_stats(&self) {
        if let Ok(times) = self.module_execution_times.lock() {
            info!("\n=== Performance Statistics ===");
            for (module_name, durations) in times.iter() {
                if !durations.is_empty() {
                    let total: std::time::Duration = durations.iter().sum();
                    let avg = total / durations.len() as u32;
                    info!("Module {}: {} calls, total {:?}, avg {:?}", 
                          module_name, durations.len(), total, avg);
                }
            }
        }
    }
    
    pub fn world(&self) -> &World {
        &self.world
    }
    
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
    
    pub fn current_step(&self) -> u64 {
        self.current_step
    }
    
    pub fn current_time(&self) -> f64 {
        self.current_time
    }
    
    pub fn config(&self) -> &SimulationConfig {
        &self.config
    }
}
