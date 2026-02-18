use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Phase {
    G1,
    S,
    G2,
    M,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Position {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneExpression {
    pub profile: HashMap<String, f32>,
}

impl Default for GeneExpression {
    fn default() -> Self {
        Self { profile: HashMap::new() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellCycleState {
    pub phase: Phase,
    pub progress: f32,
}

impl Default for CellCycleState {
    fn default() -> Self {
        Self { phase: Phase::G1, progress: 0.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PTMProfile {
    pub acetylation_level: f32,
    pub oxidation_level: f32,
    pub methylation_level: f32,
    pub phosphorylation_level: f32,
}

impl Default for PTMProfile {
    fn default() -> Self {
        Self {
            acetylation_level: 0.0,
            oxidation_level: 0.0,
            methylation_level: 0.0,
            phosphorylation_level: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CAFD {
    pub name: String,
    pub activity: f32,
    pub concentration: f32,
}

impl CAFD {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            activity: 0.0,
            concentration: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Centriole {
    pub maturity: f32,
    pub ptm_signature: PTMProfile,
    pub associated_cafds: Vec<CAFD>,
}

impl Centriole {
    pub fn new(maturity: f32) -> Self {
        Self {
            maturity,
            ptm_signature: PTMProfile::default(),
            associated_cafds: Vec::new(),
        }
    }
    
    pub fn new_daughter() -> Self {
        Self::new(0.0)
    }
    
    pub fn new_mature() -> Self {
        Self::new(1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentriolePair {
    pub mother: Centriole,
    pub daughter: Centriole,
    pub cilium_present: bool,
    pub mtoc_activity: f32,
}

impl Default for CentriolePair {
    fn default() -> Self {
        Self {
            mother: Centriole::new_mature(),
            daughter: Centriole::new_daughter(),
            cilium_present: false,
            mtoc_activity: 0.5,
        }
    }
}
