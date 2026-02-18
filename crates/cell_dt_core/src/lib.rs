//! Ядро платформы симуляции клеточной дифференцировки

pub mod components;
pub mod error;
pub mod module;
pub mod simulation;
pub mod world;

pub use components::*;
pub use error::*;
pub use module::*;
pub use simulation::*;
pub use world::*;

pub use hecs;
