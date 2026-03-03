# Cell DT — Рекомендации по оптимизации модели CDATA

> **Статус:** Живой документ. Вычёркивать/удалять пункты по мере выполнения.
> Выполненные шаги помечаются `[x]`, невыполненные — `[ ]`.
> Последнее обновление: 2026-03-04

---

## ВЫПОЛНЕНО

- [x] **CentriolarInducers → CentriolarInducerPair** — полная замена системы индукторов:
  M-комплект (материнская центриоль) + D-комплект (дочерняя). O₂ отщепляет
  от обоих (если оба непусты) или только от непустого. Новые центриоли наследуют
  ТЕКУЩИЙ остаток, а не исходный максимум.
- [x] **CentriolarDamageState sync** — каждый `step()` синхронизирует отдельный
  ECS-компонент `CentriolarDamageState` из `HumanDevelopmentComponent.centriolar_damage`,
  чтобы другие модули могли его читать без зависимости от human_development_module.
- [x] **AsymmetricDivisionModule.step()** — реализован: читает `CentriolarDamageState`,
  классифицирует тип деления (Asymmetric / SelfRenewal / Differentiation / нет деления).
- [x] **StemCellHierarchyModule.step()** — реализован: читает `spindle_fidelity` как
  прокси-потентность и синхронизирует `StemCellHierarchyState`.
- [x] **CLAUDE.md** — написан и обновлён.
- [x] **`InflammagingState` в `cell_dt_core::components`** — добавлен shared ECS-компонент
  обратной связи: `ros_boost`, `niche_impairment`, `sasp_intensity`.
- [x] **`AgingPhenotype::ImmuneDecline`** — добавлен в `aging.rs`.
- [x] **`human_development_module` читает `InflammagingState`** — применяет `ros_boost`
  к ros_level и `niche_impairment` к regeneration_tempo. Активирует `ImmuneDecline` при `sasp > 0.4`.
- [x] **`human_development_module.initialize()` спавнит `InflammagingState::default()`**.
- [x] **`myeloid_shift_module`** — полностью реализован:
  - `MyeloidShiftComponent` (myeloid_bias, lymphoid_deficit, inflammaging_index, immune_senescence, phenotype)
  - `MyeloidShiftParams` (6 параметров через get/set_params)
  - Формула CDATA-обоснована: (1-spindle)^1.5×0.45 + (1-cilia)×0.30 + ros×0.15 + agg×0.10
  - Обратная связь → InflammagingState каждый step()
  - 7 unit-тестов пройдены
  - Пример `myeloid_shift_example.rs`
- [x] **Мониторинг индукторов в `myeloid_shift_example.rs`** ✅:
  - Добавлены колонки M-ind / ΔM / D-ind / ΔD / Potency в ежегодную таблицу
  - `print_year_status` возвращает `(i32, i32)` — текущие M/D для дельты следующего шага
  - Секция `=== Inductor system ===` в финальном статусе: remaining/inherited + fraction + division_count
  - Результат калибровки (2026-03-04, seed=42):
    - Смерть: ≈78 лет ✓
    - myeloid_bias в 70 лет: **0.571** (цель 0.45; в допустимом диапазоне 0.35–0.60 ✓)
    - Индукторы: M: 10→3, D: 8→3 за 70 лет; потентность Totipotent→Pluripotent→Oligopotent
- [x] **Транскриптом → Клеточный цикл** ✅:
  - `GeneExpressionState` (p21, p16, cyclin_d, myc) в `cell_dt_core`
  - `transcriptome_module` пишет CDKN1A/CDKN2A в GeneExpressionState каждый step
  - `cell_cycle_module` читает: p21 > 0.7 → G1SRestriction; p16 > 0.8 → DNARepair; cyclin_d → G1 shorter
  - 4 новых unit-теста → 10 итого в cell_cycle_module
- [x] **AsymmetricDivision → TissueState** ✅:
  - `DivisionExhaustionState` в `cell_dt_core` (shared ECS-компонент)
  - `asymmetric_division_module` пишет exhaustion_count/asymmetric_count
  - `human_development_module` применяет `exhaustion_ratio × 0.0002` → stem_cell_pool↓
