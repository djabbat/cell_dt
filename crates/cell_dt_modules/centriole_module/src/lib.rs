use cell_dt_core::{
    SimulationModule, SimulationResult,
    components::*,
    hecs::World,
};
use serde_json::{json, Value};
use log::{info, debug};
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct CentrioleParams {
    pub acetylation_rate: f32,
    pub oxidation_rate: f32,
    pub mtoc_activity_threshold: f32,
    pub cafd_recruitment_probability: f32,
    pub age_effect_factor: f32,
    pub parallel_cells: bool,  // Новый флаг для параллельной обработки клеток
}

impl Default for CentrioleParams {
    fn default() -> Self {
        Self {
            acetylation_rate: 0.01,
            oxidation_rate: 0.005,
            mtoc_activity_threshold: 0.3,
            cafd_recruitment_probability: 0.1,
            age_effect_factor: 0.2,
            parallel_cells: true,  // По умолчанию включено
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
    
    pub fn with_parallel(parallel_cells: bool) -> Self {
        let mut params = CentrioleParams::default();
        params.parallel_cells = parallel_cells;
        Self {
            params,
            step_count: 0,
        }
    }
    
    fn update_centriole(&self, centriole: &mut Centriole, dt: f64) {
        let dt_f32 = dt as f32;
        
        // Обновление ПТМ
        centriole.ptm_signature.acetylation_level += self.params.acetylation_rate * dt_f32;
        centriole.ptm_signature.oxidation_level += self.params.oxidation_rate * dt_f32;
        
        // Ограничиваем значения
        centriole.ptm_signature.acetylation_level = centriole.ptm_signature.acetylation_level.clamp(0.0, 1.0);
        centriole.ptm_signature.oxidation_level = centriole.ptm_signature.oxidation_level.clamp(0.0, 1.0);
        
        // Увеличение зрелости
        centriole.maturity += 0.01 * dt_f32;
        centriole.maturity = centriole.maturity.clamp(0.0, 1.0);
        
        // Обновление CAFD факторов
        for cafd in centriole.associated_cafds.iter_mut() {
            cafd.activity *= 0.99;  // Естественная деградация
            cafd.concentration *= 0.98;
        }
        
        // Удаление неактивных CAFD
        centriole.associated_cafds.retain(|cafd| cafd.concentration > 0.01);
    }
    
    fn update_centriole_pair(&self, pair: &mut CentriolePair, dt: f64) {
        self.update_centriole(&mut pair.mother, dt);
        self.update_centriole(&mut pair.daughter, dt);
        
        // Активность MTOC зависит от зрелости материнской центриоли и окислительного стресса
        pair.mtoc_activity = pair.mother.maturity * 
                             (1.0 - pair.mother.ptm_signature.oxidation_level * 0.5);
        
        pair.mtoc_activity = pair.mtoc_activity.clamp(0.0, 1.0);
        
        // Случайное образование цилий
        if !pair.cilium_present && pair.mother.maturity > 0.8 {
            if rand::random::<f32>() < 0.001 * dt as f32 {
                pair.cilium_present = true;
                debug!("Cilium formed!");
            }
        }
    }
}

impl SimulationModule for CentrioleModule {
    fn name(&self) -> &str {
        "centriole_module"
    }
    
    fn step(&mut self, world: &mut World, dt: f64) -> SimulationResult<()> {
        self.step_count += 1;
        debug!("Centriole module step {} with dt={}, parallel_cells={}", 
               self.step_count, dt, self.params.parallel_cells);
        
        // Получаем все пары центриолей
        let mut query = world.query::<&mut CentriolePair>();
        let pairs: Vec<_> = query.iter().map(|(_, pair)| pair).collect();
        
        if self.params.parallel_cells && pairs.len() > 100 {
            // Параллельная обработка для большого количества клеток
            debug!("Processing {} cells in parallel", pairs.len());
            pairs.into_par_iter().for_each(|pair| {
                self.update_centriole_pair(pair, dt);
            });
        } else {
            // Последовательная обработка для малого количества клеток
            debug!("Processing {} cells sequentially", pairs.len());
            for pair in pairs {
                self.update_centriole_pair(pair, dt);
            }
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
        if let Some(threshold) = params.get("mtoc_activity_threshold").and_then(|v| v.as_f64()) {
            self.params.mtoc_activity_threshold = threshold as f32;
        }
        if let Some(prob) = params.get("cafd_recruitment_probability").and_then(|v| v.as_f64()) {
            self.params.cafd_recruitment_probability = prob as f32;
        }
        if let Some(factor) = params.get("age_effect_factor").and_then(|v| v.as_f64()) {
            self.params.age_effect_factor = factor as f32;
        }
        if let Some(parallel) = params.get("parallel_cells").and_then(|v| v.as_bool()) {
            self.params.parallel_cells = parallel;
        }
        
        Ok(())
    }
    
    fn initialize(&mut self, world: &mut World) -> SimulationResult<()> {
        info!("Initializing centriole module with parallel_cells={}", self.params.parallel_cells);
        
        // Добавляем случайные CAFD для некоторых клеток при инициализации
        let mut query = world.query::<&mut CentriolePair>();
        for (_, pair) in query.iter() {
            if rand::random::<f32>() < 0.3 {  // 30% клеток имеют CAFD
                let mut cafd = CAFD::new("YAP");
                cafd.activity = rand::random::<f32>();
                cafd.concentration = rand::random::<f32>() * 0.5;
                pair.mother.associated_cafds.push(cafd);
            }
        }
        
        Ok(())
    }
}

impl Default for CentrioleModule {
    fn default() -> Self {
        Self::new()
    }
}
