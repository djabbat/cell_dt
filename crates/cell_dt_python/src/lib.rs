//! Python биндинги для Cell DT платформы

// pyo3 attr-макросы генерируют impl-блоки вне модуля — известная особенность pyo3 0.20.
// Исправляется обновлением до pyo3 ≥ 0.21. Подавляем до миграции.
#![allow(non_local_definitions)]

use cell_dt_core::{
    SimulationManager, SimulationConfig,
    components::*,
};
use centriole_module::CentrioleModule;
use cell_cycle_module::{CellCycleModule, CellCycleParams};
use transcriptome_module::{TranscriptomeModule, TranscriptomeParams};
use pyo3::prelude::*;
use pyo3::types::{PyDict};
use numpy::{PyArray1, PyArray2};
use std::collections::HashMap;

/// Модуль Python
#[pymodule]
fn cell_dt(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PySimulation>()?;
    m.add_class::<PyCellData>()?;
    m.add_class::<PyCentrioleData>()?;
    m.add_class::<PyCellCycleData>()?;
    m.add_class::<PyTranscriptomeData>()?;
    m.add_class::<PyCellCycleParams>()?;
    
    m.add_function(wrap_pyfunction!(run_simulation, m)?)?;
    m.add_function(wrap_pyfunction!(create_cell_population, m)?)?;
    m.add_function(wrap_pyfunction!(analyze_transcriptome, m)?)?;
    
    Ok(())
}

/// Данные центриоли для Python
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyCentrioleData {
    #[pyo3(get)]
    mother_maturity: f32,
    #[pyo3(get)]
    daughter_maturity: f32,
    #[pyo3(get)]
    mtoc_activity: f32,
    #[pyo3(get)]
    cilium_present: bool,
    #[pyo3(get)]
    acetylation_level: f32,
    #[pyo3(get)]
    oxidation_level: f32,
    #[pyo3(get)]
    cafd_count: usize,
}

impl From<&CentriolePair> for PyCentrioleData {
    fn from(centriole: &CentriolePair) -> Self {
        Self {
            mother_maturity: centriole.mother.maturity,
            daughter_maturity: centriole.daughter.maturity,
            mtoc_activity: centriole.mtoc_activity,
            cilium_present: centriole.cilium_present,
            acetylation_level: centriole.mother.ptm_signature.acetylation_level,
            oxidation_level: centriole.mother.ptm_signature.oxidation_level,
            cafd_count: centriole.mother.associated_cafds.len(),
        }
    }
}

/// Данные клеточного цикла для Python
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyCellCycleData {
    #[pyo3(get)]
    phase: String,
    #[pyo3(get)]
    progress: f32,
    #[pyo3(get)]
    cycle_count: u32,
    #[pyo3(get)]
    checkpoint: Option<String>,
    #[pyo3(get)]
    growth_signal: f32,
    #[pyo3(get)]
    stress_level: f32,
    #[pyo3(get)]
    dna_damage: f32,
}

impl From<&CellCycleStateExtended> for PyCellCycleData {
    fn from(cycle: &CellCycleStateExtended) -> Self {
        Self {
            phase: format!("{:?}", cycle.phase),
            progress: cycle.progress,
            cycle_count: cycle.cycle_count,
            checkpoint: cycle.current_checkpoint.map(|c| format!("{:?}", c)),
            growth_signal: cycle.growth_factors.growth_signal,
            stress_level: cycle.growth_factors.stress_level,
            dna_damage: cycle.growth_factors.dna_damage,
        }
    }
}

/// Данные транскриптома для Python
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyTranscriptomeData {
    #[pyo3(get)]
    expressed_genes: Vec<String>,
    #[pyo3(get)]
    total_expression: f32,
    #[pyo3(get)]
    active_pathways: usize,
    #[pyo3(get)]
    cell_type: String,
    #[pyo3(get)]
    is_stem_cell: bool,
}

impl From<&transcriptome_module::TranscriptomeState> for PyTranscriptomeData {
    fn from(t: &transcriptome_module::TranscriptomeState) -> Self {
        Self {
            expressed_genes: t.expressed_genes.iter().cloned().collect(),
            total_expression: t.total_expression,
            active_pathways: t.active_pathways,
            cell_type: t.get_cell_type(),
            is_stem_cell: t.is_stem_cell(),
        }
    }
}

/// Данные одной клетки для Python
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyCellData {
    #[pyo3(get)]
    cell_id: u64,
    #[pyo3(get)]
    centriole: PyCentrioleData,
    #[pyo3(get)]
    cell_cycle: PyCellCycleData,
    #[pyo3(get)]
    transcriptome: Option<PyTranscriptomeData>,
}

/// Python класс для управления симуляцией
#[pyclass]
pub struct PySimulation {
    sim: SimulationManager,
    cell_count: usize,
}

