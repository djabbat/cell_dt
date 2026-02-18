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

// Типы для расширенного клеточного цикла
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CyclinType {
    CyclinD,
    CyclinE,
    CyclinA,
    CyclinB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CdkType {
    Cdk4,
    Cdk6,
    Cdk2,
    Cdk1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Checkpoint {
    G1SRestriction,
    G2MCheckpoint,
    SpindleAssembly,
    DNARepair,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyclinCdkComplex {
    pub cyclin_type: CyclinType,
    pub cdk_type: CdkType,
    pub activity: f32,
    pub concentration: f32,
    pub phosphorylation_level: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthFactors {
    pub growth_signal: f32,
    pub nutrient_level: f32,
    pub stress_level: f32,
    pub dna_damage: f32,
    pub oxidative_stress: f32,
}

impl Default for GrowthFactors {
    fn default() -> Self {
        Self {
            growth_signal: 0.8,
            nutrient_level: 1.0,
            stress_level: 0.0,
            dna_damage: 0.0,
            oxidative_stress: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointState {
    pub checkpoint: Checkpoint,
    pub satisfied: bool,
    pub time_in_checkpoint: f32,
    pub arrest_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellCycleStateExtended {
    pub phase: Phase,
    pub progress: f32,
    pub cyclin_cdk_complexes: Vec<CyclinCdkComplex>,
    pub checkpoints: Vec<CheckpointState>,
    pub current_checkpoint: Option<Checkpoint>,
    pub growth_factors: GrowthFactors,
    pub cycle_count: u32,
    pub time_in_current_phase: f32,
    pub total_time: f32,
    pub centriole_influence: f32,
}

impl CellCycleStateExtended {
    pub fn new() -> Self {
        Self {
            phase: Phase::G1,
            progress: 0.0,
            cyclin_cdk_complexes: Vec::new(),
            checkpoints: Vec::new(),
            current_checkpoint: None,
            growth_factors: GrowthFactors::default(),
            cycle_count: 0,
            time_in_current_phase: 0.0,
            total_time: 0.0,
            centriole_influence: 0.0,
        }
    }
}

impl Default for CellCycleStateExtended {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_enum() {
        let phases = vec![Phase::G1, Phase::S, Phase::G2, Phase::M];
        assert_eq!(phases.len(), 4);
    }

    #[test]
    fn test_centriole_creation() {
        let mother = Centriole::new_mature();
        let daughter = Centriole::new_daughter();
        
        assert_eq!(mother.maturity, 1.0);
        assert_eq!(daughter.maturity, 0.0);
        assert_eq!(mother.associated_cafds.len(), 0);
    }

    #[test]
    fn test_centriole_pair_default() {
        let pair = CentriolePair::default();
        assert_eq!(pair.mother.maturity, 1.0);
        assert_eq!(pair.daughter.maturity, 0.0);
        assert_eq!(pair.mtoc_activity, 0.5);
        assert!(!pair.cilium_present);
    }

    #[test]
    fn test_cafd_creation() {
        let cafd = CAFD::new("YAP");
        assert_eq!(cafd.name, "YAP");
        assert_eq!(cafd.activity, 0.0);
        assert_eq!(cafd.concentration, 0.0);
    }

    #[test]
    fn test_ptm_profile_default() {
        let ptm = PTMProfile::default();
        assert_eq!(ptm.acetylation_level, 0.0);
        assert_eq!(ptm.oxidation_level, 0.0);
    }
}

impl CellCycleStateExtended {
    /// Получить активность конкретного комплекса
    pub fn get_complex_activity(&self, cyclin_type: CyclinType, cdk_type: CdkType) -> f32 {
        for complex in &self.cyclin_cdk_complexes {
            if complex.cyclin_type == cyclin_type && complex.cdk_type == cdk_type {
                return complex.activity;
            }
        }
        0.0
    }
    
    /// Учет влияния центриоли (заглушка)
    pub fn apply_centriole_influence(&mut self, _centriole: &CentriolePair) {
        // Будет реализовано позже
    }
    
    /// Обновление циклинов (заглушка)
    pub fn update_cyclins(&mut self, _dt: f32) {
        // Будет реализовано позже
    }
}
