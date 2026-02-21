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
        Self {
            params: CentrioleParams { parallel_cells, ..Default::default() },
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