#[pymethods]
impl PySimulation {
    #[new]
    pub fn new(max_steps: u64, dt: f64, num_threads: Option<usize>, seed: Option<u64>) -> Self {
        let config = SimulationConfig {
            max_steps,
            dt,
            checkpoint_interval: 100,
            num_threads,
            seed,
            parallel_modules: false,
        };
        
        Self {
            sim: SimulationManager::new(config),
            cell_count: 0,
        }
    }
    
    /// Создать популяцию клеток
    pub fn create_population(&mut self, count: usize) -> PyResult<()> {
        let world = self.sim.world_mut();
        
        for _ in 0..count {
            let _ = world.spawn((
                CentriolePair::default(),
                CellCycleStateExtended::new(),
            ));
        }
        
        self.cell_count = count;
        Ok(())
    }
    
    /// Создать популяцию с транскриптомом
    pub fn create_population_with_transcriptome(&mut self, count: usize) -> PyResult<()> {
        let world = self.sim.world_mut();
        
        for _ in 0..count {
            let _ = world.spawn((
                CentriolePair::default(),
                CellCycleStateExtended::new(),
            ));
        }
        
        self.cell_count = count;
        Ok(())
    }
    
    /// Зарегистрировать модули
    pub fn register_modules(
        &mut self,
        enable_centriole: bool,
        enable_cell_cycle: bool,
        enable_transcriptome: bool,
        cell_cycle_params: Option<PyCellCycleParams>,
    ) -> PyResult<()> {
        if enable_centriole {
            let module = CentrioleModule::with_parallel(true);
            self.sim.register_module(Box::new(module))
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        }
        
        if enable_cell_cycle {
            let params = cell_cycle_params.unwrap_or_default().into();
            let module = CellCycleModule::with_params(params);
            self.sim.register_module(Box::new(module))
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        }
        
        if enable_transcriptome {
            let params = TranscriptomeParams::default();
            let module = TranscriptomeModule::with_params(params);
            self.sim.register_module(Box::new(module))
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        }
        
        Ok(())
    }
    
    /// Запустить симуляцию
    pub fn run(&mut self) -> PyResult<Vec<PyCellData>> {
        self.sim.initialize()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        
        self.sim.run()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        
        Ok(self.get_cell_data())
    }
    
    /// Запустить симуляцию пошагово
    pub fn step(&mut self, steps: u64) -> PyResult<Vec<PyCellData>> {
        for _ in 0..steps {
            self.sim.step()
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        }
        
        Ok(self.get_cell_data())
    }
    
    /// Получить данные всех клеток
    pub fn get_cell_data(&self) -> Vec<PyCellData> {
        let world = self.sim.world();
        let mut query = world.query::<(
            &CentriolePair,
            &CellCycleStateExtended,
            Option<&transcriptome_module::TranscriptomeState>,
        )>();
        
        query.iter()
            .map(|(entity, (centriole, cell_cycle, transcriptome))| {
                PyCellData {
                    cell_id: entity.to_bits().get(),
                    centriole: PyCentrioleData::from(centriole),
                    cell_cycle: PyCellCycleData::from(cell_cycle),
                    transcriptome: transcriptome.map(PyTranscriptomeData::from),
                }
            })
            .collect()
    }
    
    /// Получить данные центриолей как NumPy массив
    pub fn get_centriole_data_numpy(&self, py: Python) -> PyResult<Py<PyArray2<f32>>> {
        let cells = self.get_cell_data();
        let mut data = Vec::new();
        
        for cell in cells {
            data.push(vec![
                cell.centriole.mother_maturity,
                cell.centriole.daughter_maturity,
                cell.centriole.mtoc_activity,
                cell.centriole.acetylation_level,
                cell.centriole.oxidation_level,
            ]);
        }
        
        let array = PyArray2::from_vec2(py, &data)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(array.to_owned())
    }
    
    /// Получить распределение фаз клеточного цикла
    pub fn get_phase_distribution(&self, py: Python) -> PyResult<Py<PyDict>> {
        let cells = self.get_cell_data();
        let mut phase_counts = HashMap::new();
        
        for cell in cells {
            *phase_counts.entry(cell.cell_cycle.phase).or_insert(0) += 1;
        }
        
        let dict = PyDict::new(py);
        for (phase, count) in phase_counts {
            dict.set_item(phase, count)?;
        }
        
        Ok(dict.into())
    }
    
    /// Получить временной ряд экспрессии генов (заглушка)
    pub fn get_expression_history(&self, py: Python, _gene: &str) -> PyResult<Py<PyArray1<f32>>> {
        let empty: Vec<f32> = Vec::new();
        Ok(PyArray1::from_vec(py, empty).to_owned())
    }
    
    /// Сохранить состояние симуляции (заглушка)
    pub fn save_checkpoint(&self, _path: &str) -> PyResult<()> {
        Ok(())
    }
    
    /// Загрузить состояние симуляции (заглушка)
    pub fn load_checkpoint(&mut self, _path: &str) -> PyResult<()> {
        Ok(())
    }
    
