//! P11 — Терапевтические интервенции (InterventionSchedule)
//!
//! Позволяет задать расписание терапий, активных в заданном возрастном диапазоне.
//! Каждая интервенция применяется в `HumanDevelopmentModule::step()` при совпадении возраста.
//!
//! ## Поддерживаемые интервенции
//!
//! | Тип                  | Механизм CDATA                                              |
//! |----------------------|-------------------------------------------------------------|
//! | Senolytics           | Клиренс сенесцентных клеток → снижение SASP                 |
//! | NadPlus              | NAD⁺/сиртуины → активация митофагии → снижение ROS         |
//! | CaloricRestriction   | Снижение метаболического потока → меньше ROS                |
//! | TertActivation       | Теломераза → удлинение теломер                             |
//! | Antioxidant          | Прямое снижение оксидативных повреждений                   |
//! | CafdRetainer         | Стабилизация индукторов → снижение вероятности отщепления  |
//! | CafdReleaser         | Принудительное ускорение отщепления → дифференцировка      |
//! | CentrosomeTransplant | Замена центросомы молодой → восстановление индукторов       |

use crate::damage::DamageParams;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Типы интервенций
// ---------------------------------------------------------------------------

/// Один терапевтический сценарий.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intervention {
    /// Возраст начала терапии [лет]
    pub start_age_years: f32,
    /// Возраст окончания терапии [лет]. `None` = до конца жизни.
    pub end_age_years: Option<f32>,
    /// Вид терапии
    pub kind: InterventionKind,
}

/// Виды терапевтических вмешательств.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterventionKind {
    /// Сенолитики — удаляют сенесцентные клетки, снижая SASP.
    ///
    /// `clearance_rate` [0..1/год]: доля сенесцентных клеток, удаляемых за год.
    /// Типичное значение для Navitoclax/Dasatinib+Quercetin: 0.3–0.5/год.
    Senolytics { clearance_rate: f32 },

    /// NAD⁺-бустеры (NMN, NR) — активируют сиртуины и митофагию.
    ///
    /// `mitophagy_boost` [0..1]: аддитивная добавка к `mitophagy_flux`.
    /// Усиливает репарацию придатков и снижает ROS через PINK1/Parkin.
    NadPlus { mitophagy_boost: f32 },

    /// Калорийное ограничение — снижает метаболический поток и ROS.
    ///
    /// `ros_factor` [0..1]: множитель для `base_ros_damage_rate` и `aggregation_rate`.
    /// Типичный ответ на 30% CR у грызунов: ros_factor ≈ 0.7.
    CaloricRestriction { ros_factor: f32 },

    /// Активация теломеразы (TERT-ген-терапия) — удлиняет теломеры.
    ///
    /// `elongation_per_div` [0..0.01]: добавка к длине теломер за деление.
    /// Типичный эффект AAV-TERT у мышей: +20–30% длины теломер.
    TertActivation { elongation_per_div: f32 },

    /// Антиоксидантная терапия — переключает DamageParams на пресет `antioxidant()`.
    ///
    /// Эффективно снижает `base_ros_damage_rate × 0.5` и включает репарацию придатков.
    Antioxidant,

    /// CAFD-ретейнер — молекула, стабилизирующая индукторы (конкурирует с O₂).
    ///
    /// Источник: «Pharmacological Control via CAFDs» (2026-02-16).
    ///
    /// `retention_factor` [0..1]: доля снижения `base_detach_probability`.
    /// retention_factor=0.5 → вероятность отщепления снижается на 50%.
    /// Типичный эффект центросомального шаперона: 0.3–0.6.
    CafdRetainer { retention_factor: f32 },

    /// CAFD-релизер — принудительное отщепление индукторов → ускоренная дифференцировка.
    ///
    /// Применение: онкологический клиренс стволовых клеток опухоли.
    ///
    /// `release_factor` [0..∞): множитель ускорения `base_detach_probability`.
    /// release_factor=3.0 → вероятность отщепления ×3.
    CafdReleaser { release_factor: f32 },

    /// Трансплантация центросомы — замена центросомального комплекта молодым.
    ///
    /// Источник: «Centrosome Transplantation» (2026-01-16).
    ///
    /// Моделируется как поддержание минимального уровня индукторов:
    /// если текущий остаток ниже донорских значений — восстанавливается.
    /// После окончания окна активности начинается естественное истощение заново.
    ///
    /// `donor_m_count` — уровень M-индукторов донора (молодая клетка: 8–10).
    /// `donor_d_count` — уровень D-индукторов донора (молодая клетка: 6–8).
    CentrosomeTransplant { donor_m_count: u32, donor_d_count: u32 },
}