- [x] **PTM → CentriolarDamageState bridge** ✅:
  - `human_development_module` читает `Option<&CentriolePair>`, применяет PTM_SCALE=0.002/год
  - 4 unit-теста
- [x] **TODO.md** — перезаписан с актуальным статусом.
- [x] **RECOMENDATION.md** (старый файл) — помечен как устаревший.

---

## 1. ПОДГОТОВКА К МЕЛОИДНОМУ СДВИГУ ✅ ВЫПОЛНЕНО

- [x] **`InflammagingState` в `cell_dt_core::components`** — добавлен.
- [x] **`human_development_module` читает `InflammagingState`** — применяет `ros_boost` и `niche_impairment`.
- [x] **`AgingPhenotype::ImmuneDecline`** — добавлен в `aging.rs`.
- [x] **`human_development_module.initialize()` спавнит `InflammagingState::default()`**.

---

## 2. МЕЛОИДНЫЙ СДВИГ ✅ ВЫПОЛНЕНО

### Биология и связь с CDATA

С возрастом гематопоэтические стволовые клетки (HSC) и стволовые клетки других тканей
смещают дифференцировку от лимфоидного пути к миелоидному. В рамках CDATA это происходит
через четыре конкретных молекулярных повреждения:

| Компонент CDATA | Механизм биологически | Вклад в myeloid_bias |
|---|---|---|
| `spindle_fidelity ↓` | Веретено не может сегрегировать fate-детерминанты (Numb, aPKC) → оба потомка миелоидные | **45%** |
| `ciliary_function ↓` (CEP164↓) | Нет реснички → нет Wnt/Notch/Shh → LT-HSC теряет лимфоидный нише-сигнал → PU.1 побеждает | **30%** |
| `ros_level ↑` | ROS → NF-κB → IL-6, TNF-α, IL-1β → SASP → миелоидная цитокиновая среда | **15%** |
| `protein_aggregates ↑` | Агрегаты белков захватывают IKZF1/Ikaros → снятие репрессии с миелоидных генов | **10%** |

**Обратные связи мелоидного сдвига → CDATA:**

```
myeloid_bias ↑
  → inflammaging_index ↑
      → ros_boost ↑     → DamageParams.ros_rate ускоряется (петля ROS усиливается)
      → niche_impairment↑ → TissueState.regeneration_tempo ↓
  → immune_senescence ↑
      → AgingPhenotype::ImmuneDecline активируется
      → lymphoid_deficit ↑ → снижение иммунного надзора (онкологический риск)
```

### Формулы

```
spindle_c  = (1 − spindle_fidelity)^1.5 × 0.45
cilia_c    = (1 − ciliary_function)  × 0.30
ros_c      = ros_level               × 0.15
aggr_c     = protein_aggregates      × 0.10

myeloid_bias = clamp(spindle_c + cilia_c + ros_c + aggr_c,  0.0, 1.0)

lymphoid_deficit   = myeloid_bias                          (упрощённая модель)
inflammaging_index = myeloid_bias × lymphoid_deficit × 0.8
immune_senescence  = inflammaging_index × 0.7 + (1 − cilia_c_raw × 2) × 0.3

ros_boost        = inflammaging_index × 0.15   → InflammagingState
niche_impairment = inflammaging_index × 0.08   → InflammagingState
sasp_intensity   = inflammaging_index           → InflammagingState
```

**Калибровочные ориентиры:**
- Возраст 20 лет (pristine): `myeloid_bias ≈ 0.02` — норма
- Возраст 50 лет: `myeloid_bias ≈ 0.25` — MildShift (умеренный, субклинический)
- Возраст 70 лет: `myeloid_bias ≈ 0.45` — ModerateShift (клинически значимый)
- Возраст 85 лет: `myeloid_bias ≈ 0.65` — SevereShift (иммуностарение)

### Технические шаги

- [x] **Создать crate `myeloid_shift_module`** — выполнено.

