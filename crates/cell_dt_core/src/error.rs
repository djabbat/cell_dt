use thiserror::Error;

#[derive(Error, Debug)]
pub enum SimulationError {
    #[error("Module error: {0}")]
    ModuleError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Entity not found")]
    EntityNotFound,
    
    #[error("Component not found")]
    ComponentNotFound,
}

pub type SimulationResult<T> = Result<T, SimulationError>;
