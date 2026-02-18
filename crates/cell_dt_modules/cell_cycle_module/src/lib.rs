//! Модуль клеточного цикла с реальной биологией
//! Версия 2.0 - с очень мягкими контрольными точками

use cell_dt_core::{
    SimulationModule, SimulationResult,
    components::{
        CentriolePair, CellCycleState, CellCycleStateExtended,
        Phase, CyclinType, CdkType, Checkpoint,
    },
    hecs::{World},
};
use serde_json::{json, Value};
use log::{info, debug};
use rand::Rng;

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
            nutrient_availability: 0.95,
            growth_factor_level: 0.9,
            random_variation: 0.3,
        }
    }
}

/// Трейт для расширения функциональности клеточного цикла
pub trait CellCycleExt {
    fn update_phase_with_params(&mut self, dt: f32, params: &CellCycleParams);
    fn check_checkpoints_with_params(&mut self, params: &CellCycleParams) -> Option<Checkpoint>;
    fn should_pass_checkpoint(&self, checkpoint: Checkpoint, params: &CellCycleParams) -> bool;
}

impl CellCycleExt for CellCycleStateExtended {
    fn update_phase_with_params(&mut self, dt: f32, params: &CellCycleParams) {
        self.time_in_current_phase += dt;
        self.total_time += dt;
        
        // Проверяем контрольные точки
        if let Some(checkpoint) = self.check_checkpoints_with_params(params) {
            self.current_checkpoint = Some(checkpoint);
            
            // Иногда все же пропускаем через контрольную точку со случайной вероятностью
            let mut rng = rand::thread_rng();
            if rng.gen::<f32>() < params.random_variation * dt {
                debug!("Cell bypassed checkpoint due to random variation");
                self.current_checkpoint = None;
                for cp in &mut self.checkpoints {
                    cp.satisfied = true;
                }
            }
            return;
        } else {
            self.current_checkpoint = None;
        }
        
        // Длительность фаз с учетом случайности
        let mut rng = rand::thread_rng();
        let phase_duration = match self.phase {
            Phase::G1 => 5.0 * (1.0 + rng.gen::<f32>() * params.random_variation),
            Phase::S => 4.0 * (1.0 + rng.gen::<f32>() * params.random_variation * 0.5),
            Phase::G2 => 2.0 * (1.0 + rng.gen::<f32>() * params.random_variation * 0.3),
            Phase::M => 0.5 * (1.0 + rng.gen::<f32>() * params.random_variation),
        };
        
        self.progress += dt / phase_duration;
        
        if self.progress >= 1.0 {
            self.progress = 0.0;
            self.time_in_current_phase = 0.0;
            
            match self.phase {
                Phase::G1 => {
                    self.phase = Phase::S;
                    debug!("Cell entered S phase");
                }
                Phase::S => {
                    self.phase = Phase::G2;
                    debug!("Cell entered G2 phase");
                }
                Phase::G2 => {
                    self.phase = Phase::M;
                    debug!("Cell entered M phase");
                }
                Phase::M => {
                    self.phase = Phase::G1;
                    self.cycle_count += 1;
                    info!("Cell completed cycle {}!", self.cycle_count);
                }
            }
        }
    }
    
    fn should_pass_checkpoint(&self, checkpoint: Checkpoint, params: &CellCycleParams) -> bool {
        let mut rng = rand::thread_rng();
        
        // Базовая вероятность прохождения
        let base_probability = match checkpoint {
            Checkpoint::G1SRestriction => {
                let cyclin_d = self.get_complex_activity(CyclinType::CyclinD, CdkType::Cdk4);
                let cyclin_e = self.get_complex_activity(CyclinType::CyclinE, CdkType::Cdk2);
                (cyclin_d + cyclin_e) / 2.0
            }
            Checkpoint::G2MCheckpoint => {
                let cyclin_b = self.get_complex_activity(CyclinType::CyclinB, CdkType::Cdk1);
                cyclin_b
            }
            Checkpoint::SpindleAssembly => self.centriole_influence,
            Checkpoint::DNARepair => 1.0 - self.growth_factors.dna_damage,
        };
        
        // Добавляем случайность и влияние параметров
        let probability = base_probability * (1.0 - params.checkpoint_strictness) 
                         + rng.gen::<f32>() * params.random_variation;
        
        probability > 0.3
    }
    
    fn check_checkpoints_with_params(&mut self, params: &CellCycleParams) -> Option<Checkpoint> {
        // Сначала вычисляем все необходимые значения для контрольных точек
        let mut checkpoint_results = Vec::new();
        
        for checkpoint in &self.checkpoints {
            if !checkpoint.satisfied {
                let should_pass = self.should_pass_checkpoint(checkpoint.checkpoint, params);
                checkpoint_results.push((checkpoint.checkpoint, should_pass));
            } else {
                checkpoint_results.push((checkpoint.checkpoint, true));
            }
        }
        
        // Теперь обновляем состояние контрольных точек
        for (i, (checkpoint_type, should_pass)) in checkpoint_results.iter().enumerate() {
            let checkpoint = &mut self.checkpoints[i];
            if !should_pass {
                checkpoint.time_in_checkpoint += 0.1;
                return Some(*checkpoint_type);
            } else {
                checkpoint.satisfied = true;
                checkpoint.time_in_checkpoint = 0.0;
            }
        }
        
        None
    }
}

