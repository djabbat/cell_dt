use cell_dt_core::{
    SimulationModule, SimulationResult,
    components::*,
    hecs::{World},
};
use serde_json::{json, Value};
use log::{info, debug};

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
    
    fn update_centriole(&self, centriole: &mut Centriole, dt: f32) {
        centriole.ptm_signature.acetylation_level += self.params.acetylation_rate * dt;
        centriole.ptm_signature.oxidation_level += self.params.oxidation_rate * dt;
        centriole.maturity += 0.01 * dt;
    }
    
    fn update_centriole_pair(&self, pair: &mut CentriolePair, dt: f32) {
        self.update_centriole(&mut pair.mother, dt);
        self.update_centriole(&mut pair.daughter, dt);
        pair.mtoc_activity = pair.mother.maturity * (1.0 - pair.mother.ptm_signature.oxidation_level * 0.5);
    }
}

impl SimulationModule for CentrioleModule {
    fn name(&self) -> &str {
        "centriole_module"
    }
    
    fn step(&mut self, world: &mut World, dt: f64) -> SimulationResult<()> {
        self.step_count += 1;
        let dt_f32 = dt as f32;
        
        let mut query = world.query::<&mut CentriolePair>();
        
        for (_, pair) in query.iter() {
            self.update_centriole_pair(pair, dt_f32);
        }
        
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
    
    fn initialize(&mut self, world: &mut World) -> SimulationResult<()> {
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

    #[test]
    fn test_centriole_params_default() {
        let params = CentrioleParams::default();
        assert_eq!(params.acetylation_rate, 0.02);
        assert_eq!(params.oxidation_rate, 0.01);
        assert!(params.parallel_cells);
    }

    #[test]
    fn test_update_centriole() {
        let module = CentrioleModule::new();
        let mut centriole = Centriole::new(0.5);
        let initial_acetylation = centriole.ptm_signature.acetylation_level;
        let initial_oxidation = centriole.ptm_signature.oxidation_level;
        
        module.update_centriole(&mut centriole, 1.0);
        
        assert!(centriole.ptm_signature.acetylation_level > initial_acetylation);
        assert!(centriole.ptm_signature.oxidation_level > initial_oxidation);
        assert!(centriole.maturity > 0.5);
    }

    #[test]
    fn test_module_name() {
        let module = CentrioleModule::new();
        assert_eq!(module.name(), "centriole_module");
    }
}
