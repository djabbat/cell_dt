//! Модуль клеточного цикла (упрощенная версия)

use cell_dt_core::{
    SimulationModule, SimulationResult,
    components::{
        CentriolePair, CellCycleState, CellCycleStateExtended,
        Phase,
    },
    hecs::{World},
};
use serde_json::{json, Value};
use log::{info, debug};

/// Параметры модуля клеточного цикла
#[derive(Debug, Clone)]
pub struct CellCycleParams {
    pub base_cycle_time: f32,
    pub growth_factor_sensitivity: f32,
    pub stress_sensitivity: f32,
    pub checkpoint_strictness: f32,
    pub enable_apoptosis: bool,
    pub nutrient_availability: f32,
    pub growth_factor_level: f32,
    pub random_variation: f32,
}

impl Default for CellCycleParams {
    fn default() -> Self {
        Self {
            base_cycle_time: 24.0,
            growth_factor_sensitivity: 0.3,
            stress_sensitivity: 0.2,
            checkpoint_strictness: 0.1,
            enable_apoptosis: true,
            nutrient_availability: 0.9,
            growth_factor_level: 0.8,
            random_variation: 0.2,
        }
    }
}

/// Модуль клеточного цикла
pub struct CellCycleModule {
    params: CellCycleParams,
    step_count: u64,
    cells_arrested: usize,
    cells_divided: usize,
}

impl CellCycleModule {
    pub fn new() -> Self {
        Self {
            params: CellCycleParams::default(),
            step_count: 0,
            cells_arrested: 0,
            cells_divided: 0,
        }
    }
    
    pub fn with_params(params: CellCycleParams) -> Self {
        Self {
            params,
            step_count: 0,
            cells_arrested: 0,
            cells_divided: 0,
        }
    }
    
    fn update_cell_cycle(&self, cell_cycle: &mut CellCycleStateExtended, _centriole: Option<&CentriolePair>, dt: f32) {
        cell_cycle.time_in_current_phase += dt;
        
        let phase_duration = match cell_cycle.phase {
            Phase::G1 => 10.0,
            Phase::S => 8.0,
            Phase::G2 => 4.0,
            Phase::M => 1.0,
        };
        
        cell_cycle.progress += dt / phase_duration;
        
        if cell_cycle.progress >= 1.0 {
            cell_cycle.progress = 0.0;
            cell_cycle.time_in_current_phase = 0.0;
            
            match cell_cycle.phase {
                Phase::G1 => {
                    cell_cycle.phase = Phase::S;
                }
                Phase::S => {
                    cell_cycle.phase = Phase::G2;
                }
                Phase::G2 => {
                    cell_cycle.phase = Phase::M;
                }
                Phase::M => {
                    cell_cycle.phase = Phase::G1;
                    cell_cycle.cycle_count += 1;
                }
            }
        }
    }
}

impl SimulationModule for CellCycleModule {
    fn name(&self) -> &str {
        "cell_cycle_module"
    }
    
    fn step(&mut self, world: &mut World, dt: f64) -> SimulationResult<()> {
        self.step_count += 1;
        let dt_f32 = dt as f32;
        
        debug!("Cell cycle module step {}", self.step_count);
        
        self.cells_arrested = 0;
        self.cells_divided = 0;
        
        let mut query = world.query::<(&mut CellCycleStateExtended, Option<&CentriolePair>)>();
        
        for (_, (cell_cycle, centriole_opt)) in query.iter() {
            self.update_cell_cycle(cell_cycle, centriole_opt, dt_f32);
        }
        
        Ok(())
    }
    
    fn get_params(&self) -> Value {
        json!({
            "base_cycle_time": self.params.base_cycle_time,
            "growth_factor_sensitivity": self.params.growth_factor_sensitivity,
            "stress_sensitivity": self.params.stress_sensitivity,
            "checkpoint_strictness": self.params.checkpoint_strictness,
            "enable_apoptosis": self.params.enable_apoptosis,
            "nutrient_availability": self.params.nutrient_availability,
            "growth_factor_level": self.params.growth_factor_level,
            "random_variation": self.params.random_variation,
            "step_count": self.step_count,
            "cells_arrested": self.cells_arrested,
            "cells_divided": self.cells_divided,
        })
    }
    
    fn set_params(&mut self, params: &Value) -> SimulationResult<()> {
        if let Some(time) = params.get("base_cycle_time").and_then(|v| v.as_f64()) {
            self.params.base_cycle_time = time as f32;
        }
        if let Some(sens) = params.get("growth_factor_sensitivity").and_then(|v| v.as_f64()) {
            self.params.growth_factor_sensitivity = sens as f32;
        }
        if let Some(sens) = params.get("stress_sensitivity").and_then(|v| v.as_f64()) {
            self.params.stress_sensitivity = sens as f32;
        }
        if let Some(strict) = params.get("checkpoint_strictness").and_then(|v| v.as_f64()) {
            self.params.checkpoint_strictness = strict as f32;
        }
        if let Some(apoptosis) = params.get("enable_apoptosis").and_then(|v| v.as_bool()) {
            self.params.enable_apoptosis = apoptosis;
        }
        if let Some(nutrient) = params.get("nutrient_availability").and_then(|v| v.as_f64()) {
            self.params.nutrient_availability = nutrient as f32;
        }
        if let Some(growth) = params.get("growth_factor_level").and_then(|v| v.as_f64()) {
            self.params.growth_factor_level = growth as f32;
        }
        if let Some(random) = params.get("random_variation").and_then(|v| v.as_f64()) {
            self.params.random_variation = random as f32;
        }
        
        Ok(())
    }
    
    fn initialize(&mut self, world: &mut World) -> SimulationResult<()> {
        info!("Initializing cell cycle module");
        
        // Сначала собираем все сущности и их состояния
        let states: Vec<_> = world.query::<&CellCycleState>()
            .iter()
            .map(|(entity, state)| {
                (entity, state.clone())
            })
            .collect();
        
        // Потом обновляем
        for (entity, old_state) in states {
            let mut new_state = CellCycleStateExtended::new();
            new_state.phase = old_state.phase;
            new_state.progress = old_state.progress;
            
            let _ = world.remove_one::<CellCycleState>(entity);
            let _ = world.insert_one(entity, new_state);
        }
        
        info!("Initialized cell cycle module");
        Ok(())
    }
}

impl Default for CellCycleModule {
    fn default() -> Self {
        Self::new()
    }
}
