//! Индукторы развития для разных морфогенетических уровней

use serde::{Deserialize, Serialize};

/// Уровни морфогенеза человека
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HumanMorphogeneticLevel {
    /// Зародышевый (0-2 недели)
    Embryonic,
    /// Эмбриональный (2-8 недель)
    Fetal,
    /// Плодный (9-40 недель)
    Prenatal,
    /// Постнатальный (после рождения)
    Postnatal,
    /// Взрослый
    Adult,
    /// Старение
    Aging,
}

/// Индукторы для разных уровней развития
pub struct HumanInducers;

impl HumanInducers {
    /// Получить уровень морфогенеза по возрасту (в днях)
    pub fn get_morphogenetic_level(age_days: f64) -> HumanMorphogeneticLevel {
        if age_days < 14.0 {
            HumanMorphogeneticLevel::Embryonic
        } else if age_days < 56.0 {
            HumanMorphogeneticLevel::Fetal
        } else if age_days < 280.0 {
            HumanMorphogeneticLevel::Prenatal
        } else if age_days < 6570.0 { // 18 лет
            HumanMorphogeneticLevel::Postnatal
        } else if age_days < 18250.0 { // 50 лет
            HumanMorphogeneticLevel::Adult
        } else {
            HumanMorphogeneticLevel::Aging
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_morphogenetic_level() {
        assert!(matches!(HumanInducers::get_morphogenetic_level(7.0), HumanMorphogeneticLevel::Embryonic));
        assert!(matches!(HumanInducers::get_morphogenetic_level(30.0), HumanMorphogeneticLevel::Fetal));
        assert!(matches!(HumanInducers::get_morphogenetic_level(200.0), HumanMorphogeneticLevel::Prenatal));
        assert!(matches!(HumanInducers::get_morphogenetic_level(3650.0), HumanMorphogeneticLevel::Postnatal));
        assert!(matches!(HumanInducers::get_morphogenetic_level(10000.0), HumanMorphogeneticLevel::Adult));
        assert!(matches!(HumanInducers::get_morphogenetic_level(20000.0), HumanMorphogeneticLevel::Aging));
    }
}
