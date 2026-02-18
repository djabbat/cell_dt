//! Модуль иерархии стволовых клеток (упрощенная версия)

use cell_dt_core::{
    SimulationModule, SimulationResult,
    components::*,
    hecs::{World},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use log::{info, debug};

/// Уровни потенции клеток
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PotencyLevel {
    Totipotent,
    Pluripotent,
    Multipotent,
    Oligopotent,
    Unipotent,
    Differentiated,
}

/// Линии дифференцировки
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CellLineage {
    EmbryonicStem,
    HematopoieticStem,
    NeuralStem,
}

/// Состояние клетки в иерархии
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StemCellHierarchyState {
    pub potency_level: PotencyLevel,
    pub potency_score: f32,
    pub lineage: Option<CellLineage>,
    pub master_regulator_levels: std::collections::HashMap<String, f32>,
}

impl StemCellHierarchyState {
    pub fn new() -> Self {
        let mut master_regs = std::collections::HashMap::new();
        master_regs.insert("OCT4".to_string(), 0.9);
        master_regs.insert("NANOG".to_string(), 0.9);
        master_regs.insert("SOX2".to_string(), 0.9);
        
        Self {
            potency_level: PotencyLevel::Pluripotent,
            potency_score: 0.9,
            lineage: None,
            master_regulator_levels: master_regs,
        }
    }
    
    pub fn set_potency(&mut self, level: PotencyLevel) {
        self.potency_level = level;
        self.potency_score = match level {
            PotencyLevel::Totipotent => 1.0,
            PotencyLevel::Pluripotent => 0.8,
            PotencyLevel::Multipotent => 0.6,
            PotencyLevel::Oligopotent => 0.4,
            PotencyLevel::Unipotent => 0.2,
            PotencyLevel::Differentiated => 0.1,
        };
    }
}

impl Default for StemCellHierarchyState {
    fn default() -> Self {
        Self::new()
    }
}

/// Параметры модуля иерархии
#[derive(Debug, Clone)]
pub struct StemCellHierarchyParams {
    pub initial_potency: PotencyLevel,
    pub enable_plasticity: bool,
    pub plasticity_rate: f32,
    pub differentiation_threshold: f32,
}

impl Default for StemCellHierarchyParams {
    fn default() -> Self {
        Self {
            initial_potency: PotencyLevel::Pluripotent,
            enable_plasticity: true,
            plasticity_rate: 0.01,
            differentiation_threshold: 0.7,
        }
    }
}

/// Модуль иерархии стволовых клеток
pub struct StemCellHierarchyModule {
    params: StemCellHierarchyParams,
    step_count: u64,
}

impl StemCellHierarchyModule {
    pub fn new() -> Self {
        Self {
            params: StemCellHierarchyParams::default(),
            step_count: 0,
        }
    }
    
    pub fn with_params(params: StemCellHierarchyParams) -> Self {
        Self {
            params,
            step_count: 0,
        }
    }
}

impl SimulationModule for StemCellHierarchyModule {
    fn name(&self) -> &str {
        "stem_cell_hierarchy_module"
    }
    
    fn step(&mut self, _world: &mut World, _dt: f64) -> SimulationResult<()> {
        self.step_count += 1;
        debug!("Stem cell hierarchy module step {}", self.step_count);
        Ok(())
    }
    
    fn get_params(&self) -> Value {
        json!({
            "initial_potency": format!("{:?}", self.params.initial_potency),
            "enable_plasticity": self.params.enable_plasticity,
            "plasticity_rate": self.params.plasticity_rate,
            "differentiation_threshold": self.params.differentiation_threshold,
            "step_count": self.step_count,
        })
    }
    
    fn set_params(&mut self, params: &Value) -> SimulationResult<()> {
        if let Some(p) = params.get("enable_plasticity").and_then(|v| v.as_bool()) {
            self.params.enable_plasticity = p;
        }
        if let Some(p) = params.get("plasticity_rate").and_then(|v| v.as_f64()) {
            self.params.plasticity_rate = p as f32;
        }
        Ok(())
    }
    
    fn initialize(&mut self, world: &mut World) -> SimulationResult<()> {
        info!("Initializing stem cell hierarchy module");
        
        let entities: Vec<_> = world.query::<&CellCycleStateExtended>()
            .iter()
            .map(|(e, _)| e)
            .collect();
        
        let entity_count = entities.len();
        
        for &entity in &entities {
            if !world.contains(entity) {
                continue;
            }
            let hierarchy = StemCellHierarchyState::new();
            world.insert_one(entity, hierarchy)?;
        }
        
        info!("Initialized hierarchy for {} cells", entity_count);
        Ok(())
    }
}

impl Default for StemCellHierarchyModule {
    fn default() -> Self {
        Self::new()
    }
}

/// Фабрики для создания различных типов стволовых клеток
pub mod factories {
    use super::*;

    pub fn create_embryonic_stem_cell() -> StemCellHierarchyState {
        let mut state = StemCellHierarchyState::new();
        state.set_potency(PotencyLevel::Pluripotent);
        state.master_regulator_levels.insert("OCT4".to_string(), 1.0);
        state.master_regulator_levels.insert("NANOG".to_string(), 1.0);
        state.master_regulator_levels.insert("SOX2".to_string(), 1.0);
        state
    }

    pub fn create_hematopoietic_stem_cell() -> StemCellHierarchyState {
        let mut state = StemCellHierarchyState::new();
        state.set_potency(PotencyLevel::Multipotent);
        state.lineage = Some(CellLineage::HematopoieticStem);
        state
    }

    pub fn create_neural_stem_cell() -> StemCellHierarchyState {
        let mut state = StemCellHierarchyState::new();
        state.set_potency(PotencyLevel::Multipotent);
        state.lineage = Some(CellLineage::NeuralStem);
        state
    }
}