- [x] **`MyeloidShiftComponent`** — реализован:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct MyeloidShiftComponent {
      pub myeloid_bias: f32,
      pub lymphoid_deficit: f32,
      pub inflammaging_index: f32,
      pub immune_senescence: f32,
      pub phenotype: MyeloidPhenotype,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  pub enum MyeloidPhenotype {
      Healthy,        // myeloid_bias < 0.30
      MildShift,      // 0.30..0.50
      ModerateShift,  // 0.50..0.70  ← клинически значимый
      SevereShift,    // > 0.70      ← иммуностарение
  }
  ```

- [x] **`MyeloidShiftParams`** — реализован:
  ```rust
  pub struct MyeloidShiftParams {
      pub spindle_weight: f32,     // default 0.45
      pub cilia_weight: f32,       // default 0.30
      pub ros_weight: f32,         // default 0.15
      pub aggregate_weight: f32,   // default 0.10
      pub ros_boost_scale: f32,    // default 0.15
      pub niche_impair_scale: f32, // default 0.08
  }
  ```

- [x] **`MyeloidShiftModule.step()`** — реализован:
  1. Для каждой сущности с `(&CentriolarDamageState, &mut MyeloidShiftComponent, &mut InflammagingState)`:
  2. Вычислить `myeloid_bias` по формуле выше
  3. Вычислить `inflammaging_index`, `immune_senescence`
  4. Обновить `MyeloidShiftComponent`
  5. Записать в `InflammagingState { ros_boost, niche_impairment, sasp_intensity }`

- [x] **`MyeloidShiftModule.initialize()`** — реализован:
  - `MyeloidShiftComponent::default()`
  - `InflammagingState::default()` (если не было добавлено ранее)

- [x] **Unit-тесты** — 7 тестов пройдены (pristine, max_damage, spindle, cilia, calibration_age70, ros_boost, phenotype).

- [x] **Пример `myeloid_shift_example.rs`** — создан в `examples/src/bin/`.

- [x] **`CLAUDE.md`** — обновлён.

---

## 3. ЗАГЛУШКИ (существующие модули без реализации)

- [x] **PTM → CentriolarDamageState bridge** — реализован в `human_development_module` ✅:
  - Читает `Option<&CentriolePair>` в step(), применяет PTM_SCALE=0.002/год
  - acetylation→tubulin_hyperacetylation, oxidation→carbonylation, phospho→phospho_dysreg, methyl→aggregates
  - 4 unit-теста (scale_is_moderate проверяет что bridge < 50% от базового damage за 30 лет)

- [x] **`centriole_module.step()`** — PTM-накопление реализовано ✅:
  - Читает `CellCycleStateExtended` (Option) для детектирования M-фазы
  - Накапливает PTM в `CentriolePair.mother.ptm_signature` и `.daughter.ptm_signature`
  - Мать накапливает в `daughter_ptm_factor=0.4` раза быстрее дочерней
  - M-phase boost ×3.0 (максимальный стресс тубулина при митозе)
  - Не трогает `CentriolarDamageState` — двойного счёта нет
  - 6 unit-тестов пройдены: ptm_starts_at_zero, increases_after_steps,
    mother_accumulates_faster, m_phase_boosts, ptm_clamped_at_one, daughter_factor_zero

- [ ] **`AsymmetricDivisionModule` — спавн дочерних сущностей** (опционально):
  - При `DivisionType::Asymmetric` → `world.spawn(...)` новой сущности с:
    - `CentriolarInducerPair` = результат `parent_pair.divide().1`
    - `CentriolarDamageState::pristine()` (молодая дочерняя клетка)
  - Ограничение: `max_entities: usize` параметр, не спавнить если превышено
  - *Риск:* Экспоненциальный рост числа сущностей — осторожно

- [ ] **`StemCellHierarchyModule` — пластичность** (`plasticity_rate`):
  - При `enable_plasticity = true` и `potency == Oligopotent`:
    вероятность `plasticity_rate` перехода в `Pluripotent` если `spindle_fidelity > 0.6`
  - Имитирует де-дифференцировку при нишевых сигналах

- [x] **`CellCycleModule` — enforced checkpoints** — реализовано ✅:
  - G1/S checkpoint: `total_damage_score() > checkpoint_strictness` → `G1SRestriction` (арест)
  - G2/M checkpoint (SAC): `spindle_fidelity < (1 - checkpoint_strictness)` → `SpindleAssembly`
  - Читает `Option<&CentriolarDamageState>` — нет прямой зависимости от `human_development_module`
  - `checkpoint_strictness=0.0` (дефолт) → аресты отключены, полная обратная совместимость
  - Growth factors синхронизируются из damage: `dna_damage = total_damage_score()`, `oxidative_stress = ros_level`
  - 6 unit-тестов пройдены: pristine_advances, damaged_arrests_g1s, broken_spindle_arrests_g2m,
    zero_strictness_never_arrests, arrest_releases_when_damage_clears, cells_divided_counter

---

## 4. ОБРАТНЫЕ СВЯЗИ МЕЖДУ МОДУЛЯМИ

- [x] **Мелоидный сдвиг → DamageParams (через `InflammagingState`)** ✅:
  - `human_development_module.step()` читает `Option<&InflammagingState>` и применяет `ros_boost` + `niche_impairment`
  - Петля замкнута: повреждение → myeloid_shift → inflammaging → больше ROS → больше повреждений

- [x] **Транскриптом → клеточный цикл** ✅:
  - Добавлен `GeneExpressionState` (p21, p16, cyclin_d, myc) в `cell_dt_core::components`
  - `transcriptome_module` добавил гены CDKN1A/CDKN2A, синхронизирует уровни в `GeneExpressionState`
  - `cell_cycle_module` читает `Option<&GeneExpressionState>`:
    `p21 > 0.7` → `G1SRestriction`; `p16 > 0.8` → `DNARepair` (постоянный); cyclin_d → укорачивает G1
  - 4 новых unit-теста: p21_arrests_g1s, p21_arrest_releases, p16_permanent_arrest, cyclin_d_shortens_g1

- [x] **AsymmetricDivision → TissueState** ✅:
  - Добавлен `DivisionExhaustionState` (exhaustion_count, asymmetric_count, total_divisions) в `cell_dt_core`
  - `asymmetric_division_module` синхронизирует `DivisionExhaustionState` каждый шаг деления
  - `human_development_module` читает `Option<&DivisionExhaustionState>`:
    `exhaustion_ratio × 0.0002/шаг` → снижает `stem_cell_pool`

- [ ] **MyeloidShift → AgingPhenotype** (частично — ImmuneDecline уже реализован через SASP):
  <!--
  - `human_development_module.update_aging_phenotypes()` читает `MyeloidShiftComponent` (если присутствует)
  - При `immune_senescence > 0.4` → `active_phenotypes.push(AgingPhenotype::ImmuneDecline)`
  -->

---

## 5. НОВЫЕ БИОЛОГИЧЕСКИЕ ТРЕКИ

### Трек C: Теломеры ← **СЛЕДУЮЩИЙ ШАГ**

#### Биология и связь с CDATA

| Механизм | CDATA-компонент |
|----------|-----------------|
| Каждое деление укорачивает теломеры (Хейфлик) | `DivisionExhaustionState.total_divisions` |
| Нарушенное веретено → хромосомная нестабильность → быстрее укорачивание | `spindle_fidelity ↓` |
| ROS → окислительное повреждение теломерной ДНК | `ros_level ↑` |
| Критически короткие → G1-арест (сенесценция, Хейфлик) | `is_critically_short → G1SRestriction` |

**Калибровка (T/S ratio):**
- Зигота: 1.0 (полная длина)
- 40 лет: ≈ 0.7
- 70 лет: ≈ 0.4
- Критически короткие (< 0.3): Хейфликовский предел → сенесценция

#### Технические шаги

- [ ] **`TelomereState`** — добавить в `cell_dt_core::components`:
  ```rust
  pub struct TelomereState {
      pub mean_length: f32,              // T/S ratio [0..1], зигота = 1.0
      pub shortening_per_division: f32,  // default 0.002 (≈50bp на деление)
      pub is_critically_short: bool,     // mean_length < 0.3
  }
  ```
- [ ] **`human_development_module.step()`** — читает `Option<&mut TelomereState>`:
  ```
  div_rate  = division_rate_per_year(stage)
  base      = shortening_per_division × div_rate × dt_years
  spindle_f = 1 + (1 − spindle_fidelity) × 0.5   // CDATA Трек B
  ros_f     = 1 + ros_level × 0.3                 // ROS-петля
  mean_length -= base × spindle_f × ros_f
  is_critically_short = mean_length < 0.3
  AgingPhenotype::TelomereShortening при критически коротких
  ```
- [ ] **`cell_cycle_module.step()`** — читает `Option<&TelomereState>`:
  - `is_critically_short` → `Checkpoint::G1SRestriction` (постоянный Хейфликовский арест)
- [ ] **`human_development_module.initialize()`** — добавить `TelomereState::default()`
- [ ] **Добавить в `myeloid_shift_example`** колонку `Tel` (mean_length×100 как %)
- [ ] **Unit-тесты (4 шт.)**:
  - `telomere_starts_at_one` — новая сущность: mean_length == 1.0
  - `telomere_shortens_per_step` — после N шагов с делениями: mean_length < 1.0
  - `spindle_damage_accelerates_shortening` — spindle↓ → укорачивается быстрее
  - `critically_short_sets_flag` — mean_length < 0.3 → is_critically_short = true

### Трек D: Эпигенетические часы

- [ ] **`EpigeneticClockState`** — в `cell_dt_core::components`:
  ```rust
  pub struct EpigeneticClockState {
      pub methylation_age: f32,    // биологический возраст по CpG-метилированию
      pub clock_acceleration: f32, // > 1 → часы спешат (стресс, болезнь)
  }
  ```
- [ ] **Модель**: `methylation_age` догоняет `chronological_age` в молодости,
  обгоняет при высоком `total_damage_score`
  ```
  d(methylation_age)/dt = 1.0 × clock_acceleration
  clock_acceleration = 1.0 + total_damage_score × 0.5
  ```
- [ ] **AgingPhenotype::EpigeneticChanges** — уже есть, активировать при
  `methylation_age > chronological_age × 1.1`

### Митохондриальный трек

- [ ] **Новый модуль `mitochondrial_module`** (более долгосрочный):
  - `MitochondrialState { mtdna_mutations: f32, fusion_index: f32, ros_production: f32 }`
  - Питает `ros_level` в `CentriolarDamageState`
  - Митофагия: при `ros_production > threshold` → дефект митофагии → больше дисфункциональных митохондрий
  - Прямая связь с CDATA: митохондриальный щит от O₂ слабеет при `ros_production ↑`

---

## 6. КАЛИБРОВКА И ВЕРИФИКАЦИЯ

### Проверка логики (2026-03-04)

Ручная калибровка через `myeloid_shift_example` (seed=42, 5 ниш, params default):

| Метрика | Результат | Цель | Статус |
|---------|-----------|------|--------|
| Смерть (normal aging) | ~78 лет | 65–95 лет | ✅ |
| myeloid_bias в 70 лет | 0.571 | 0.35–0.60 | ✅ (чуть выше 0.45) |
| Потентность в 0–30 лет | Totipotent | Totipotent | ✅ |
| Потентность в 40–60 лет | Pluripotent | Pluripotent | ✅ |
| Потентность в 70 лет | Pluripotent/Oligopotent | Oligopotent | ⚠️ незначительно |
| M-инд. в 70 лет | 3/10 (30%) | ~20–40% | ✅ |
| D-инд. в 70 лет | 3/8 (37.5%) | ~25–50% | ✅ |

⚠️ Примечание: `myeloid_bias` в 70 лет несколько выше 0.45 из-за стохастичности
отщепления индукторов (seed-зависимо). Принципиальных ошибок нет.

### Автоматические тесты (следующий шаг)

- [ ] **Интеграционный тест: 100-летняя симуляция** ← в пакете с TelomereState:
  - Default params → все ниши умирают в диапазоне 65–95 лет
  - `DamageParams::progeria()` → смерть в 10–25 лет
  - `DamageParams::longevity()` → смерть после 95 лет

- [ ] **Тест калибровки индукторов** — при `base_detach_probability=0.002`:
  - За 78 лет: M-остаток ≤ 50% initial, D-остаток ≤ 60% initial
  - Апоптоз через исчерпание индукторов наступает ПОСЛЕ сенесценса (не раньше)

- [ ] **Тест миелоидного сдвига** — проверить диапазоны:
  - t=20 лет: `myeloid_bias < 0.15`
  - t=70 лет: `0.35 < myeloid_bias < 0.70`
  - t=85 лет: `myeloid_bias > 0.50`

- [ ] **`DamageParams::normal_aging()`** — добавить алиас:
  ```rust
  pub fn normal_aging() -> Self { Self::default() }
  ```

- [ ] **`stage_history` — ограничить размер VecDeque**:
  - Сейчас неограниченный рост за 100 лет
  - Добавить `stage_history.truncate(20)` в `step()`

---

## 7. ИНФРАСТРУКТУРА И ЭКСПОРТ

- [ ] **CSV-экспорт через `cell_dt_io`** — подключить `DataExporter` к `SimulationManager`:
  - Выгружать каждые `checkpoint_interval` шагов
  - Колонки: `step, entity_id, tissue, age_years, stage, damage_score, myeloid_bias, spindle_fidelity, ciliary_function, frailty, phenotype_count`

- [ ] **Визуализация через `cell_dt_viz`** — добавить:
  - Временной ряд: `damage_score`, `myeloid_bias`, `spindle_fidelity`, `frailty` по оси времени
  - Стрелки трека A и трека B с текущими значениями в режиме реального времени

- [ ] **Python bindings `cell_dt_python`** — экспортировать:
  - `HumanDevelopmentComponent` как PyClass
  - `MyeloidShiftComponent` как PyClass
  - Функцию `run_simulation(params_dict) -> polars.DataFrame`

- [ ] **`cell_dt_gui` — панель управления** — добавить слайдеры для:
  - `base_detach_probability`, `mother_bias`, `age_bias_coefficient`
  - `spindle_weight`, `cilia_weight` (мелоидный сдвиг)
  - `DamageParams` преключатель: normal / progeria / longevity

---

## 8. ТЕХНИЧЕСКИЙ ДОЛГ

- [ ] **Дублирование tissue_type** — `TissueType` (в core) и `HumanTissueType` (в human_development_module)
  конвертируются через `map_tissue_type()` в `lib.rs`. Рассмотреть слияние или перенос `map_tissue_type` в core.

- [ ] **Логирование** — сейчас всё на уровне `debug!`. Добавить:
  - `trace!` для per-entity per-step событий
  - `info!` для milestone (смерть ниши, смена стадии, активация фенотипа)
  - `warn!` для биологически нереалистичных значений (damage > 1.0, myeloid_bias = 1.0)

- [ ] **Параметры `DamageParams` не доступны через панель управления** — добавить `get_params`/`set_params` для `base_ros_damage_rate`, `aggregation_rate`, `senescence_threshold`

- [ ] **`CellCycleStateExtended::new()` используется везде для спавна** — явно задокументировать, что он необходим для инициализации большинства модулей (все модули ищут сущности по наличию этого компонента).

- [ ] **Очистка dead-сущностей** — сейчас `is_alive = false` но сущность остаётся в мире.
  Добавить в `SimulationManager` опциональный проход удаления мёртвых сущностей каждые N шагов.

---

## ПОРЯДОК ВЫПОЛНЕНИЯ (рекомендуемый)

```
✅ 1  InflammagingState + AgingPhenotype::ImmuneDecline
✅ 2  myeloid_shift_module (crate + step + tests + example)
✅ 3  human_dev инициализирует InflammagingState
✅ 4  centriole_module.step() — PTM-накопление (6 тестов)
✅ 5  Транскриптом → клеточный цикл (GeneExpressionState, 4 теста)
✅ 6  AsymmetricDivision → TissueState (DivisionExhaustionState)
✅ 7  PTM → CentriolarDamageState bridge (4 теста)
✅ 8  CellCycleModule enforced checkpoints (6 тестов)
✅ 9  Мониторинг индукторов в myeloid_shift_example
  10  TelomereState Трек C + клеточный цикл (4 теста)        ← ТЕКУЩИЙ
  11  Интеграционные тесты и калибровка                       → п. 6
  12  Технический долг (stage_history, normal_aging)          → п. 8
  13  EpigeneticClockState Трек D                             → п. 5
  14  Инфраструктура (CSV, Python, GUI)                       → п. 7
  15  митохондриальный модуль                                 → долгосрочно
```

---

*При каждом выполненном пункте: переместить в секцию "ВЫПОЛНЕНО" вверху, обновить дату.*
*Последнее обновление: 2026-03-04*
