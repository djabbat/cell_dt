//! Модуль асимметричного деления стволовых клеток

use cell_dt_core::{
    SimulationModule, SimulationResult,
    components::*,
    hecs::{World},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use log::{info, debug};
use std::collections::HashMap;

/// Типы деления клеток
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DivisionType {
    Symmetric,
    Asymmetric,
    SelfRenewal,
    Differentiation,
}

/// Компонент для отслеживания асимметричного деления
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsymmetricDivisionComponent {
    pub division_type: DivisionType,
    pub niche_id: Option<u64>,
    pub stemness_potential: f32,
}

impl Default for AsymmetricDivisionComponent {
    fn default() -> Self {
        Self {
            division_type: DivisionType::Symmetric,
            niche_id: None,
            stemness_potential: 0.8,
        }
    }
}

/// Параметры модуля асимметричного деления
#[derive(Debug, Clone)]
pub struct AsymmetricDivisionParams {
    pub asymmetric_division_probability: f32,
    pub symmetric_renewal_probability: f32,
    pub symmetric_diff_probability: f32,
    pub stem_cell_niche_capacity: usize,
    pub max_niches: usize,
}

impl Default for AsymmetricDivisionParams {
    fn default() -> Self {
        Self {
            asymmetric_division_probability: 0.3,
            symmetric_renewal_probability: 0.4,
            symmetric_diff_probability: 0.3,
            stem_cell_niche_capacity: 10,
            max_niches: 100,
        }
    }
}

/// Модуль асимметричного деления
pub struct AsymmetricDivisionModule {
    params: AsymmetricDivisionParams,
    step_count: u64,
    niches: HashMap<u64, (f32, f32, f32, f32)>,
    next_niche_id: u64,
}

impl AsymmetricDivisionModule {
    pub fn new() -> Self {
        Self {
            params: AsymmetricDivisionParams::default(),
            step_count: 0,
            niches: HashMap::new(),
            next_niche_id: 1,
        }
    }
    
    pub fn with_params(params: AsymmetricDivisionParams) -> Self {
        Self {
            params,
            step_count: 0,
            niches: HashMap::new(),
            next_niche_id: 1,
        }
    }
    
    /// Создать новую нишу
    pub fn create_niche(&mut self, x: f32, y: f32, z: f32, radius: f32) -> u64 {
        let niche_id = self.next_niche_id;
        self.niches.insert(niche_id, (x, y, z, radius));
        self.next_niche_id += 1;
        niche_id
    }
}

impl SimulationModule for AsymmetricDivisionModule {
    fn name(&self) -> &str {
        "asymmetric_division_module"
    }
    
    fn step(&mut self, _world: &mut World, _dt: f64) -> SimulationResult<()> {
        self.step_count += 1;
        debug!("Asymmetric division module step {}", self.step_count);
        Ok(())
    }
    
    fn get_params(&self) -> Value {
        json!({
            "asymmetric_division_probability": self.params.asymmetric_division_probability,
            "symmetric_renewal_probability": self.params.symmetric_renewal_probability,
            "symmetric_diff_probability": self.params.symmetric_diff_probability,
            "stem_cell_niche_capacity": self.params.stem_cell_niche_capacity,
            "max_niches": self.params.max_niches,
            "step_count": self.step_count,
            "active_niches": self.niches.len(),
        })
    }
    
    fn set_params(&mut self, params: &Value) -> SimulationResult<()> {
        if let Some(p) = params.get("asymmetric_division_probability").and_then(|v| v.as_f64()) {
            self.params.asymmetric_division_probability = p as f32;
        }
        Ok(())
    }
    
    fn initialize(&mut self, world: &mut World) -> SimulationResult<()> {
        info!("Initializing asymmetric division module");
        
        let entities: Vec<_> = world.query::<&CellCycleStateExtended>()
            .iter()
            .map(|(e, _)| e)
            .collect();
        
        let entity_count = entities.len();
        
        for &entity in &entities {
            if !world.contains(entity) {
                continue;
            }
            let component = AsymmetricDivisionComponent::default();
            world.insert_one(entity, component)?;
        }
        
        // Создаем несколько ниш
        for i in 0..3 {
            self.create_niche(0.0, 0.0, (i * 10) as f32, 5.0);
        }
        
        info!("Initialized {} cells with asymmetric division capability", entity_count);
        info!("Created {} stem cell niches", self.niches.len());
        
        Ok(())
    }
}

impl Default for AsymmetricDivisionModule {
    fn default() -> Self {
        Self::new()
    }
}
