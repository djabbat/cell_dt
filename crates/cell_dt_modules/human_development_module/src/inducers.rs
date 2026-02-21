//! Система S/H индукторов дифференцировки (Tkemaladze 2005/2023)
//!
//! S-структура (соматическая линия): N_S индукторов → при каждом
//! дифференцирующем делении один высвобождается, переключая генную сеть.
//! Когда S_count = 0 → терминальная дифференцировка или апоптоз.
//!
//! H-структура (гаметная линия): N_H индукторов → при последнем
//! высвобождении запускается мейоз и восстановление тотипотентности.

use cell_dt_core::components::CentriolarInducers;

/// Морфогенетический уровень (потенция) по числу оставшихся S-индукторов
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MorphogeneticLevel {
    /// S_count = S_max → нулевой статус (тотипотентная)
    Null,
    /// S_count > 75% → плюрипотентная
    Pluripotent,
    /// S_count > 50% → мультипотентная
    Multipotent,
    /// S_count > 25% → олигопотентная
    Oligopotent,
    /// S_count > 0  → унипотентная стволовая
    Unipotent,
    /// S_count = 0  → терминально дифференцированная
    Terminal,
}

/// Расширение CentriolarInducers методами дифференцировки (trait extension)
pub trait InducerDivisionExt {
    fn morphogenetic_level(&self) -> MorphogeneticLevel;
    fn asymmetric_divide(&mut self, spindle_ok: bool, rng_val: f32) -> DivisionOutcome;
}

impl InducerDivisionExt for CentriolarInducers {
    /// Вычислить морфогенетический уровень по текущему S-счётчику
    fn morphogenetic_level(&self) -> MorphogeneticLevel {
        if self.s_max == 0 {
            return MorphogeneticLevel::Terminal;
        }
        let ratio = self.s_count as f32 / self.s_max as f32;
        match ratio {
            r if r >= 1.0         => MorphogeneticLevel::Null,
            r if r >  0.75        => MorphogeneticLevel::Pluripotent,
            r if r >  0.50        => MorphogeneticLevel::Multipotent,
            r if r >  0.25        => MorphogeneticLevel::Oligopotent,
            r if r >  0.0         => MorphogeneticLevel::Unipotent,
            _                     => MorphogeneticLevel::Terminal,
        }
    }

    /// Провести дифференцирующее деление:
    /// - потребляет S-индуктор
    /// - возвращает (дочерняя_стволовая, дочерняя_дифференцирующаяся)
    ///
    /// `spindle_ok`: если false — симметричное деление (оба теряют или
    ///               оба сохраняют стволовость, с вероятностью 0.5)
    fn asymmetric_divide(
        &mut self,
        spindle_ok: bool,
        rng_val: f32,  // [0..1)
    ) -> DivisionOutcome {
        self.differentiation_divisions += 1;

        if spindle_ok {
            // Нормальное асимметричное деление: одна клетка дифференцируется
            if self.s_count > 0 {
                let mut differentiating_daughter = self.clone();
                differentiating_daughter.consume_s_inducer();
                DivisionOutcome::Asymmetric {
                    stem_daughter:          self.clone(),
                    differentiating_daughter,
                }
            } else {
                DivisionOutcome::TerminalDifferentiation
            }
        } else if rng_val < 0.5 {
            // Симметричное истощение: обе клетки дифференцируются
            DivisionOutcome::SymmetricDifferentiation
        } else {
            // Симметричное самообновление: обе сохраняют стволовость
            DivisionOutcome::SymmetricSelfRenewal
        }
    }
}

/// Результат деления стволовой клетки
#[derive(Debug, Clone)]
pub enum DivisionOutcome {
    /// Нормальное асимметричное деление (одна дочь стволовая, другая дифференцируется)
    Asymmetric {
        stem_daughter:           CentriolarInducers,
        differentiating_daughter: CentriolarInducers,
    },
    /// Симметричное истощение (обе дочери дифференцируются → пул убывает)
    SymmetricDifferentiation,
    /// Симметричное самообновление (обе стволовые → клональная экспансия)
    SymmetricSelfRenewal,
    /// Терминальная дифференцировка (S_count = 0, клетка выходит из цикла)
    TerminalDifferentiation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_morphogenetic_levels() {
        let full  = CentriolarInducers::zygote(50, 4);
        assert_eq!(full.morphogenetic_level(), MorphogeneticLevel::Null);

        let mut mid = CentriolarInducers::zygote(50, 4);
        mid.s_count = 25;
        assert_eq!(mid.morphogenetic_level(), MorphogeneticLevel::Oligopotent);

        let mut terminal = CentriolarInducers::zygote(50, 4);
        terminal.s_count = 0;
        assert_eq!(terminal.morphogenetic_level(), MorphogeneticLevel::Terminal);
    }

    #[test]
    fn test_inducer_consumption() {
        let mut ind = CentriolarInducers::zygote(5, 2);
        assert!(!ind.is_terminally_differentiated());
        for _ in 0..5 {
            ind.consume_s_inducer();
        }
        assert!(ind.is_terminally_differentiated());
        // Лишнее потребление — не выходит за ноль
        ind.consume_s_inducer();
        assert_eq!(ind.s_count, 0);
    }

    #[test]
    fn test_s_status_gradient() {
        let mut ind = CentriolarInducers::zygote(100, 4);
        assert_eq!(ind.s_status(), 0.0);
        ind.s_count = 50;
        assert!((ind.s_status() - 0.5).abs() < 1e-6);
        ind.s_count = 0;
        assert!((ind.s_status() - 1.0).abs() < 1e-6);
    }
}
