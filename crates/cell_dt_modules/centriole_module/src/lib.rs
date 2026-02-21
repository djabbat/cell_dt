use cell_dt_core::{
    SimulationModule, SimulationResult,
    hecs::World,
};
use serde_json::{json, Value};
use log::info;

#[derive(Debug, Clone)]
pub struct CentrioleParams {
    pub acetylation_rate: f32,
    pub oxidation_rate: f32,
    pub parallel_cells: bool,
}

impl Default for CentrioleParams {
    fn default() -> Self {
        Self {
            acetylation_rate: 0.02,
            oxidation_rate: 0.01,
            parallel_cells: true,
        }
    }
}

pub struct CentrioleModule {
    params: CentrioleParams,
    step_count: u64,
}

impl CentrioleModule {
    pub fn new() -> Self {
        Self {
            params: CentrioleParams::default(),
            step_count: 0,
        }
    }
    
    pub fn with_parallel(parallel_cells: bool) -> Self {
        let mut params = CentrioleParams::default();
        params.parallel_cells = parallel_cells;
        Self {
            params,
            step_count: 0,
        }
    }
}

impl SimulationModule for CentrioleModule {
    fn name(&self) -> &str {
        "centriole_module"
    }
    
    fn step(&mut self, _world: &mut World, _dt: f64) -> SimulationResult<()> {
        self.step_count += 1;
        Ok(())
    }
    
    fn get_params(&self) -> Value {
        json!({
            "acetylation_rate": self.params.acetylation_rate,
            "oxidation_rate": self.params.oxidation_rate,
            "parallel_cells": self.params.parallel_cells,
        })
    }
    
    fn set_params(&mut self, params: &Value) -> SimulationResult<()> {
        if let Some(rate) = params.get("acetylation_rate").and_then(|v| v.as_f64()) {
            self.params.acetylation_rate = rate as f32;
        }
        if let Some(rate) = params.get("oxidation_rate").and_then(|v| v.as_f64()) {
            self.params.oxidation_rate = rate as f32;
        }
        if let Some(parallel) = params.get("parallel_cells").and_then(|v| v.as_bool()) {
            self.params.parallel_cells = parallel;
        }
        Ok(())
    }
    
    fn initialize(&mut self, _world: &mut World) -> SimulationResult<()> {
        info!("Initializing centriole module");
        Ok(())
    }
}

impl Default for CentrioleModule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cell_dt_core::hecs::World;

    #[test]
    fn test_default_params() {
        let module = CentrioleModule::new();
        assert_eq!(module.params.acetylation_rate, 0.02);
        assert_eq!(module.params.oxidation_rate, 0.01);
        assert!(module.params.parallel_cells);
    }

    #[test]
    fn test_with_parallel_false() {
        let module = CentrioleModule::with_parallel(false);
        assert!(!module.params.parallel_cells);
    }

    #[test]
    fn test_with_parallel_true() {
        let module = CentrioleModule::with_parallel(true);
        assert!(module.params.parallel_cells);
    }

    #[test]
    fn test_step_increments_counter() {
        let mut module = CentrioleModule::new();
        let mut world = World::new();
        assert_eq!(module.step_count, 0);
        module.step(&mut world, 0.1).unwrap();
        assert_eq!(module.step_count, 1);
        module.step(&mut world, 0.1).unwrap();
        assert_eq!(module.step_count, 2);
    }

    #[test]
    fn test_name() {
        let module = CentrioleModule::new();
        assert_eq!(module.name(), "centriole_module");
    }

    #[test]
    fn test_set_params_updates_rates() {
        let mut module = CentrioleModule::new();
        let params = serde_json::json!({
            "acetylation_rate": 0.05,
            "oxidation_rate": 0.03,
            "parallel_cells": false,
        });
        module.set_params(&params).unwrap();
        assert_eq!(module.params.acetylation_rate, 0.05);
        assert_eq!(module.params.oxidation_rate, 0.03);
        assert!(!module.params.parallel_cells);
    }

    #[test]
    fn test_set_params_ignores_unknown_keys() {
        let mut module = CentrioleModule::new();
        let params = serde_json::json!({ "unknown_key": 99 });
        assert!(module.set_params(&params).is_ok());
        // Original values unchanged
        assert_eq!(module.params.acetylation_rate, 0.02);
    }

    #[test]
    fn test_get_params_has_all_keys() {
        let module = CentrioleModule::new();
        let params = module.get_params();
        assert!(params.get("acetylation_rate").is_some());
        assert!(params.get("oxidation_rate").is_some());
        assert!(params.get("parallel_cells").is_some());
    }

    #[test]
    fn test_get_params_roundtrip() {
        let mut module = CentrioleModule::new();
        let original = module.get_params();
        module.set_params(&original).unwrap();
        let after = module.get_params();
        assert_eq!(original["acetylation_rate"], after["acetylation_rate"]);
    }
}
