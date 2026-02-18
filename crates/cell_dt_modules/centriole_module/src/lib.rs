use cell_dt_core::{
    SimulationModule, SimulationResult,
    components::*,
    hecs::World,
};
use serde_json::{json, Value};
use log::{info, debug};

#[derive(Debug, Clone)]
pub struct CentrioleParams {
    pub acetylation_rate: f32,
    pub oxidation_rate: f32,
    pub mtoc_activity_threshold: f32,
    pub cafd_recruitment_probability: f32,
    pub age_effect_factor: f32,
}

impl Default for CentrioleParams {
    fn default() -> Self {
        Self {
            acetylation_rate: 0.01,
            oxidation_rate: 0.005,
            mtoc_activity_threshold: 0.3,
            cafd_recruitment_probability: 0.1,
            age_effect_factor: 0.2,
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
    
    pub fn with_params(params: CentrioleParams) -> Self {
        Self {
            params,
            step_count: 0,
        }
    }
    
    fn update_centriole(&self, centriole: &mut Centriole, dt: f64) {
        let dt_f32 = dt as f32;
        
        centriole.ptm_signature.acetylation_level += self.params.acetylation_rate * dt_f32;
        centriole.ptm_signature.oxidation_level += self.params.oxidation_rate * dt_f32;
        
        centriole.ptm_signature.acetylation_level = centriole.ptm_signature.acetylation_level.clamp(0.0, 1.0);
        centriole.ptm_signature.oxidation_level = centriole.ptm_signature.oxidation_level.clamp(0.0, 1.0);
        
        centriole.maturity += 0.01 * dt_f32;
        centriole.maturity = centriole.maturity.clamp(0.0, 1.0);
    }
    
    fn update_centriole_pair(&self, pair: &mut CentriolePair, dt: f64) {
        self.update_centriole(&mut pair.mother, dt);
        self.update_centriole(&mut pair.daughter, dt);
        
        pair.mtoc_activity = pair.mother.maturity * 
                             (1.0 - pair.mother.ptm_signature.oxidation_level * 0.5);
    }
}

impl SimulationModule for CentrioleModule {
    fn name(&self) -> &str {
        "centriole_module"
    }
    
    fn step(&mut self, world: &mut World, dt: f64) -> SimulationResult<()> {
        self.step_count += 1;
        debug!("Centriole module step {} with dt={}", self.step_count, dt);
        
        // Правильный синтаксис для итерации в hecs
        let mut query = world.query::<&mut CentriolePair>();
        for (_, pair) in query.iter() {
            self.update_centriole_pair(pair, dt);
        }
        
        Ok(())
    }
    
    fn get_params(&self) -> Value {
        json!({
            "acetylation_rate": self.params.acetylation_rate,
            "oxidation_rate": self.params.oxidation_rate,
            "mtoc_activity_threshold": self.params.mtoc_activity_threshold,
            "cafd_recruitment_probability": self.params.cafd_recruitment_probability,
            "age_effect_factor": self.params.age_effect_factor,
        })
    }
    
    fn set_params(&mut self, params: &Value) -> SimulationResult<()> {
        if let Some(rate) = params.get("acetylation_rate").and_then(|v| v.as_f64()) {
            self.params.acetylation_rate = rate as f32;
        }
        if let Some(rate) = params.get("oxidation_rate").and_then(|v| v.as_f64()) {
            self.params.oxidation_rate = rate as f32;
        }
        if let Some(threshold) = params.get("mtoc_activity_threshold").and_then(|v| v.as_f64()) {
            self.params.mtoc_activity_threshold = threshold as f32;
        }
        if let Some(prob) = params.get("cafd_recruitment_probability").and_then(|v| v.as_f64()) {
            self.params.cafd_recruitment_probability = prob as f32;
        }
        if let Some(factor) = params.get("age_effect_factor").and_then(|v| v.as_f64()) {
            self.params.age_effect_factor = factor as f32;
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
