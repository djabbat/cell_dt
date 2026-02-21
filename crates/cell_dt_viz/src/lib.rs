//! Модуль визуализации для Cell DT платформы
//! Поддерживает 2D графики через plotters и 3D визуализацию через kiss3d

mod plot2d;
mod plot3d;
mod heatmap;
mod timeseries;

pub use plot2d::*;
pub use plot3d::*;
pub use heatmap::*;
pub use timeseries::*;

use cell_dt_core::{
    components::{CentriolePair, CellCycleState, Phase},
    hecs::World,
};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use parking_lot::Mutex;

/// Типы визуализации
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualizationType {
    Heatmap,
    ScatterPlot,
    TimeSeries,
    Histogram,
    PieChart,
    BarChart,
    Network,
    Dendrogram,
}

/// Данные для визуализации
#[derive(Debug, Clone)]
pub struct VisualizationData {
    pub step: u64,
    pub time: f64,
    pub cell_count: usize,
    pub phase_distribution: HashMap<Phase, usize>,
    pub centriole_maturity: Vec<f32>,
    pub mtoc_activity: Vec<f32>,
    pub cafd_counts: Vec<usize>,
    pub cilia_count: usize,
}

impl VisualizationData {
    pub fn from_world(world: &World) -> Self {
        let mut query = world.query::<(&CentriolePair, &CellCycleState)>();
        
        let mut phase_distribution = HashMap::new();
        let mut centriole_maturity = Vec::new();
        let mut mtoc_activity = Vec::new();
        let mut cafd_counts = Vec::new();
        let mut cilia_count = 0;
        let mut cell_count = 0;
        
        for (_, (pair, cycle)) in query.iter() {
            cell_count += 1;
            
            *phase_distribution.entry(cycle.phase).or_insert(0) += 1;
            
            centriole_maturity.push(pair.mother.maturity);
            centriole_maturity.push(pair.daughter.maturity);
            
            mtoc_activity.push(pair.mtoc_activity);
            
            cafd_counts.push(pair.mother.associated_cafds.len());
            
            if pair.cilium_present {
                cilia_count += 1;
            }
        }
        
        VisualizationData {
            step: 0,
            time: 0.0,
            cell_count,
            phase_distribution,
            centriole_maturity,
            mtoc_activity,
            cafd_counts,
            cilia_count,
        }
    }
}

/// Менеджер визуализации
pub struct VisualizationManager {
    pub data_history: Arc<Mutex<VecDeque<VisualizationData>>>,
    active_viz: Vec<Box<dyn Visualizer + Send>>,
    update_interval: u64,
    last_update: u64,
}

impl VisualizationManager {
    pub fn new(update_interval: u64) -> Self {
        Self {
            data_history: Arc::new(Mutex::new(VecDeque::new())),
            active_viz: Vec::new(),
            update_interval,
            last_update: 0,
        }
    }
    
    pub fn add_visualizer(&mut self, visualizer: Box<dyn Visualizer + Send>) {
        self.active_viz.push(visualizer);
    }
    
    pub fn update(&mut self, world: &World, step: u64, time: f64) -> Result<(), Box<dyn std::error::Error>> {
        if step - self.last_update < self.update_interval {
            return Ok(());
        }
        
        let mut data = VisualizationData::from_world(world);
        data.step = step;
        data.time = time;
        
        {
            let mut history = self.data_history.lock();
            history.push_back(data.clone());
            if history.len() > 1000 {
                history.pop_front();
            }
        }
        
        for viz in self.active_viz.iter_mut() {
            viz.update(&data)?;
        }
        
        self.last_update = step;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cell_dt_core::hecs::World;

    // ==================== VisualizationData ====================

    #[test]
    fn test_visualization_data_from_empty_world() {
        let world = World::new();
        let data = VisualizationData::from_world(&world);
        assert_eq!(data.cell_count, 0);
        assert_eq!(data.cilia_count, 0);
        assert!(data.phase_distribution.is_empty());
        assert!(data.centriole_maturity.is_empty());
        assert!(data.mtoc_activity.is_empty());
    }

    #[test]
    fn test_visualization_data_fields_default() {
        let world = World::new();
        let data = VisualizationData::from_world(&world);
        assert_eq!(data.step, 0);
        assert_eq!(data.time, 0.0);
    }

    // ==================== VisualizationManager ====================

    #[test]
    fn test_manager_new_history_empty() {
        let manager = VisualizationManager::new(10);
        let history = manager.data_history.lock();
        assert!(history.is_empty());
    }

    #[test]
    fn test_manager_skips_update_before_interval() {
        let mut manager = VisualizationManager::new(5);
        let world = World::new();

        // Steps 0..4 are all below the interval threshold
        for i in 0..4u64 {
            manager.update(&world, i, i as f64 * 0.1).unwrap();
        }

        let history = manager.data_history.lock();
        assert!(history.is_empty(), "history should be empty before interval is reached");
    }

    #[test]
    fn test_manager_records_data_at_interval() {
        let mut manager = VisualizationManager::new(5);
        let world = World::new();

        // Step 5 crosses the interval (5 - 0 = 5, not < 5)
        manager.update(&world, 5, 0.5).unwrap();

        let history = manager.data_history.lock();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].step, 5);
    }

    #[test]
    fn test_manager_history_bounded_at_1000() {
        let mut manager = VisualizationManager::new(1);
        let world = World::new();

        for i in 0..1010u64 {
            manager.update(&world, i, i as f64 * 0.1).unwrap();
        }

        let history = manager.data_history.lock();
        assert!(history.len() <= 1000, "history exceeded 1000 entries: {}", history.len());
    }

    #[test]
    fn test_manager_data_history_arc_is_shared() {
        let manager = VisualizationManager::new(1);
        let arc1 = manager.data_history.clone();
        let arc2 = manager.data_history.clone();

        arc1.lock().push_back(VisualizationData {
            step: 42,
            time: 1.0,
            cell_count: 5,
            phase_distribution: HashMap::new(),
            centriole_maturity: vec![],
            mtoc_activity: vec![],
            cafd_counts: vec![],
            cilia_count: 2,
        });

        assert_eq!(arc2.lock().len(), 1);
        assert_eq!(arc2.lock()[0].step, 42);
    }
}

/// Трейт для визуализаторов
pub trait Visualizer {
    fn name(&self) -> &str;
    fn update(&mut self, data: &VisualizationData) -> Result<(), Box<dyn std::error::Error>>;
    fn save_snapshot(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>>;
}