    /// Получить текущий шаг
    pub fn current_step(&self) -> u64 {
        self.sim.current_step()
    }
    
    /// Получить текущее время
    pub fn current_time(&self) -> f64 {
        self.sim.current_time()
    }
    
    /// Получить количество клеток
    pub fn cell_count(&self) -> usize {
        self.cell_count
    }
}

/// Параметры клеточного цикла для Python
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyCellCycleParams {
    #[pyo3(get, set)]
    base_cycle_time: f32,
    #[pyo3(get, set)]
    growth_factor_sensitivity: f32,
    #[pyo3(get, set)]
    stress_sensitivity: f32,
    #[pyo3(get, set)]
    checkpoint_strictness: f32,
    #[pyo3(get, set)]
    enable_apoptosis: bool,
    #[pyo3(get, set)]
    nutrient_availability: f32,
    #[pyo3(get, set)]
    growth_factor_level: f32,
    #[pyo3(get, set)]
    random_variation: f32,
}

#[pymethods]
impl PyCellCycleParams {
    #[new]
    #[pyo3(signature = (
        base_cycle_time = 24.0,
        growth_factor_sensitivity = 0.3,
        stress_sensitivity = 0.2,
        checkpoint_strictness = 0.1,
        enable_apoptosis = true,
        nutrient_availability = 0.9,
        growth_factor_level = 0.8,
        random_variation = 0.2,
    ))]
    pub fn new(
        base_cycle_time: f32,
        growth_factor_sensitivity: f32,
        stress_sensitivity: f32,
        checkpoint_strictness: f32,
        enable_apoptosis: bool,
        nutrient_availability: f32,
        growth_factor_level: f32,
        random_variation: f32,
    ) -> Self {
        Self {
            base_cycle_time,
            growth_factor_sensitivity,
            stress_sensitivity,
            checkpoint_strictness,
            enable_apoptosis,
            nutrient_availability,
            growth_factor_level,
            random_variation,
        }
    }
}

impl Default for PyCellCycleParams {
    fn default() -> Self {
        Self::new(24.0, 0.3, 0.2, 0.1, true, 0.9, 0.8, 0.2)
    }
}

impl From<PyCellCycleParams> for CellCycleParams {
    fn from(py_params: PyCellCycleParams) -> Self {
        CellCycleParams {
            base_cycle_time: py_params.base_cycle_time,
            growth_factor_sensitivity: py_params.growth_factor_sensitivity,
            stress_sensitivity: py_params.stress_sensitivity,
            checkpoint_strictness: py_params.checkpoint_strictness,
            enable_apoptosis: py_params.enable_apoptosis,
            nutrient_availability: py_params.nutrient_availability,
            growth_factor_level: py_params.growth_factor_level,
            random_variation: py_params.random_variation,
        }
    }
}

/// Функция для быстрого запуска симуляции из Python
#[pyfunction]
#[pyo3(signature = (
    num_cells = 100,
    steps = 1000,
    dt = 0.1,
    enable_centriole = true,
    enable_cell_cycle = true,
    enable_transcriptome = true,
))]
pub fn run_simulation(
    num_cells: usize,
    steps: u64,
    dt: f64,
    enable_centriole: bool,
    enable_cell_cycle: bool,
    enable_transcriptome: bool,
) -> PyResult<Vec<PyCellData>> {
    let mut sim = PySimulation::new(steps, dt, Some(4), Some(42));
    
    if enable_transcriptome {
        sim.create_population_with_transcriptome(num_cells)?;
    } else {
        sim.create_population(num_cells)?;
    }
    
    let params = PyCellCycleParams::default();
    sim.register_modules(
        enable_centriole,
        enable_cell_cycle,
        enable_transcriptome,
        Some(params),
    )?;
    
    sim.run()
}

/// Создать популяцию клеток с заданными параметрами
#[pyfunction]
pub fn create_cell_population(
    count: usize,
    _phases: Vec<String>,
) -> PyResult<Vec<PyCellData>> {
    let mut sim = PySimulation::new(1, 0.1, Some(1), Some(42));
    sim.create_population(count)?;
    
    Ok(sim.get_cell_data())
}

/// Анализ транскриптома
#[pyfunction]
pub fn analyze_transcriptome(cell_data: Vec<PyCellData>) -> PyResult<HashMap<String, f32>> {
    let mut stats = HashMap::new();
    let mut total_cells = 0;
    let mut stem_cells = 0;
    
    for cell in cell_data {
        total_cells += 1;
        if let Some(t) = cell.transcriptome {
            if t.is_stem_cell {
                stem_cells += 1;
            }
        }
    }
    
    stats.insert("total_cells".to_string(), total_cells as f32);
    stats.insert("stem_cells".to_string(), stem_cells as f32);
    if total_cells > 0 {
        stats.insert("stem_cell_ratio".to_string(), 
                    (stem_cells as f32) / (total_cells as f32));
    }
    
    Ok(stats)
}