impl Intervention {
    /// Проверить, активна ли интервенция при данном возрасте.
    #[inline]
    pub fn is_active_at(&self, age_years: f32) -> bool {
        age_years >= self.start_age_years
            && self.end_age_years.is_none_or(|end| age_years < end)
    }
}

// ---------------------------------------------------------------------------
// Применение интервенций к DamageParams
// ---------------------------------------------------------------------------

/// Результат применения всех активных интервенций к параметрам шага.
#[derive(Debug, Clone)]
pub struct InterventionEffect {
    /// Эффективные параметры повреждений (с поправками от активных интервенций)
    pub damage_params: DamageParams,
    /// Дополнительный буст митофагии [0..1] (NadPlus → repair amplification)
    pub extra_mitophagy: f32,
    /// Фракция сенесцентных клеток, удалённых в этом шаге [0..1]
    pub senolytic_clearance: f32,
    /// Добавка к длине теломер за деление (TertActivation) [0..0.01]
    pub tert_elongation: f32,
    /// Множитель base_detach_probability для индукторов (CafdRetainer/CafdReleaser).
    /// < 1.0 = стабилизация; > 1.0 = ускоренное отщепление; 1.0 = без эффекта.
    pub detach_probability_modifier: f32,
    /// Трансплантация центросомы: минимальные уровни M и D после трансплантации.
    /// `None` = нет активной трансплантации.
    pub centrosome_transplant: Option<(u32, u32)>,
}

impl InterventionEffect {
    /// Нейтральный эффект (никаких интервенций не применяется).
    pub fn none(base: &DamageParams) -> Self {
        Self {
            damage_params: base.clone(),
            extra_mitophagy: 0.0,
            senolytic_clearance: 0.0,
            tert_elongation: 0.0,
            detach_probability_modifier: 1.0,
            centrosome_transplant: None,
        }
    }
}

/// Вычислить суммарный эффект всех активных интервенций при данном возрасте.
pub fn compute_effect(
    interventions: &[Intervention],
    age_years: f32,
    base: &DamageParams,
    dt_years: f32,
) -> InterventionEffect {
    let mut eff = InterventionEffect::none(base);

    for iv in interventions {
        if !iv.is_active_at(age_years) {
            continue;
        }
        match iv.kind {
            InterventionKind::Senolytics { clearance_rate } => {
                // Клиренс за шаг = clearance_rate × dt_years (дробная доля)
                eff.senolytic_clearance += clearance_rate * dt_years;
            }
            InterventionKind::NadPlus { mitophagy_boost } => {
                eff.extra_mitophagy += mitophagy_boost;
                // NAD⁺ → SIRT3 → снижает агрегацию белков (~15%)
                eff.damage_params.aggregation_rate *= 1.0 - mitophagy_boost * 0.15;
                // NAD⁺ → снижает базовый ROS через митохондриальную эффективность
                eff.damage_params.base_ros_damage_rate *= 1.0 - mitophagy_boost * 0.20;
            }
            InterventionKind::CaloricRestriction { ros_factor } => {
                eff.damage_params.base_ros_damage_rate *= ros_factor;
                eff.damage_params.aggregation_rate     *= ros_factor;
                // CR также снижает темп ацетилирования (SIRT1 активация)
                eff.damage_params.acetylation_rate     *= ros_factor.sqrt();
            }
            InterventionKind::TertActivation { elongation_per_div } => {
                eff.tert_elongation += elongation_per_div;
            }
            InterventionKind::Antioxidant => {
                // Применяем пресет antioxidant: ROS ×0.5, агрегация ×0.7, репарация включена
                let anti = DamageParams::antioxidant();
                eff.damage_params.base_ros_damage_rate =
                    eff.damage_params.base_ros_damage_rate.min(anti.base_ros_damage_rate);
                eff.damage_params.aggregation_rate =
                    eff.damage_params.aggregation_rate.min(anti.aggregation_rate);
                // Включаем репарацию если она выключена
                if eff.damage_params.cep164_repair_rate < anti.cep164_repair_rate {
                    eff.damage_params.cep164_repair_rate = anti.cep164_repair_rate;
                    eff.damage_params.cep89_repair_rate  = anti.cep89_repair_rate;
                    eff.damage_params.ninein_repair_rate = anti.ninein_repair_rate;
                    eff.damage_params.cep170_repair_rate = anti.cep170_repair_rate;
                    eff.damage_params.appendage_repair_mitophagy_coupling =
                        anti.appendage_repair_mitophagy_coupling;
                }
            }
            InterventionKind::CafdRetainer { retention_factor } => {
                // Снижает base_detach_probability на retention_factor [0..1]:
                // modifier × (1 - retention_factor). Стабилизирует индукторы.
                eff.detach_probability_modifier *= 1.0 - retention_factor.clamp(0.0, 1.0);
            }
            InterventionKind::CafdReleaser { release_factor } => {
                // Ускоряет отщепление индукторов: modifier × (1 + release_factor).
                // Применяется в онкологии для принудительной дифференцировки.
                eff.detach_probability_modifier *= 1.0 + release_factor.max(0.0);
            }
            InterventionKind::CentrosomeTransplant { donor_m_count, donor_d_count } => {
                // Устанавливает донорские уровни индукторов (минимальный порог).
                // Несколько трансплантаций в один шаг → берём наибольшие донорские значения.
                eff.centrosome_transplant = Some(match eff.centrosome_transplant {
                    None => (donor_m_count, donor_d_count),
                    Some((prev_m, prev_d)) => (prev_m.max(donor_m_count), prev_d.max(donor_d_count)),
                });
            }
        }
    }

    // Clamp: скорости не могут уйти в отрицательные значения
    eff.damage_params.base_ros_damage_rate = eff.damage_params.base_ros_damage_rate.max(0.0);
    eff.damage_params.aggregation_rate     = eff.damage_params.aggregation_rate.max(0.0);
    eff.damage_params.acetylation_rate     = eff.damage_params.acetylation_rate.max(0.0);
    eff.senolytic_clearance = eff.senolytic_clearance.clamp(0.0, 0.99);

    eff
}

