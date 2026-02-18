//! Модуль транскриптома (в разработке)

use cell_dt_core::{
    SimulationModule, SimulationResult,
    hecs::World,
};
use serde_json::{json, Value};
use log::info;

pub struct TranscriptomeModule {
    step_count: u64,
}

impl TranscriptomeModule {
    pub fn new() -> Self {
        Self { step_count: 0 }
    }
}

impl SimulationModule for TranscriptomeModule {
    fn name(&self) -> &str {
        "transcriptome_module"
    }
    
    fn step(&mut self, _world: &mut World, _dt: f64) -> SimulationResult<()> {
        self.step_count += 1;
        Ok(())
    }
    
    fn get_params(&self) -> Value {
        json!({ "status": "under_construction" })
    }
    
    fn set_params(&mut self, _params: &Value) -> SimulationResult<()> {
        Ok(())
    }
    
    fn initialize(&mut self, _world: &mut World) -> SimulationResult<()> {
        info!("Initializing transcriptome module (placeholder)");
        Ok(())
    }
}

impl Default for TranscriptomeModule {
    fn default() -> Self {
        Self::new()
    }
}
