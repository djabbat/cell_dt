//! Модуль развития человека — CDATA (Centriolar Damage Accumulation Theory of Aging)
//!
//! Интегрирует эмбриологию, геронтологию и теорию накопления
//! центриолярных повреждений Дж. Ткемаладзе в единую ECS-симуляцию.
//!
//! ## Ключевые механизмы
//! * **S/H-индукторы** — счётчик Хейфлика; каждый дифференцирующий митоз
//!   тратит один S-индуктор (≈50 соматических делений на клетку).
//! * **Трек A** — дефект реснички (CEP164↓ → Shh/Wnt-сигнал↓ → ниша не
//!   восстанавливается).
//! * **Трек B** — дефект веретена (spindle_fidelity↓ → асимметрия нарушена →
//!   истощение пула ИЛИ клональная экспансия).
//! * **Петля ROS** — повреждённая центриоль → митофагия↓ → ROS↑ → больше
//!   карбонилирования и агрегатов.

use cell_dt_core::{
    SimulationModule, SimulationResult,
    hecs::World,
    components::{
        CentriolarDamageState, CentriolarInducers,
        TissueState, TissueType,
        CellCycleStateExtended,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use log::{info, debug};
use rand::Rng;
use std::collections::VecDeque;

mod inducers;
mod tissues;
mod aging;
pub mod damage;
pub mod development;

pub use inducers::*;
pub use tissues::*;
pub use aging::*;
pub use damage::{DamageParams, accumulate_damage};
pub use development::{division_rate_per_year, base_ros_level, stage_for_age};

// ---------------------------------------------------------------------------
// Этапы развития (15 стадий — от зиготы до старческого возраста)
// ---------------------------------------------------------------------------

/// Этапы развития человека
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HumanDevelopmentalStage {
    /// Зигота (0–1 день)
    Zygote,
    /// Дробление (1–3 дня)
    Cleavage,
    /// Морула (3–4 дня)
    Morula,
    /// Бластоциста (4–7 дней)
    Blastocyst,
    /// Имплантация (7–14 дней)
    Implantation,
    /// Гаструляция (14–21 день)
    Gastrulation,
    /// Нейруляция (21–28 дней)
    Neurulation,
    /// Органогенез (4–8 недель)
    Organogenesis,
    /// Плодный период (9–40 недель)
    Fetal,
    /// Новорождённый (0–1 год)
    Newborn,
    /// Детство (1–12 лет)
    Childhood,
    /// Подростковый возраст (12–18 лет)
    Adolescence,
    /// Взрослый (18–50 лет)
    Adult,
    /// Средний возраст (50–70 лет)
    MiddleAge,
    /// Старческий возраст (70+ лет)
    Elderly,
}

// ---------------------------------------------------------------------------
// Главный компонент — прикрепляется к каждой сущности (стволовой нише)
// ---------------------------------------------------------------------------

/// Полный CDATA-компонент развития и старения человека.
///
/// Каждая сущность (entity) в ECS представляет одну стволовую нишу.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanDevelopmentComponent {
    // --- Стадии развития ---
    /// Текущая детализированная стадия развития (15 вариантов)
    pub stage: HumanDevelopmentalStage,
    /// Возраст в днях
    pub age_days: f64,
    /// Уровень морфогенеза
    pub morphogenetic_level: HumanMorphogeneticLevel,
    /// Анатомический тип ткани данной ниши
    pub tissue_type: HumanTissueType,
    /// История переходов между стадиями (стадия, день начала)
    pub stage_history: VecDeque<(HumanDevelopmentalStage, f64)>,

    // --- CDATA: молекулярные повреждения материнской центриоли ---
    /// Молекулярное состояние центриоли (5 типов повреждений + придатки)
    pub centriolar_damage: CentriolarDamageState,
    /// Калиброванные скорости накопления повреждений (default ≈ 76 лет)
    pub damage_rates: DamageParams,

    // --- CDATA: система индукторов (S/H-структуры) ---
    /// Запас соматических (S) и гаметных (H) индукторов
    pub inducers: CentriolarInducers,

    // --- Тканевое состояние (интегральные метрики ниши) ---
    pub tissue_state: TissueState,

    // --- Фенотипы старения ---
    /// Связь повреждений центриоли с фенотипами старения
    pub centriole_aging: CentrioleAgingLink,
    /// Активные фенотипы старения
    pub active_phenotypes: Vec<AgingPhenotype>,

    // --- Жив ли организм/ниша ---
    pub is_alive: bool,
}

impl HumanDevelopmentComponent {
    /// Создать компонент для указанного типа ткани.
    pub fn for_tissue(tissue_type: HumanTissueType) -> Self {
        let core_type = map_tissue_type(tissue_type);
        Self {
            stage: HumanDevelopmentalStage::Zygote,
            age_days: 0.0,
            morphogenetic_level: HumanMorphogeneticLevel::Embryonic,
            tissue_type,
            stage_history: VecDeque::new(),
            centriolar_damage: CentriolarDamageState::pristine(),
            damage_rates: DamageParams::default(),
            inducers: CentriolarInducers::default(),
            tissue_state: TissueState::new(core_type),
            centriole_aging: CentrioleAgingLink::default(),
            active_phenotypes: Vec::new(),
            is_alive: true,
        }
    }

    /// Возраст в годах.
    pub fn age_years(&self) -> f64 {
        self.age_days / 365.25
    }

    /// Индекс дряхлости [0..1]; при > 0.95 — смерть.
    pub fn frailty(&self) -> f32 {
        1.0 - self.tissue_state.functional_capacity
    }

    /// Суммарный балл повреждений [0..1].
    pub fn damage_score(&self) -> f32 {
        self.centriolar_damage.total_damage_score()
    }
}

impl Default for HumanDevelopmentComponent {
    fn default() -> Self {
        Self::for_tissue(HumanTissueType::Epithelial)
    }
}

// ---------------------------------------------------------------------------
// Параметры модуля
// ---------------------------------------------------------------------------

/// Параметры модуля развития человека
#[derive(Debug, Clone)]
pub struct HumanDevelopmentParams {
    /// Ускорение времени: 1.0 = 1 симуляционный шаг (dt) соответствует 1 дню.
    /// При dt=1.0 и time_acceleration=1.0: 365 шагов = 1 год.
    pub time_acceleration: f64,
    /// Включить ли накопление повреждений (старение)
    pub enable_aging: bool,
    /// Включить ли морфогенез
    pub enable_morphogenesis: bool,
    /// Уровень детализации тканей (резерв для будущих расширений)
    pub tissue_detail_level: usize,
}

impl Default for HumanDevelopmentParams {
    fn default() -> Self {
        Self {
            time_acceleration: 1.0,
            enable_aging: true,
            enable_morphogenesis: true,
            tissue_detail_level: 3,
        }
    }
}

// ---------------------------------------------------------------------------
// Модуль
// ---------------------------------------------------------------------------

/// Модуль развития человека (реализует `SimulationModule`)
pub struct HumanDevelopmentModule {
    params: HumanDevelopmentParams,
    step_count: u64,
}

impl HumanDevelopmentModule {
    pub fn new() -> Self {
        Self {
            params: HumanDevelopmentParams::default(),
            step_count: 0,
        }
    }

    pub fn with_params(params: HumanDevelopmentParams) -> Self {
        Self {
            params,
            step_count: 0,
        }
    }

    // -----------------------------------------------------------------------
    // Вспомогательные функции
    // -----------------------------------------------------------------------

    /// Обновить детализированную стадию развития по возрасту.
    fn update_stage(component: &mut HumanDevelopmentComponent) {
        let age = component.age_days;
        let new_stage = if age < 1.0 {
            HumanDevelopmentalStage::Zygote
        } else if age < 3.0 {
            HumanDevelopmentalStage::Cleavage
        } else if age < 4.0 {
            HumanDevelopmentalStage::Morula
        } else if age < 7.0 {
            HumanDevelopmentalStage::Blastocyst
        } else if age < 14.0 {
            HumanDevelopmentalStage::Implantation
        } else if age < 21.0 {
            HumanDevelopmentalStage::Gastrulation
        } else if age < 28.0 {
            HumanDevelopmentalStage::Neurulation
        } else if age < 56.0 {
            HumanDevelopmentalStage::Organogenesis
        } else if age < 280.0 {
            HumanDevelopmentalStage::Fetal
        } else if age < 365.0 {
            HumanDevelopmentalStage::Newborn
        } else if age < 4380.0 {
            // 12 лет
            HumanDevelopmentalStage::Childhood
        } else if age < 6570.0 {
            // 18 лет
            HumanDevelopmentalStage::Adolescence
        } else if age < 18250.0 {
            // 50 лет
            HumanDevelopmentalStage::Adult
        } else if age < 25550.0 {
            // 70 лет
            HumanDevelopmentalStage::MiddleAge
        } else {
            HumanDevelopmentalStage::Elderly
        };

        if new_stage != component.stage {
            component.stage_history.push_back((new_stage, age));
            component.stage = new_stage;
        }
    }

    /// Обновить тканевое состояние на основе центриолярных повреждений.
    ///
    /// Тканевые метрики — прямое отражение молекулярного состояния центриоли
    /// (без отдельного накопления), что обеспечивает корректную калибровку:
    ///
    /// * **Трек A** (цилиарный): `ciliary_function` → `regeneration_tempo`.
    ///   Снижение CEP164/CEP89/Ninein/CEP170 → первичная ресничка не работает →
    ///   нарушена Shh/Wnt-сигнализация ниши → самообновление невозможно.
    /// * **Трек B** (веретено): `spindle_fidelity↓` → растёт вероятность
    ///   симметричных про-дифференцировочных делений → уменьшается
    ///   `stem_cell_pool`.
    /// * Смерть: управляется `centriolar_damage.is_senescent` (суммарный ущерб
    ///   > 0.75), откалиброванным в damage.rs на ~78 лет.
    fn update_tissue_state(component: &mut HumanDevelopmentComponent) {
        let dam = &component.centriolar_damage;

        // Трек A: функция реснички определяет темп регенерации напрямую
        component.tissue_state.regeneration_tempo = dam.ciliary_function;

        // Трек B: пул стволовых клеток обратно пропорционален
        //         вероятности симметричного про-дифференцировочного деления
        let pool_loss_prob = dam.pool_exhaustion_probability();
        component.tissue_state.stem_cell_pool = (1.0 - pool_loss_prob).max(0.0);

        // Индуктор истощён — Pool дополнительно снижается до нуля
        if component.inducers.is_terminally_differentiated() {
            component.tissue_state.stem_cell_pool = 0.0;
        }

        // Доля сенесцентных клеток пропорциональна суммарному ущербу
        component.tissue_state.senescent_fraction =
            (dam.total_damage_score() * 0.85).min(1.0);

        // Функциональная ёмкость = pool × tempo × (1 − 0.8 × senescent)
        component.tissue_state.update_functional_capacity();
    }

    /// Обновить систему S/H-индукторов.
    ///
    /// При каждом Track-B-событии (симметричное про-дифференцировочное деление)
    /// потребляется один S-индуктор. Когда S=0, пул стволовых клеток истощён.
    fn update_inducer_system(
        component: &mut HumanDevelopmentComponent,
        div_rate_per_year: f32,
        dt_years: f32,
        rng: &mut impl Rng,
    ) {
        if component.inducers.is_terminally_differentiated() {
            return;
        }
        // Ожидаемое число Track-B-событий за шаг
        let pool_ex_prob = component.centriolar_damage.pool_exhaustion_probability();
        let expected_events = pool_ex_prob * div_rate_per_year * dt_years;
        if rng.gen::<f32>() < expected_events {
            component.inducers.consume_s_inducer();
        }
    }

    /// Обновить связь центриолярных повреждений с фенотипами старения.
    fn update_aging_phenotypes(component: &mut HumanDevelopmentComponent) {
        let dam = &component.centriolar_damage;

        // CentrioleAgingLink
        component.centriole_aging.cilia_loss =
            (1.0 - dam.ciliary_function).max(0.0);
        component.centriole_aging.ptm_accumulation =
            (dam.tubulin_hyperacetylation + dam.phosphorylation_dysregulation) / 2.0;
        component.centriole_aging.cycle_dysregulation =
            (1.0 - dam.spindle_fidelity).max(0.0);
        component.centriole_aging.asymmetry_loss =
            dam.symmetric_division_probability();
        component.centriole_aging.satellite_accumulation =
            dam.protein_aggregates;

        // Активные фенотипы
        component.active_phenotypes.clear();
        let total = dam.total_damage_score();

        if total > 0.1 { component.active_phenotypes.push(AgingPhenotype::ReducedProliferation); }
        if dam.protein_aggregates > 0.2 { component.active_phenotypes.push(AgingPhenotype::ProteinAggregation); }
        if dam.ros_level > 0.3 { component.active_phenotypes.push(AgingPhenotype::MitochondrialDysfunction); }
        if component.centriole_aging.ptm_accumulation > 0.15 {
            component.active_phenotypes.push(AgingPhenotype::EpigeneticChanges);
        }
        if component.tissue_state.senescent_fraction > 0.3 {
            component.active_phenotypes.push(AgingPhenotype::SenescentAccumulation);
        }
        if component.centriole_aging.cilia_loss > 0.3 {
            component.active_phenotypes.push(AgingPhenotype::SignalingDysregulation);
        }
        if dam.protein_aggregates > 0.3 { component.active_phenotypes.push(AgingPhenotype::ProteostasisLoss); }
        if component.tissue_state.stem_cell_pool < 0.5 {
            component.active_phenotypes.push(AgingPhenotype::StemCellExhaustion);
        }
        if total > 0.5 { component.active_phenotypes.push(AgingPhenotype::AlteredCommunication); }
    }
}

impl Default for HumanDevelopmentModule {
    fn default() -> Self {
        Self::new()
    }
}

impl SimulationModule for HumanDevelopmentModule {
    fn name(&self) -> &str {
        "human_development_module"
    }

    fn step(&mut self, world: &mut World, dt: f64) -> SimulationResult<()> {
        self.step_count += 1;

        // dt в днях (при dt=1.0 и time_acceleration=1.0 каждый шаг = 1 день)
        let dt_days = dt * self.params.time_acceleration;
        let dt_years = (dt_days / 365.25) as f32;

        debug!("Human development step {}, dt_days={:.3}", self.step_count, dt_days);

        let mut rng = rand::thread_rng();
        let mut query = world.query::<&mut HumanDevelopmentComponent>();

        for (_, comp) in query.iter() {
            if !comp.is_alive {
                continue;
            }

            // 1. Возраст
            comp.age_days += dt_days;
            let age_years = comp.age_years() as f32;

            // 2. Стадия и морфогенетический уровень
            if self.params.enable_morphogenesis {
                comp.morphogenetic_level =
                    HumanInducers::get_morphogenetic_level(comp.age_days);
                Self::update_stage(comp);
            }

            if self.params.enable_aging {
                // 3. Скорость делений на этой стадии (для Track B и индукторов)
                let core_stage = stage_for_age(comp.age_years());
                let div_rate = division_rate_per_year(core_stage);

                // 4. Накопление молекулярных повреждений (5 типов + ROS петля)
                accumulate_damage(
                    &mut comp.centriolar_damage,
                    &comp.damage_rates,
                    age_years,
                    dt_years,
                );

                // 5. Тканевое состояние (Трек A + Трек B, прямое отражение ущерба)
                Self::update_tissue_state(comp);

                // 6. Система индукторов
                Self::update_inducer_system(comp, div_rate, dt_years, &mut rng);

                // 7. Фенотипы старения
                Self::update_aging_phenotypes(comp);

                // 8. Смерть
                // Первичный критерий — молекулярный сенесценс центриоли
                // (суммарный ущерб > 0.75, calibrated ~78 лет в damage.rs).
                // Резервный — критическое истощение тканевой функции (фрайлти ≥ 0.97).
                if comp.centriolar_damage.is_senescent || comp.frailty() >= 0.97 {
                    comp.is_alive = false;
                    debug!(
                        "Niche {:?} died at age {:.1} yr (senescent={}, frailty={:.3})",
                        comp.tissue_type,
                        age_years,
                        comp.centriolar_damage.is_senescent,
                        comp.frailty()
                    );
                }
            }
        }

        Ok(())
    }

    fn get_params(&self) -> Value {
        json!({
            "time_acceleration": self.params.time_acceleration,
            "enable_aging":       self.params.enable_aging,
            "enable_morphogenesis": self.params.enable_morphogenesis,
            "tissue_detail_level":  self.params.tissue_detail_level,
            "step_count":           self.step_count,
        })
    }

    fn set_params(&mut self, params: &Value) -> SimulationResult<()> {
        if let Some(v) = params.get("time_acceleration").and_then(|v| v.as_f64()) {
            self.params.time_acceleration = v;
        }
        if let Some(v) = params.get("enable_aging").and_then(|v| v.as_bool()) {
            self.params.enable_aging = v;
        }
        if let Some(v) = params.get("enable_morphogenesis").and_then(|v| v.as_bool()) {
            self.params.enable_morphogenesis = v;
        }
        Ok(())
    }

    /// Инициализировать компонент для каждой сущности с `CellCycleStateExtended`.
    /// Тип ткани назначается циклически по списку основных тканей.
    fn initialize(&mut self, world: &mut World) -> SimulationResult<()> {
        info!("Initializing human development module (CDATA)");

        let entities: Vec<_> = world
            .query::<&CellCycleStateExtended>()
            .iter()
            .map(|(e, _)| e)
            .collect();

        let tissue_cycle = [
            HumanTissueType::Neural,
            HumanTissueType::Blood,
            HumanTissueType::Epithelial,
            HumanTissueType::Muscle,
            HumanTissueType::Skin,
        ];

        let count = entities.len();
        for (i, &entity) in entities.iter().enumerate() {
            if !world.contains(entity) {
                continue;
            }
            let tissue = tissue_cycle[i % tissue_cycle.len()];
            let component = HumanDevelopmentComponent::for_tissue(tissue);
            world.insert_one(entity, component)?;
        }

        info!("Initialized CDATA development for {} niches", count);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Вспомогательные функции
// ---------------------------------------------------------------------------

/// Сопоставить анатомический тип ткани с функциональным типом стволовой ниши.
pub fn map_tissue_type(human_type: HumanTissueType) -> TissueType {
    match human_type {
        HumanTissueType::Neural => TissueType::Neural,
        HumanTissueType::Blood => TissueType::Hematopoietic,
        HumanTissueType::Epithelial
        | HumanTissueType::Liver
        | HumanTissueType::Kidney
        | HumanTissueType::Lung => TissueType::IntestinalCrypt,
        HumanTissueType::Muscle | HumanTissueType::Heart => TissueType::Muscle,
        _ => TissueType::Skin,
    }
}