/// Модуль клеточного цикла
pub struct CellCycleModule {
    params: CellCycleParams,
    step_count: u64,
    cells_arrested: usize,
    cells_divided: usize,
    cells_apoptotic: usize,
    cells_passed_checkpoint: usize,
    total_divisions: u64,
}

impl CellCycleModule {
    pub fn new() -> Self {
        Self {
            params: CellCycleParams::default(),
            step_count: 0,
            cells_arrested: 0,
            cells_divided: 0,
            cells_apoptotic: 0,
            cells_passed_checkpoint: 0,
            total_divisions: 0,
        }
    }
    
    pub fn with_params(params: CellCycleParams) -> Self {
        Self {
            params,
            step_count: 0,
            cells_arrested: 0,
            cells_divided: 0,
            cells_apoptotic: 0,
            cells_passed_checkpoint: 0,
            total_divisions: 0,
        }
    }
    
    fn update_cell_cycle(&mut self, cell_cycle: &mut CellCycleStateExtended, centriole_pair: Option<&CentriolePair>, dt: f32) {
        // Применяем влияние центриоли
        if let Some(centriole) = centriole_pair {
            cell_cycle.apply_centriole_influence(centriole);
        }
        
        // Обновляем факторы роста и стресса
        cell_cycle.growth_factors.growth_signal = self.params.growth_factor_level;
        cell_cycle.growth_factors.nutrient_level = self.params.nutrient_availability;
        
        // Естественная флуктуация
        let mut rng = rand::thread_rng();
        cell_cycle.growth_factors.stress_level = (cell_cycle.growth_factors.stress_level 
            + (rng.gen::<f32>() - 0.5) * 0.1 * dt).clamp(0.0, 0.3);
        
        // Обновляем циклины
        cell_cycle.update_cyclins(dt);
        
        // Обновляем фазу
        let old_phase = cell_cycle.phase;
        cell_cycle.update_phase_with_params(dt, &self.params);
        
        // Считаем статистику
        if cell_cycle.current_checkpoint.is_some() {
            self.cells_arrested += 1;
        } else {
            self.cells_passed_checkpoint += 1;
        }
        
        // Если клетка поделилась
        if old_phase == Phase::M && cell_cycle.phase == Phase::G1 {
            self.cells_divided += 1;
            self.total_divisions += 1;
            info!("🎉 Cell divided! Total divisions: {}", self.total_divisions);
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
        
        // Сбрасываем счетчики
        self.cells_arrested = 0;
        self.cells_divided = 0;
        self.cells_apoptotic = 0;
        self.cells_passed_checkpoint = 0;
        
        let mut query = world.query::<(&mut CellCycleStateExtended, Option<&CentriolePair>)>();
        
        for (_, (cell_cycle, centriole_opt)) in query.iter() {
            self.update_cell_cycle(cell_cycle, centriole_opt, dt_f32);
        }
        
        // Логируем статистику
        if self.step_count % 50 == 0 {
            info!("Cell cycle stats: arrested={}, divided={}, passed={}, total_divisions={}", 
                  self.cells_arrested, self.cells_divided, self.cells_passed_checkpoint, self.total_divisions);
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
            "cells_apoptotic": self.cells_apoptotic,
            "cells_passed_checkpoint": self.cells_passed_checkpoint,
            "total_divisions": self.total_divisions,
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
        info!("Initializing cell cycle module v2.0 with very soft checkpoints");
        
        let entities_to_update: Vec<_> = world.query::<&CellCycleState>()
            .iter()
            .map(|(entity, state)| {
                let state_clone = state.clone();
                (entity, state_clone)
            })
            .collect();
        
        let entity_count = entities_to_update.len();
        
        for (entity, old_state) in &entities_to_update {
            let mut new_state = CellCycleStateExtended::new();
            new_state.phase = old_state.phase;
            new_state.progress = old_state.progress;
            
            let mut rng = rand::thread_rng();
            new_state.growth_factors.growth_signal = self.params.growth_factor_level;
            new_state.growth_factors.nutrient_level = self.params.nutrient_availability;
            new_state.growth_factors.stress_level = rng.gen::<f32>() * 0.1;
            new_state.growth_factors.dna_damage = rng.gen::<f32>() * 0.05;
            
            let _ = world.remove_one::<CellCycleState>(*entity);
            let _ = world.insert_one(*entity, new_state);
        }
        
        info!("Initialized {} cells with cell cycle states", entity_count);
        
        Ok(())
    }
}

impl Default for CellCycleModule {
    fn default() -> Self {
        Self::new()
    }
}
