//! Модуль ввода/вывода данных для Cell DT платформы

mod csv_exporter;
mod config;

pub use csv_exporter::*;
pub use config::*;

use cell_dt_core::{
    components::*,
    hecs::World,
};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Ошибки ввода/вывода
#[derive(Error, Debug)]
pub enum IoError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Empty buffer: {0}")]
    EmptyBuffer(&'static str),
}

/// Результат операций ввода/вывода
pub type IoResult<T> = Result<T, IoError>;

/// Данные одной клетки для экспорта
#[derive(Debug, Clone)]
pub struct CellData {
    pub cell_id: u64,
    pub step: u64,
    pub time: f64,
    pub mother_maturity: f32,
    pub daughter_maturity: f32,
    pub mtoc_activity: f32,
    pub cilium_present: bool,
    pub phase: String,
    pub cycle_progress: f32,
    pub cycle_count: u32,
    pub growth_signal: f32,
    pub stress_level: f32,
}

impl CellData {
    pub fn from_components(
        cell_id: u64,
        step: u64,
        time: f64,
        centriole: &CentriolePair,
        cell_cycle: &CellCycleStateExtended,
    ) -> Self {
        Self {
            cell_id,
            step,
            time,
            mother_maturity: centriole.mother.maturity,
            daughter_maturity: centriole.daughter.maturity,
            mtoc_activity: centriole.mtoc_activity,
            cilium_present: centriole.cilium_present,
            phase: format!("{:?}", cell_cycle.phase),
            cycle_progress: cell_cycle.progress,
            cycle_count: cell_cycle.cycle_count,
            growth_signal: cell_cycle.growth_factors.growth_signal,
            stress_level: cell_cycle.growth_factors.stress_level,
        }
    }
    
    pub fn csv_headers() -> Vec<String> {
        vec![
            "cell_id".to_string(),
            "step".to_string(),
            "time".to_string(),
            "mother_maturity".to_string(),
            "daughter_maturity".to_string(),
            "mtoc_activity".to_string(),
            "cilium_present".to_string(),
            "phase".to_string(),
            "cycle_progress".to_string(),
            "cycle_count".to_string(),
            "growth_signal".to_string(),
            "stress_level".to_string(),
        ]
    }
    
    pub fn to_csv_record(&self) -> Vec<String> {
        vec![
            self.cell_id.to_string(),
            self.step.to_string(),
            format!("{:.6}", self.time),
            format!("{:.6}", self.mother_maturity),
            format!("{:.6}", self.daughter_maturity),
            format!("{:.6}", self.mtoc_activity),
            (self.cilium_present as u8).to_string(),
            self.phase.clone(),
            format!("{:.6}", self.cycle_progress),
            self.cycle_count.to_string(),
            format!("{:.6}", self.growth_signal),
            format!("{:.6}", self.stress_level),
        ]
    }
}

/// Менеджер экспорта данных
pub struct DataExporter {
    output_dir: PathBuf,
    prefix: String,
    buffer: Vec<CellData>,
}

impl DataExporter {
    pub fn new(output_dir: impl AsRef<Path>, prefix: &str) -> Self {
        let output_dir = output_dir.as_ref().to_path_buf();
        let _ = std::fs::create_dir_all(&output_dir);
        
        Self {
            output_dir,
            prefix: prefix.to_string(),
            buffer: Vec::new(),
        }
    }
    
    pub fn collect_data(&mut self, world: &World, step: u64, time: f64) -> IoResult<()> {
        let mut query = world.query::<(&CentriolePair, &CellCycleStateExtended)>();
        
        for (entity, (centriole, cell_cycle)) in query.iter() {
            let cell_id = entity.to_bits().get();
            
            let cell_data = CellData::from_components(
                cell_id,
                step,
                time,
                centriole,
                cell_cycle,
            );
            
            self.buffer.push(cell_data);
        }
        
        Ok(())
    }
    
