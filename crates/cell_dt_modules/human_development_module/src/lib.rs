//! # Human Development Module (CDATA)
//!
//! Моделирует полный жизненный цикл человека — от оплодотворённой яйцеклетки
//! до смерти — на основе гипотезы CDATA (Centriolar Damage Accumulation Theory
//! of Aging, Tkemaladze 2025/2026).
//!
//! ## Ключевые механизмы:
//!
//! 1. **Система индукторов** (S/H-структуры): каждая дифференцирующая митоза
//!    расходует один S-индуктор; при S_count = 0 — терминальная дифференцировка.
//!
//! 2. **Накопление повреждений центриоли**: карбонилирование белков, гиперацетилирование
//!    тубулина, потеря дистальных придатков (CEP164, CEP89, Ninein, CEP170),
//!    нарушение фосфорилирования.
//!
//! 3. **Петля обратной связи**: повреждённая центриоль → нарушение митофагии →
//!    дисфункция митохондрий → рост ROS → ещё больше повреждений.
//!
//! 4. **Два пути патологии АКД**: истощение пула (оба потомка дифференцируются)
//!    или гиперпролиферация (оба сохраняют стволовость).
//!
//! 5. **Тканеспецифичное старение**: NSC (когниция), HSC (иммунитет),
//!    миосателлиты (саркопения), кишечные крипты.

pub mod damage;
pub mod development;
pub mod inducers;
pub mod organism;
pub mod tissues;

use cell_dt_core::{
    SimulationModule, SimulationResult,
    components::*,
    hecs::World,
};
use serde_json::{json, Value};
use log::info;

pub use damage::DamageParams;
pub use development::DevelopmentParams;
pub use organism::OrganismSimulator;
pub use tissues::TissueSimulator;

/// Параметры модуля развития человека
#[derive(Debug, Clone)]
pub struct HumanDevelopmentParams {
    /// Параметры накопления повреждений
    pub damage: DamageParams,
    /// Параметры развития
    pub development: DevelopmentParams,
    /// Временной масштаб: шагов симуляции на 1 год
    pub steps_per_year: u64,
    /// Стохастический сид
    pub seed: u64,
}

impl Default for HumanDevelopmentParams {
    fn default() -> Self {
        Self {
            damage: DamageParams::default(),
            development: DevelopmentParams::default(),
            steps_per_year: 10,
            seed: 42,
        }
    }
}

/// Главный модуль — интегрирует в SimulationManager
pub struct HumanDevelopmentModule {
    pub params: HumanDevelopmentParams,
    pub organism: OrganismSimulator,
    pub tissues: Vec<TissueSimulator>,
    step_count: u64,
}

impl HumanDevelopmentModule {
    pub fn new() -> Self {
        Self::with_params(HumanDevelopmentParams::default())
    }

    pub fn with_params(params: HumanDevelopmentParams) -> Self {
        let organism = OrganismSimulator::new(&params);
        let tissues = vec![
            TissueSimulator::new(TissueType::Neural,         &params.damage),
            TissueSimulator::new(TissueType::Hematopoietic,  &params.damage),
            TissueSimulator::new(TissueType::Muscle,         &params.damage),
            TissueSimulator::new(TissueType::IntestinalCrypt,&params.damage),
            TissueSimulator::new(TissueType::Skin,           &params.damage),
        ];
        Self { params, organism, tissues, step_count: 0 }
    }

    /// Текущий возраст организма в годах
    pub fn age_years(&self) -> f64 {
        self.organism.state.age_years
    }

    /// Снимок ключевых метрик для вывода
    pub fn snapshot(&self) -> OrganismSnapshot {
        OrganismSnapshot {
            age_years:          self.organism.state.age_years,
            stage:              self.organism.state.developmental_stage,
            frailty:            self.organism.state.frailty_index,
            cognitive:          self.organism.state.cognitive_index,
            immune:             self.organism.state.immune_reserve,
            muscle:             self.organism.state.muscle_mass,
            inflammaging:       self.organism.state.inflammaging_score,
            is_alive:           self.organism.state.is_alive,
            tissues: self.tissues.iter().map(|t| t.state.clone()).collect(),
        }
    }
}

impl Default for HumanDevelopmentModule {
    fn default() -> Self {
        Self::new()
    }
}

/// Снимок состояния организма для анализа и вывода
#[derive(Debug, Clone)]
pub struct OrganismSnapshot {
    pub age_years:    f64,
    pub stage:        DevelopmentalStage,
    pub frailty:      f32,
    pub cognitive:    f32,
    pub immune:       f32,
    pub muscle:       f32,
    pub inflammaging: f32,
    pub is_alive:     bool,
    pub tissues:      Vec<TissueState>,
}

impl SimulationModule for HumanDevelopmentModule {
    fn name(&self) -> &str {
        "human_development_module"
    }

    fn initialize(&mut self, _world: &mut World) -> SimulationResult<()> {
        info!("Инициализация модуля развития человека (CDATA)");
        info!("  S-индукторов: {}, H-индукторов: {}",
            self.params.development.s_inducers_initial,
            self.params.development.h_inducers_initial);
        info!("  Базовая скорость повреждения: {:.4}",
            self.params.damage.base_ros_damage_rate);
        Ok(())
    }

    fn step(&mut self, _world: &mut World, dt: f64) -> SimulationResult<()> {
        self.step_count += 1;

        // Перевод dt (шагов) в годы
        let dt_years = dt / self.params.steps_per_year as f64;

        // 1. Обновить возраст и стадию развития организма
        self.organism.advance(dt_years);

        // 2. Обновить каждую ткань
        for tissue in &mut self.tissues {
            tissue.step(dt_years as f32, self.organism.state.age_years as f32, &self.params.damage);
        }

        // 3. Интегрировать тканевые метрики в состояние организма
        self.organism.integrate_tissue_metrics(&self.tissues);

        Ok(())
    }

    fn get_params(&self) -> Value {
        json!({
            "steps_per_year": self.params.steps_per_year,
            "seed": self.params.seed,
            "damage": {
                "base_ros_damage_rate": self.params.damage.base_ros_damage_rate,
                "ros_feedback_coefficient": self.params.damage.ros_feedback_coefficient,
                "cep164_loss_rate": self.params.damage.cep164_loss_rate,
                "senescence_threshold": self.params.damage.senescence_threshold,
            },
            "development": {
                "s_inducers_initial": self.params.development.s_inducers_initial,
                "h_inducers_initial": self.params.development.h_inducers_initial,
            }
        })
    }

    fn set_params(&mut self, params: &Value) -> SimulationResult<()> {
        if let Some(v) = params.get("steps_per_year").and_then(|v| v.as_u64()) {
            self.params.steps_per_year = v;
        }
        if let Some(v) = params.get("base_ros_damage_rate").and_then(|v| v.as_f64()) {
            self.params.damage.base_ros_damage_rate = v as f32;
        }
        Ok(())
    }
}
