//! Параметры и механизм накопления повреждений центриоли (CDATA)

use serde::{Deserialize, Serialize};

/// Параметры накопления молекулярных повреждений центриоли
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageParams {
    // --- Базовые скорости повреждения (в год) ---

    /// Базовая скорость карбонилирования белков (SAS-6, CEP135) через ROS
    pub base_ros_damage_rate: f32,
    /// Скорость нарастания гиперацетилирования (снижение HDAC6/SIRT2)
    pub acetylation_rate: f32,
    /// Скорость накопления агрегатов (CPAP, CEP290)
    pub aggregation_rate: f32,
    /// Скорость нарушения фосфорилирования (PLK4/NEK2/PP1 дисбаланс)
    pub phospho_dysregulation_rate: f32,

    // --- Потеря дистальных придатков (в год) ---
    pub cep164_loss_rate: f32,
    pub cep89_loss_rate:  f32,
    pub ninein_loss_rate: f32,
    pub cep170_loss_rate: f32,

    // --- Параметры петли обратной связи ---

    /// Коэффициент: повреждение центриоли → рост ROS
    /// (нарушение митофагии → дисфункция митохондрий → больше ROS)
    pub ros_feedback_coefficient: f32,

    /// Возраст (в годах), с которого активируется SASP (inflammaging)
    pub sasp_onset_age: f32,

    /// Порог суммарного повреждения для входа в сенесценцию
    pub senescence_threshold: f32,

    /// Дополнительный множитель повреждения после 40 лет (антагонистическая плейотропия)
    pub midlife_damage_multiplier: f32,
}

impl Default for DamageParams {
    fn default() -> Self {
        Self {
            // Калибровка для шага dt = 1 день (1/365.25 лет):
            // при этих значениях is_senescent (total_damage_score > 0.75)
            // наступает ~78 лет (норма, с midlife_damage_multiplier ×1.6 после 40 лет
            // и петлёй обратной связи ROS).
            // Молекулярные скорости ×4.2 от первичных биохимических оценок:
            base_ros_damage_rate:       0.0076,   // карбонилирование SAS-6 / CEP135 (×ROS)
            acetylation_rate:           0.0059,   // гиперацетилирование α-тубулина
            aggregation_rate:           0.0059,   // агрегаты CPAP / CEP290
            phospho_dysregulation_rate: 0.0042,   // дисбаланс PLK4 / NEK2 / PP1

            // Потеря дистальных придатков (×4.2):
            cep164_loss_rate: 0.0113,  // инициация ресничек (CEP164)
            cep89_loss_rate:  0.0084,  // CEP89
            ninein_loss_rate: 0.0084,  // Ninein (субдистальные придатки)
            cep170_loss_rate: 0.0067,  // CEP170

            ros_feedback_coefficient:   0.12,
            sasp_onset_age:             45.0,
            senescence_threshold:       0.75,
            midlife_damage_multiplier:  1.6,
        }
    }
}

impl DamageParams {
    /// Вариант "ускоренного старения" (прогерия) — все скорости ×5
    pub fn progeria() -> Self {
        let mut p = Self::default();
        p.base_ros_damage_rate       *= 5.0;
        p.acetylation_rate           *= 5.0;
        p.aggregation_rate           *= 5.0;
        p.phospho_dysregulation_rate *= 5.0;
        p.cep164_loss_rate           *= 5.0;
        p.cep89_loss_rate            *= 5.0;
        p.ninein_loss_rate           *= 5.0;
        p.cep170_loss_rate           *= 5.0;
        p.midlife_damage_multiplier   = 3.0;
        p
    }

    /// Вариант "замедленного старения" (долгожители) — все скорости ×0.6
    pub fn longevity() -> Self {
        let mut p = Self::default();
        p.base_ros_damage_rate       *= 0.6;
        p.acetylation_rate           *= 0.6;
        p.aggregation_rate           *= 0.6;
        p.phospho_dysregulation_rate *= 0.6;
        p.cep164_loss_rate           *= 0.6;
        p.cep89_loss_rate            *= 0.6;
        p.ninein_loss_rate           *= 0.6;
        p.cep170_loss_rate           *= 0.6;
        p.midlife_damage_multiplier   = 1.2;
        p
    }
}

/// Обновить состояние повреждений центриоли за один временной шаг (dt_years)
pub fn accumulate_damage(
    damage: &mut cell_dt_core::components::CentriolarDamageState,
    params: &DamageParams,
    age_years: f32,
    dt_years: f32,
) {
    // Множитель: после 40 лет повреждение нарастает быстрее
    let age_multiplier = if age_years > 40.0 {
        params.midlife_damage_multiplier
    } else {
        1.0
    };

    // Петля обратной связи: накопленный ущерб усиливает ROS
    let ros_boost = 1.0 + params.ros_feedback_coefficient * damage.total_damage_score();

    let effective_dt = dt_years * age_multiplier * ros_boost;

    // Молекулярные повреждения
    damage.protein_carbonylation = (damage.protein_carbonylation
        + params.base_ros_damage_rate * damage.ros_level * effective_dt).min(1.0);

    damage.tubulin_hyperacetylation = (damage.tubulin_hyperacetylation
        + params.acetylation_rate * effective_dt).min(1.0);

    damage.protein_aggregates = (damage.protein_aggregates
        + params.aggregation_rate * effective_dt).min(1.0);

    damage.phosphorylation_dysregulation = (damage.phosphorylation_dysregulation
        + params.phospho_dysregulation_rate * effective_dt).min(1.0);

    // Потеря придатков (необратима)
    damage.cep164_integrity = (damage.cep164_integrity
        - params.cep164_loss_rate * effective_dt).max(0.0);
    damage.cep89_integrity  = (damage.cep89_integrity
        - params.cep89_loss_rate  * effective_dt).max(0.0);
    damage.ninein_integrity = (damage.ninein_integrity
        - params.ninein_loss_rate * effective_dt).max(0.0);
    damage.cep170_integrity = (damage.cep170_integrity
        - params.cep170_loss_rate * effective_dt).max(0.0);

    // ROS нарастает с возрастом и повреждениями (петля)
    let base_ros = 0.05 + age_years * 0.005;
    damage.ros_level = (base_ros
        + params.ros_feedback_coefficient * damage.total_damage_score()).min(1.0);

    // Пересчёт производных метрик
    damage.update_functional_metrics();
}
