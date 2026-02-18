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

/// Базовое состояние клеточного цикла
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

/// Типы циклинов
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CyclinType {
    CyclinD,
    CyclinE,
    CyclinA,
    CyclinB,
}

/// Типы CDK
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CdkType {
    Cdk4,
    Cdk6,
    Cdk2,
    Cdk1,
}

/// Контрольные точки
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Checkpoint {
    G1SRestriction,
    G2MCheckpoint,
    SpindleAssembly,
    DNARepair,
}

/// Комплекс Cyclin-CDK
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyclinCdkComplex {
    pub cyclin_type: CyclinType,
    pub cdk_type: CdkType,
    pub activity: f32,
    pub concentration: f32,
    pub phosphorylation_level: f32,
}

/// Факторы роста и стресса
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

/// Состояние контрольной точки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointState {
    pub checkpoint: Checkpoint,
    pub satisfied: bool,
    pub time_in_checkpoint: f32,
    pub arrest_reason: Option<String>,
}

/// Расширенное состояние клеточного цикла
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
            cyclin_cdk_complexes: vec![
                CyclinCdkComplex {
                    cyclin_type: CyclinType::CyclinD,
                    cdk_type: CdkType::Cdk4,
                    activity: 0.1,
                    concentration: 0.1,
                    phosphorylation_level: 0.0,
                },
                CyclinCdkComplex {
                    cyclin_type: CyclinType::CyclinD,
                    cdk_type: CdkType::Cdk6,
                    activity: 0.1,
                    concentration: 0.1,
                    phosphorylation_level: 0.0,
                },
                CyclinCdkComplex {
                    cyclin_type: CyclinType::CyclinE,
                    cdk_type: CdkType::Cdk2,
                    activity: 0.0,
                    concentration: 0.0,
                    phosphorylation_level: 0.0,
                },
                CyclinCdkComplex {
                    cyclin_type: CyclinType::CyclinA,
                    cdk_type: CdkType::Cdk2,
                    activity: 0.0,
                    concentration: 0.0,
                    phosphorylation_level: 0.0,
                },
                CyclinCdkComplex {
                    cyclin_type: CyclinType::CyclinA,
                    cdk_type: CdkType::Cdk1,
                    activity: 0.0,
                    concentration: 0.0,
                    phosphorylation_level: 0.0,
                },
                CyclinCdkComplex {
                    cyclin_type: CyclinType::CyclinB,
                    cdk_type: CdkType::Cdk1,
                    activity: 0.0,
                    concentration: 0.0,
                    phosphorylation_level: 0.0,
                },
            ],
            checkpoints: vec![
                CheckpointState {
                    checkpoint: Checkpoint::G1SRestriction,
                    satisfied: false,
                    time_in_checkpoint: 0.0,
                    arrest_reason: None,
                },
                CheckpointState {
                    checkpoint: Checkpoint::G2MCheckpoint,
                    satisfied: false,
                    time_in_checkpoint: 0.0,
                    arrest_reason: None,
                },
                CheckpointState {
                    checkpoint: Checkpoint::SpindleAssembly,
                    satisfied: false,
                    time_in_checkpoint: 0.0,
                    arrest_reason: None,
                },
                CheckpointState {
                    checkpoint: Checkpoint::DNARepair,
                    satisfied: true,
                    time_in_checkpoint: 0.0,
                    arrest_reason: None,
                },
            ],
            current_checkpoint: None,
            growth_factors: GrowthFactors::default(),
            cycle_count: 0,
            time_in_current_phase: 0.0,
            total_time: 0.0,
            centriole_influence: 0.0,
        }
    }
    
    /// Обновление циклинов в зависимости от фазы
    pub fn update_cyclins(&mut self, dt: f32) {
        match self.phase {
            Phase::G1 => {
                self.update_complex(CyclinType::CyclinD, CdkType::Cdk4, 0.1 * dt, 0.01);
                self.update_complex(CyclinType::CyclinD, CdkType::Cdk6, 0.1 * dt, 0.01);
                self.update_complex(CyclinType::CyclinE, CdkType::Cdk2, 0.05 * dt, 0.02);
            }
            Phase::S => {
                self.update_complex(CyclinType::CyclinD, CdkType::Cdk4, -0.05 * dt, 0.03);
                self.update_complex(CyclinType::CyclinD, CdkType::Cdk6, -0.05 * dt, 0.03);
                self.update_complex(CyclinType::CyclinE, CdkType::Cdk2, 0.02 * dt, 0.02);
                self.update_complex(CyclinType::CyclinA, CdkType::Cdk2, 0.08 * dt, 0.02);
            }
            Phase::G2 => {
                self.update_complex(CyclinType::CyclinA, CdkType::Cdk1, 0.05 * dt, 0.02);
                self.update_complex(CyclinType::CyclinB, CdkType::Cdk1, 0.1 * dt, 0.01);
            }
            Phase::M => {
                self.update_complex(CyclinType::CyclinB, CdkType::Cdk1, -0.2 * dt, 0.05);
            }
        }
    }
    
    /// Обновление конкретного комплекса
    fn update_complex(&mut self, cyclin_type: CyclinType, cdk_type: CdkType, delta: f32, activity_delta: f32) {
        for complex in &mut self.cyclin_cdk_complexes {
            if complex.cyclin_type == cyclin_type && complex.cdk_type == cdk_type {
                complex.concentration = (complex.concentration + delta).clamp(0.0, 1.0);
                complex.activity = (complex.activity + activity_delta * complex.concentration).clamp(0.0, 1.0);
                break;
            }
        }
    }
    
    /// Получить активность конкретного комплекса
    pub fn get_complex_activity(&self, cyclin_type: CyclinType, cdk_type: CdkType) -> f32 {
        for complex in &self.cyclin_cdk_complexes {
            if complex.cyclin_type == cyclin_type && complex.cdk_type == cdk_type {
                return complex.activity;
            }
        }
        0.0
    }
    
    /// Проверка контрольных точек - новая версия без конфликта заимствований
    pub fn check_checkpoints(&mut self) -> Option<Checkpoint> {
        // Сначала вычисляем все необходимые значения
        let cyclin_d_level = self.get_complex_activity(CyclinType::CyclinD, CdkType::Cdk4);
        let cyclin_e_level = self.get_complex_activity(CyclinType::CyclinE, CdkType::Cdk2);
        let cyclin_b_level = self.get_complex_activity(CyclinType::CyclinB, CdkType::Cdk1);
        
        // Теперь итерируемся по контрольным точкам
        for checkpoint in &mut self.checkpoints {
            match checkpoint.checkpoint {
                Checkpoint::G1SRestriction => {
                    checkpoint.satisfied = cyclin_d_level > 0.3 && 
                                          cyclin_e_level > 0.2 && 
                                          self.growth_factors.stress_level < 0.3 &&
                                          self.growth_factors.dna_damage < 0.1;
                    
                    if !checkpoint.satisfied {
                        checkpoint.time_in_checkpoint += 0.1;
                        return Some(Checkpoint::G1SRestriction);
                    }
                }
                Checkpoint::G2MCheckpoint => {
                    checkpoint.satisfied = cyclin_b_level > 0.5 && 
                                          self.growth_factors.dna_damage < 0.05;
                    
                    if !checkpoint.satisfied {
                        checkpoint.time_in_checkpoint += 0.1;
                        return Some(Checkpoint::G2MCheckpoint);
                    }
                }
                Checkpoint::SpindleAssembly => {
                    checkpoint.satisfied = self.centriole_influence > 0.7;
                    
                    if !checkpoint.satisfied {
                        checkpoint.time_in_checkpoint += 0.1;
                        return Some(Checkpoint::SpindleAssembly);
                    }
                }
                Checkpoint::DNARepair => {
                    if self.growth_factors.dna_damage > 0.0 {
                        self.growth_factors.dna_damage *= 0.95;
                        checkpoint.time_in_checkpoint += 0.1;
                        return Some(Checkpoint::DNARepair);
                    } else {
                        checkpoint.satisfied = true;
                    }
                }
            }
        }
        None
    }
    
    /// Обновление фазы клеточного цикла
    pub fn update_phase(&mut self, dt: f32) {
        self.time_in_current_phase += dt;
        
        if let Some(checkpoint) = self.check_checkpoints() {
            self.current_checkpoint = Some(checkpoint);
            return;
        } else {
            self.current_checkpoint = None;
        }
        
        let phase_duration = match self.phase {
            Phase::G1 => 10.0,
            Phase::S => 8.0,
            Phase::G2 => 4.0,
            Phase::M => 1.0,
        };
        
        self.progress += dt / phase_duration;
        
        if self.progress >= 1.0 {
            self.progress = 0.0;
            self.time_in_current_phase = 0.0;
            
            match self.phase {
                Phase::G1 => {
                    self.phase = Phase::S;
                }
                Phase::S => {
                    self.phase = Phase::G2;
                }
                Phase::G2 => {
                    self.phase = Phase::M;
                }
                Phase::M => {
                    self.phase = Phase::G1;
                    self.cycle_count += 1;
                }
            }
        }
    }
    
    /// Учет влияния центриоли
    pub fn apply_centriole_influence(&mut self, centriole_pair: &CentriolePair) {
        self.centriole_influence = (centriole_pair.mother.maturity + centriole_pair.daughter.maturity) / 2.0;
        
        if centriole_pair.mtoc_activity < 0.3 {
            if let Some(checkpoint) = self.checkpoints.iter_mut()
                .find(|c| c.checkpoint == Checkpoint::SpindleAssembly) {
                checkpoint.arrest_reason = Some("Low MTOC activity".to_string());
            }
        }
        
        self.growth_factors.oxidative_stress = centriole_pair.mother.ptm_signature.oxidation_level;
    }
    
    /// Получить статистику
    pub fn get_stats(&self) -> HashMap<String, f32> {
        let mut stats = HashMap::new();
        
        stats.insert("progress".to_string(), self.progress);
        stats.insert("cycle_count".to_string(), self.cycle_count as f32);
        stats.insert("time_in_phase".to_string(), self.time_in_current_phase);
        stats.insert("centriole_influence".to_string(), self.centriole_influence);
        stats.insert("growth_signal".to_string(), self.growth_factors.growth_signal);
        stats.insert("stress_level".to_string(), self.growth_factors.stress_level);
        stats.insert("dna_damage".to_string(), self.growth_factors.dna_damage);
        
        for complex in &self.cyclin_cdk_complexes {
            let key = format!("{:?}_{:?}", complex.cyclin_type, complex.cdk_type);
            stats.insert(key, complex.activity);
        }
        
        stats
    }
}

impl Default for CellCycleStateExtended {
    fn default() -> Self {
        Self::new()
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
