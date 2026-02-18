use crate::{SimulationResult, hecs::World};
use serde_json::Value;

pub trait SimulationModule: Send + Sync {
    fn name(&self) -> &str;
    fn step(&mut self, world: &mut World, dt: f64) -> SimulationResult<()>;
    fn get_params(&self) -> Value;
    fn set_params(&mut self, params: &Value) -> SimulationResult<()>;
    
    fn initialize(&mut self, _world: &mut World) -> SimulationResult<()> {
        Ok(())
    }
    
    fn cleanup(&mut self) -> SimulationResult<()> {
        Ok(())
    }
}