    pub fn save_snapshot(&mut self, step: u64) -> IoResult<PathBuf> {
        if self.buffer.is_empty() {
            return Err(IoError::EmptyBuffer("no data collected for this snapshot"));
        }
        
        let csv_path = self.output_dir.join(format!(
            "{}_step_{:06}.csv",
            self.prefix, step
        ));
        
        csv_exporter::write_csv(&csv_path, &self.buffer)?;
        self.buffer.clear();
        
        Ok(csv_path)
    }
    
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cell_data(id: u64) -> CellData {
        CellData {
            cell_id: id,
            step: 1,
            time: 0.1,
            mother_maturity: 0.9,
            daughter_maturity: 0.4,
            mtoc_activity: 0.7,
            cilium_present: true,
            phase: "G1".to_string(),
            cycle_progress: 0.5,
            cycle_count: 1,
            growth_signal: 0.6,
            stress_level: 0.1,
        }
    }

    // ==================== CellData ====================

    #[test]
    fn test_csv_headers_count() {
        assert_eq!(CellData::csv_headers().len(), 12);
    }

    #[test]
    fn test_csv_record_count_matches_headers() {
        let data = make_cell_data(1);
        assert_eq!(data.to_csv_record().len(), CellData::csv_headers().len());
    }

    #[test]
    fn test_csv_record_values() {
        let data = make_cell_data(42);
        let record = data.to_csv_record();
        assert_eq!(record[0], "42");       // cell_id
        assert_eq!(record[1], "1");        // step
        assert_eq!(record[7], "G1");       // phase
        assert_eq!(record[9], "1");        // cycle_count
        assert_eq!(record[6], "1");        // cilium_present → 1
    }

    #[test]
    fn test_csv_record_cilium_false() {
        let mut data = make_cell_data(1);
        data.cilium_present = false;
        let record = data.to_csv_record();
        assert_eq!(record[6], "0");
    }

    // ==================== IoError ====================

    #[test]
    fn test_io_error_empty_buffer_display() {
        let err = IoError::EmptyBuffer("nothing to save");
        let msg = format!("{}", err);
        assert!(msg.contains("nothing to save"));
    }

    // ==================== DataExporter ====================

    #[test]
    fn test_save_snapshot_empty_buffer_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let mut exporter = DataExporter::new(dir.path(), "test");
        let result = exporter.save_snapshot(0);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IoError::EmptyBuffer(_)));
    }

    #[test]
    fn test_data_exporter_creates_nested_directory() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a").join("b").join("c");
        let _ = DataExporter::new(&nested, "test");
        assert!(nested.exists());
    }

    #[test]
    fn test_save_snapshot_writes_csv() {
        let dir = tempfile::tempdir().unwrap();
        let mut exporter = DataExporter::new(dir.path(), "cells");
        exporter.buffer.push(make_cell_data(7));
        exporter.buffer.push(make_cell_data(8));

        let path = exporter.save_snapshot(5).unwrap();
        assert!(path.exists());

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("cell_id"));  // header row
        assert!(content.contains("7"));
        assert!(content.contains("8"));
    }

    #[test]
    fn test_save_snapshot_clears_buffer() {
        let dir = tempfile::tempdir().unwrap();
        let mut exporter = DataExporter::new(dir.path(), "cells");
        exporter.buffer.push(make_cell_data(1));
        exporter.save_snapshot(0).unwrap();
        assert!(exporter.buffer.is_empty());
    }

    #[test]
    fn test_clear_empties_buffer() {
        let dir = tempfile::tempdir().unwrap();
        let mut exporter = DataExporter::new(dir.path(), "cells");
        exporter.buffer.push(make_cell_data(1));
        exporter.buffer.push(make_cell_data(2));
        exporter.clear();
        assert!(exporter.buffer.is_empty());
    }

    // ==================== csv_exporter ====================

    #[test]
    fn test_write_csv_creates_file_with_header() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.csv");
        let cells = vec![make_cell_data(99)];

        csv_exporter::write_csv(&path, &cells).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("cell_id"));
        assert!(content.contains("99"));
    }

    #[test]
    fn test_write_csv_empty_cells_writes_only_header() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.csv");

        csv_exporter::write_csv(&path, &[]).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("cell_id"));
        // Only one line (header)
        assert_eq!(content.lines().count(), 1);
    }
}
