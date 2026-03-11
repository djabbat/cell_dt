use crate::{SimulationResult, hecs::World};
use serde_json::Value;

// ---------------------------------------------------------------------------
// P12: Трейт автоматического CSV-экспорта
// ---------------------------------------------------------------------------

/// Трейт для автоматического сбора и записи CDATA-данных через [`SimulationManager`].
///
/// Имплементируется в `cell_dt_io::CdataExporter`. Хранится в `SimulationManager`
/// и вызывается автоматически каждые N шагов.
///
/// # Пример (в `cell_dt_io`)
/// ```ignore
/// impl CdataCollect for CdataExporter {
///     fn collect(&mut self, world: &World, step: u64) { ... }
///     fn write_csv(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> { ... }
/// }
/// ```
pub trait CdataCollect: Send {
    /// Собрать снимок данных из ECS-мира на шаге `step`.
    fn collect(&mut self, world: &World, step: u64);
    /// Записать все собранные данные в CSV-файл.
    fn write_csv(&self, path: &str) -> Result<(), Box<dyn std::error::Error>>;
    /// Число записей в буфере.
    fn buffered(&self) -> usize;
}

pub trait SimulationModule: Send + Sync {
    fn name(&self) -> &str;
    fn step(&mut self, world: &mut World, dt: f64) -> SimulationResult<()>;
    fn get_params(&self) -> Value;
    fn set_params(&mut self, params: &Value) -> SimulationResult<()>;

    fn initialize(&mut self, _world: &mut World) -> SimulationResult<()> {
        Ok(())
    }

    fn cleanup(&mut self) -> SimulationResult<()> {
        Ok(())
    }

    /// Установить случайный seed для воспроизводимости симуляций.
    ///
    /// Вызывается [`SimulationManager::initialize()`] перед [`initialize()`].
    /// Модули с рандомностью должны переопределить этот метод и сохранить seed
    /// для использования в [`step()`] вместо `rand::thread_rng()`.
    ///
    /// По умолчанию — no-op (модуль не использует RNG или использует thread_rng).
    fn set_seed(&mut self, _seed: u64) {}
}
