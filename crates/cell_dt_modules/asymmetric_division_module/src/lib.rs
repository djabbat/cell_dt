//! Модуль асимметричного деления стволовых клеток
//!
//! Тип деления определяется двумя факторами:
//! 1. **Уровень потентности** (`CentriolarInducerPair`) — сколько индукторов
//!    осталось на каждой центриоли.
//! 2. **Точность веретена** (`spindle_fidelity`) — насколько правильно
//!    формируется митотическое веретено при накопленных повреждениях.
//!
//! O₂, достигая центриолей, отщепляет индукторы (логика в `human_development_module`).
//! Этот модуль отражает следствие: выбирает тип деления на основе текущего состояния.

use cell_dt_core::{
    SimulationModule, SimulationResult,
    components::*,
    hecs::World,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use log::{info, debug};
use std::collections::HashMap;

/// Тип деления стволовой клетки
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DivisionType {
    /// Оба потомка — стволовые клетки (экспансия пула)
    Symmetric,
    /// Один потомок — стволовая, другой — более дифференцированная
    Asymmetric,
    /// Оба потомка — стволовые клетки (самообновление без экспансии)
    SelfRenewal,
    /// Оба потомка — дифференцированные (истощение пула)
    Differentiation,
}

/// Компонент для отслеживания асимметричного деления
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsymmetricDivisionComponent {
    pub division_type: DivisionType,
    pub niche_id: Option<u64>,
    pub stemness_potential: f32,
    /// Число асимметричных делений в этой нише
    pub asymmetric_count: u32,
    /// Число делений типа Differentiation (истощение)
    pub exhaustion_count: u32,
}

impl Default for AsymmetricDivisionComponent {
    fn default() -> Self {
        Self {
            division_type: DivisionType::Symmetric,
            niche_id: None,
            stemness_potential: 0.8,
            asymmetric_count: 0,
            exhaustion_count: 0,
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
    /// Порог spindle_fidelity ниже которого деление всегда симметричное (истощение)
    pub spindle_failure_threshold: f32,
}

impl Default for AsymmetricDivisionParams {
    fn default() -> Self {
        Self {
            asymmetric_division_probability: 0.3,
            symmetric_renewal_probability: 0.4,
            symmetric_diff_probability: 0.3,
            stem_cell_niche_capacity: 10,
            max_niches: 100,
            spindle_failure_threshold: 0.3,
        }
    }
}

/// Модуль асимметричного деления
pub struct AsymmetricDivisionModule {
    params: AsymmetricDivisionParams,
    step_count: u64,
    /// Ниши: id → (x, y, z, radius)
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

    pub fn create_niche(&mut self, x: f32, y: f32, z: f32, radius: f32) -> u64 {
        let id = self.next_niche_id;
        self.niches.insert(id, (x, y, z, radius));
        self.next_niche_id += 1;
        id
    }

    /// Определить тип деления на основе потентности и повреждений веретена.
    ///
    /// # Логика:
    /// - **Тотипотент/Плюрипотент + высокий spindle** → асимметричное (норма)
    /// - **Олигопотент** → чаще симметричное самообновление или дифференцировка
    /// - **Унипотент** → почти всегда дифференцировка
    /// - **spindle_fidelity < порог** → веретено нарушено → только симметричное
    ///   (оба потомка дифференцируются — истощение пула)
    /// - **Апоптоз** → не делится
    fn classify_division(
        potency: PotencyLevel,
        spindle_fidelity: f32,
        spindle_threshold: f32,
    ) -> Option<DivisionType> {
        // Если веретено сломано — только симметричное дифференцирование
        if spindle_fidelity < spindle_threshold {
            return Some(DivisionType::Differentiation);
        }

        match potency {
            PotencyLevel::Totipotent | PotencyLevel::Pluripotent => {
                // Здоровая стволовая клетка с двумя заполненными комплектами
                // → асимметричное деление (один потомок теряет часть индукторов)
                Some(DivisionType::Asymmetric)
            }
            PotencyLevel::Oligopotent => {
                // Один комплект уже пуст → труднее удерживать стволовость
                // → равновероятно самообновление или дифференцировка
                Some(DivisionType::SelfRenewal)
            }
            PotencyLevel::Unipotent => {
                // Почти никакого запаса → дифференцировка
                Some(DivisionType::Differentiation)
            }
            PotencyLevel::Apoptosis => {
                // Апоптоз инициирован — деление невозможно
                None
            }
        }
    }
}

impl SimulationModule for AsymmetricDivisionModule {
    fn name(&self) -> &str { "asymmetric_division_module" }

    fn step(&mut self, world: &mut World, _dt: f64) -> SimulationResult<()> {
        self.step_count += 1;
        debug!("Asymmetric division step {}", self.step_count);

        // Определяем тип деления для каждой сущности с нужными компонентами.
        // CentriolarInducerPair хранится в HumanDevelopmentComponent, но здесь
        // мы работаем с CentriolarDamageState (доступен через cell_dt_core),
        // используя spindle_fidelity и is_senescent как прокси.
        for (_, (div_comp, damage, cycle)) in world.query_mut::<(
            &mut AsymmetricDivisionComponent,
            &CentriolarDamageState,
            &CellCycleStateExtended,
        )>() {
            // Обновляем только в фазе M (деление)
            if cycle.phase != Phase::M {
                continue;
            }

            // Прокси-потентность из spindle_fidelity (пока HumanDevelopmentComponent
            // не является отдельным ECS-компонентом с CentriolarInducerPair)
            let proxy_potency = if damage.is_senescent {
                PotencyLevel::Apoptosis
            } else if damage.spindle_fidelity > 0.95 {
                PotencyLevel::Totipotent
            } else if damage.spindle_fidelity > 0.75 {
                PotencyLevel::Pluripotent
            } else if damage.spindle_fidelity > 0.45 {
                PotencyLevel::Oligopotent
            } else if damage.spindle_fidelity > 0.15 {
                PotencyLevel::Unipotent
            } else {
                PotencyLevel::Apoptosis
            };

            if let Some(div_type) = Self::classify_division(
                proxy_potency,
                damage.spindle_fidelity,
                self.params.spindle_failure_threshold,
            ) {
                // Статистика
                match div_type {
                    DivisionType::Asymmetric => div_comp.asymmetric_count += 1,
                    DivisionType::Differentiation => div_comp.exhaustion_count += 1,
                    _ => {}
                }
                div_comp.division_type = div_type;
            }

            // Обновить stemness_potential из spindle_fidelity
            div_comp.stemness_potential = damage.spindle_fidelity
                * (1.0 - damage.protein_aggregates * 0.3);
        }

        Ok(())
    }

    fn get_params(&self) -> Value {
        json!({
            "asymmetric_division_probability": self.params.asymmetric_division_probability,
            "symmetric_renewal_probability":   self.params.symmetric_renewal_probability,
            "symmetric_diff_probability":      self.params.symmetric_diff_probability,
            "stem_cell_niche_capacity":        self.params.stem_cell_niche_capacity,
            "max_niches":                      self.params.max_niches,
            "spindle_failure_threshold":       self.params.spindle_failure_threshold,
            "step_count":                      self.step_count,
            "active_niches":                   self.niches.len(),
        })
    }

    fn set_params(&mut self, params: &Value) -> SimulationResult<()> {
        if let Some(v) = params.get("asymmetric_division_probability").and_then(|v| v.as_f64()) {
            self.params.asymmetric_division_probability = v as f32;
        }
        if let Some(v) = params.get("spindle_failure_threshold").and_then(|v| v.as_f64()) {
            self.params.spindle_failure_threshold = v as f32;
        }
        Ok(())
    }

    fn initialize(&mut self, world: &mut World) -> SimulationResult<()> {
        info!("Initializing asymmetric division module");

        let entities: Vec<_> = world
            .query::<&CellCycleStateExtended>()
            .iter()
            .map(|(e, _)| e)
            .collect();

        let count = entities.len();
        for &entity in &entities {
            if !world.contains(entity) { continue; }
            world.insert_one(entity, AsymmetricDivisionComponent::default())?;
        }

        // Создаём стартовые ниши
        for i in 0..3 {
            self.create_niche(0.0, 0.0, (i * 10) as f32, 5.0);
        }

        info!("Initialized {} cells, {} niches", count, self.niches.len());
        Ok(())
    }
}

impl Default for AsymmetricDivisionModule {
    fn default() -> Self { Self::new() }
}
