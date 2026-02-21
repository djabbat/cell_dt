use crate::{
    SimulationError, SimulationModule, SimulationResult,
    hecs::World,
};
use std::collections::HashMap;
use std::time::Instant;
use log::{info, debug, warn};

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
        
        let dt = self.config.dt;
        
        for (_, module) in self.modules.iter_mut() {
            module.step(&mut self.world, dt)?;
        }
        
        self.current_step += 1;
        self.current_time += dt;
        
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
        }
        
        let total_time = start_time.elapsed();
        info!("Simulation completed in {:?}. Final time: {}", total_time, self.current_time);
        
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestModule;
    
    impl SimulationModule for TestModule {
        fn name(&self) -> &str { "test_module" }
        fn step(&mut self, _world: &mut World, _dt: f64) -> SimulationResult<()> { Ok(()) }
        fn get_params(&self) -> serde_json::Value { serde_json::json!({}) }
        fn set_params(&mut self, _params: &serde_json::Value) -> SimulationResult<()> { Ok(()) }
    }

    #[test]
    fn test_simulation_manager_new() {
        let config = SimulationConfig::default();
        let sim = SimulationManager::new(config);
        assert_eq!(sim.current_step(), 0);
        assert_eq!(sim.current_time(), 0.0);
    }

    #[test]
    fn test_register_module() {
        let config = SimulationConfig::default();
        let mut sim = SimulationManager::new(config);
        
        let result = sim.register_module(Box::new(TestModule));
        assert!(result.is_ok());
        
        // Попытка зарегистрировать тот же модуль должна вернуть ошибку
        let result2 = sim.register_module(Box::new(TestModule));
        assert!(result2.is_err());
    }

    #[test]
    fn test_step_increment() {
        let config = SimulationConfig {
            max_steps: 10,
            dt: 0.5,
            ..Default::default()
        };
        
        let mut sim = SimulationManager::new(config);
        
        for i in 0..5 {
            sim.step().unwrap();
            assert_eq!(sim.current_step(), i + 1);
            assert_eq!(sim.current_time(), (i + 1) as f64 * 0.5);
        }
    }
}