// ---------------------------------------------------------------------------
// Тесты
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::damage::DamageParams;

    fn base() -> DamageParams { DamageParams::default() }

    #[test]
    fn senolytics_extend_lifespan() {
        // Сенолитики снижают senolytic_clearance > 0 при активном возрасте
        let ivs = vec![Intervention {
            start_age_years: 60.0,
            end_age_years: None,
            kind: InterventionKind::Senolytics { clearance_rate: 0.4 },
        }];
        // До начала — неактивна
        let before = compute_effect(&ivs, 59.9, &base(), 1.0 / 365.25);
        assert_eq!(before.senolytic_clearance, 0.0,
            "до 60 лет сенолитики не работают");

        // После начала — активна
        let after = compute_effect(&ivs, 60.1, &base(), 1.0 / 365.25);
        assert!(after.senolytic_clearance > 0.0,
            "после 60 лет senolytic_clearance > 0");
    }

    #[test]
    fn nad_plus_improves_mitochondria_at_70() {
        // NAD+ при возрасте 70 лет снижает ROS-скорость и добавляет митофагию
        let ivs = vec![Intervention {
            start_age_years: 50.0,
            end_age_years: None,
            kind: InterventionKind::NadPlus { mitophagy_boost: 0.5 },
        }];
        let eff = compute_effect(&ivs, 70.0, &base(), 1.0 / 365.25);

        let b = base();
        assert!(eff.damage_params.base_ros_damage_rate < b.base_ros_damage_rate,
            "NAD+ снижает base_ros_damage_rate: {:.6} < {:.6}",
            eff.damage_params.base_ros_damage_rate, b.base_ros_damage_rate);
        assert!(eff.extra_mitophagy > 0.0,
            "NAD+ добавляет extra_mitophagy: {:.3}", eff.extra_mitophagy);
        assert!(eff.damage_params.aggregation_rate < b.aggregation_rate,
            "NAD+ снижает aggregation_rate");
    }

    #[test]
    fn caloric_restriction_reduces_ros_and_aggregation() {
        let ivs = vec![Intervention {
            start_age_years: 30.0,
            end_age_years: Some(90.0),
            kind: InterventionKind::CaloricRestriction { ros_factor: 0.7 },
        }];
        let eff = compute_effect(&ivs, 50.0, &base(), 1.0 / 365.25);
        let b = base();
        assert!(eff.damage_params.base_ros_damage_rate < b.base_ros_damage_rate);
        assert!(eff.damage_params.aggregation_rate     < b.aggregation_rate);
        assert!(eff.damage_params.acetylation_rate     < b.acetylation_rate);
        // Снаружи диапазона — нет эффекта
        let outside = compute_effect(&ivs, 95.0, &base(), 1.0 / 365.25);
        assert_eq!(outside.damage_params.base_ros_damage_rate, b.base_ros_damage_rate);
    }

    #[test]
    fn tert_activation_gives_elongation() {
        let ivs = vec![Intervention {
            start_age_years: 50.0,
            end_age_years: None,
            kind: InterventionKind::TertActivation { elongation_per_div: 0.001 },
        }];
        let eff = compute_effect(&ivs, 55.0, &base(), 1.0 / 365.25);
        assert!(eff.tert_elongation > 0.0, "TertActivation добавляет elongation");
    }

    #[test]
    fn antioxidant_enables_repair_rates() {
        let base_params = DamageParams::default();
        assert_eq!(base_params.cep164_repair_rate, 0.0, "по умолчанию репарация выключена");

        let ivs = vec![Intervention {
            start_age_years: 0.0,
            end_age_years: None,
            kind: InterventionKind::Antioxidant,
        }];
        let eff = compute_effect(&ivs, 40.0, &base_params, 1.0 / 365.25);
        assert!(eff.damage_params.cep164_repair_rate > 0.0, "Antioxidant включает repair");
        assert!(eff.damage_params.base_ros_damage_rate < base_params.base_ros_damage_rate);
    }

    #[test]
    fn cafd_retainer_reduces_detach_probability() {
        let ivs = vec![Intervention {
            start_age_years: 60.0,
            end_age_years: None,
            kind: InterventionKind::CafdRetainer { retention_factor: 0.6 },
        }];
        let eff = compute_effect(&ivs, 65.0, &base(), 1.0 / 365.25);
        // modifier = 1.0 - 0.6 = 0.4 → снижение на 60%
        assert!((eff.detach_probability_modifier - 0.4).abs() < 1e-5,
            "CafdRetainer(0.6): modifier={:.4}, ожидается 0.4", eff.detach_probability_modifier);
        // До начала — нейтральный
        let before = compute_effect(&ivs, 59.0, &base(), 1.0 / 365.25);
        assert!((before.detach_probability_modifier - 1.0).abs() < 1e-5,
            "до 60 лет modifier=1.0");
    }

    #[test]
    fn cafd_releaser_increases_detach_probability() {
        let ivs = vec![Intervention {
            start_age_years: 0.0,
            end_age_years: Some(5.0),
            kind: InterventionKind::CafdReleaser { release_factor: 3.0 },
        }];
        let eff = compute_effect(&ivs, 1.0, &base(), 1.0 / 365.25);
        // modifier = 1.0 + 3.0 = 4.0
        assert!((eff.detach_probability_modifier - 4.0).abs() < 1e-5,
            "CafdReleaser(3.0): modifier={:.4}, ожидается 4.0", eff.detach_probability_modifier);
    }

    #[test]
    fn centrosome_transplant_sets_donor_levels() {
        let ivs = vec![Intervention {
            start_age_years: 50.0,
            end_age_years: Some(51.0), // 1 год активен
            kind: InterventionKind::CentrosomeTransplant { donor_m_count: 9, donor_d_count: 7 },
        }];
        let eff = compute_effect(&ivs, 50.5, &base(), 1.0 / 365.25);
        assert_eq!(eff.centrosome_transplant, Some((9, 7)),
            "transplant должен передать донорские уровни 9/7");
        // Вне окна — нет трансплантации
        let outside = compute_effect(&ivs, 55.0, &base(), 1.0 / 365.25);
        assert!(outside.centrosome_transplant.is_none(),
            "вне окна трансплантация не активна");
    }

    #[test]
    fn cafd_retainer_stacks_with_releaser() {
        // Одновременный ретейнер+релизер: эффекты перемножаются
        let ivs = vec![
            Intervention {
                start_age_years: 0.0,
                end_age_years: None,
                kind: InterventionKind::CafdRetainer { retention_factor: 0.5 },
            },
            Intervention {
                start_age_years: 0.0,
                end_age_years: None,
                kind: InterventionKind::CafdReleaser { release_factor: 1.0 },
            },
        ];
        let eff = compute_effect(&ivs, 30.0, &base(), 1.0 / 365.25);
        // modifier = (1.0 - 0.5) × (1.0 + 1.0) = 0.5 × 2.0 = 1.0
        assert!((eff.detach_probability_modifier - 1.0).abs() < 1e-5,
            "ретейнер×0.5 + релизер×2.0 = нейтральный: {:.4}", eff.detach_probability_modifier);
    }

    #[test]
    fn combined_interventions_stack() {
        // NadPlus + CaloricRestriction суммируются
        let ivs = vec![
            Intervention {
                start_age_years: 50.0,
                end_age_years: None,
                kind: InterventionKind::NadPlus { mitophagy_boost: 0.3 },
            },
            Intervention {
                start_age_years: 50.0,
                end_age_years: None,
                kind: InterventionKind::CaloricRestriction { ros_factor: 0.8 },
            },
        ];
        let eff = compute_effect(&ivs, 60.0, &base(), 1.0 / 365.25);
        let single_nad = compute_effect(&ivs[..1], 60.0, &base(), 1.0 / 365.25);
        // Комбинация должна давать ещё меньший ROS
        assert!(eff.damage_params.base_ros_damage_rate < single_nad.damage_params.base_ros_damage_rate,
            "комбинация CR+NAD+ сильнее одного NAD+");
    }
}
