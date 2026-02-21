//! Тканеспецифичные симуляторы стволовых ниш (CDATA)

use cell_dt_core::components::{
    CentriolarDamageState, TissueState, TissueType,
};
use crate::damage::{accumulate_damage, DamageParams};

/// Тканеспецифичные профили повреждений
/// (по данным статей Tkemaladze 2023/2025)
struct TissueProfile {
    /// Общий множитель скорости повреждений
    damage_multiplier: f32,
    /// Чувствительность к потере реснички (Shh/Wnt)
    ciliary_sensitivity: f32,
    /// Пропорция утраты придатков (более чувствительные ткани)
    appendage_vulnerability: f32,
}

fn profile_for(tissue: &TissueType) -> TissueProfile {
    match tissue {
        // HSC: крайне чувствительны к повреждениям → миелоидное смещение,
        // иммуностарение
        TissueType::Hematopoietic => TissueProfile {
            damage_multiplier:       1.3,
            ciliary_sensitivity:     0.9,
            appendage_vulnerability: 1.2,
        },
        // NSC: сильная зависимость от Shh-реснички; медленная потеря пула
        TissueType::Neural => TissueProfile {
            damage_multiplier:       0.8,
            ciliary_sensitivity:     1.3,
            appendage_vulnerability: 1.1,
        },
        // Кишечные крипты: высокий темп деления → быстрее истощается пул
        TissueType::IntestinalCrypt => TissueProfile {
            damage_multiplier:       1.2,
            ciliary_sensitivity:     0.7,
            appendage_vulnerability: 1.0,
        },
        // Мышечные сателлиты: умеренный темп; медленная саркопения
        TissueType::Muscle => TissueProfile {
            damage_multiplier:       0.9,
            ciliary_sensitivity:     0.8,
            appendage_vulnerability: 0.9,
        },
        // Кожный эпителий: умеренные повреждения
        TissueType::Skin => TissueProfile {
            damage_multiplier:       1.1,
            ciliary_sensitivity:     0.6,
            appendage_vulnerability: 1.0,
        },
        // Половые клетки: защищены, низкий уровень повреждений
        TissueType::Germline => TissueProfile {
            damage_multiplier:       0.5,
            ciliary_sensitivity:     1.0,
            appendage_vulnerability: 0.8,
        },
    }
}

/// Симулятор одной тканевой ниши
pub struct TissueSimulator {
    pub state:   TissueState,
    /// Повреждение центриоли в стволовых клетках ниши
    pub damage:  CentriolarDamageState,
    profile: TissueProfile,
}

impl TissueSimulator {
    pub fn new(tissue_type: TissueType, _params: &DamageParams) -> Self {
        let profile = profile_for(&tissue_type);
        Self {
            state:  TissueState::new(tissue_type),
            damage: CentriolarDamageState::pristine(),
            profile,
        }
    }

    /// Шаг симуляции ткани
    pub fn step(&mut self, dt_years: f32, age_years: f32, params: &DamageParams) {
        // 1. Накопить повреждения с тканеспецифичным множителем
        let mut scaled_params = params.clone();
        scaled_params.base_ros_damage_rate       *= self.profile.damage_multiplier;
        scaled_params.acetylation_rate           *= self.profile.damage_multiplier;
        scaled_params.aggregation_rate           *= self.profile.damage_multiplier;
        scaled_params.phospho_dysregulation_rate *= self.profile.damage_multiplier;
        scaled_params.cep164_loss_rate *= self.profile.appendage_vulnerability;
        scaled_params.cep89_loss_rate  *= self.profile.appendage_vulnerability;
        scaled_params.ninein_loss_rate *= self.profile.appendage_vulnerability;
        scaled_params.cep170_loss_rate *= self.profile.appendage_vulnerability;

        accumulate_damage(&mut self.damage, &scaled_params, age_years, dt_years);

        // 2. Вероятность симметричного деления (нарушение АКД)
        let p_exhaust = self.damage.pool_exhaustion_probability();

        // 3. Потеря пула пропорциональна вероятности симметричного истощения
        let pool_loss = p_exhaust
            * tissue_division_rate(&self.state.tissue_type)
            * dt_years;
        self.state.stem_cell_pool = (self.state.stem_cell_pool - pool_loss).max(0.0);

        // 4. Темп регенерации: ресничка (нишевая сигнализация) × точность веретена
        let ciliary_signaling = self.damage.ciliary_function
            .powf(1.0 / self.profile.ciliary_sensitivity);
        self.state.regeneration_tempo =
            (self.state.stem_cell_pool * ciliary_signaling * self.damage.spindle_fidelity)
            .max(0.0);

        // 5. Доля сенесцентных клеток
        let senescent_rate = self.damage.total_damage_score().powf(2.0) * 0.4;
        self.state.senescent_fraction =
            (self.state.senescent_fraction + senescent_rate * dt_years).min(1.0);

        // 6. Функциональная ёмкость
        self.state.update_functional_capacity();
    }
}

fn tissue_division_rate(tissue: &TissueType) -> f32 {
    match tissue {
        TissueType::IntestinalCrypt => 10.0,
        TissueType::Hematopoietic   => 7.0,
        TissueType::Skin            => 5.0,
        TissueType::Muscle          => 2.0,
        TissueType::Neural          => 0.8,
        TissueType::Germline        => 3.0,
    }
}
