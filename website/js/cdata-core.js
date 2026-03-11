/**
 * CDATA — Centriolar Damage Accumulation Theory of Aging
 * Core simulation engine (JavaScript port of Rust digital twin)
 * Author of theory: Jaba Tkemaladze
 */

// ─── Constants ────────────────────────────────────────────────────────────────
export const CDATA = {
  STEPS_PER_YEAR: 365,
  SENESCENCE_THRESHOLD: 0.75,
  INDUCER_M_INIT: 10,
  INDUCER_D_INIT: 8,
  BASE_DETACH_PROB: 0.002,
  MOTHER_BIAS: 0.5,
  MIDLIFE_MULTIPLIER: 1.6,
  MIDLIFE_AGE: 40,
};

// ─── Potency Level ────────────────────────────────────────────────────────────
export function getPotency(m, d) {
  if (m >= CDATA.INDUCER_M_INIT && d >= CDATA.INDUCER_D_INIT) return 'Totipotent';
  if (m >= 1 && d >= 1) return 'Pluripotent';
  if (m === 0 && d >= 2) return 'Oligopotent';
  if (d === 0 && m >= 2) return 'Oligopotent';
  if ((m === 0 && d === 1) || (d === 0 && m === 1)) return 'Unipotent';
  return 'Apoptosis';
}

export function potencyScore(p) {
  return { Totipotent: 1.0, Pluripotent: 0.75, Oligopotent: 0.5, Unipotent: 0.25, Apoptosis: 0 }[p] ?? 0;
}

// ─── Damage Parameters ────────────────────────────────────────────────────────
export class DamageParams {
  constructor(opts = {}) {
    this.base_damage_rate         = opts.base_damage_rate         ?? 0.00008;
    this.ros_feedback_coefficient  = opts.ros_feedback_coefficient  ?? 0.12;
    this.senescence_threshold      = opts.senescence_threshold      ?? CDATA.SENESCENCE_THRESHOLD;
    this.midlife_damage_multiplier = opts.midlife_damage_multiplier ?? CDATA.MIDLIFE_MULTIPLIER;
    this.midlife_age               = opts.midlife_age               ?? CDATA.MIDLIFE_AGE;
    this.noise_scale               = opts.noise_scale               ?? 0.1;
    // Track rates
    this.cilia_decay_rate    = opts.cilia_decay_rate    ?? 0.00012;
    this.spindle_decay_rate  = opts.spindle_decay_rate  ?? 0.00010;
    this.telomere_shorten    = opts.telomere_shorten    ?? 0.0004;
    this.epigenetic_rate     = opts.epigenetic_rate     ?? 0.000027; // 1 year per year
    this.mtdna_mutation_rate = opts.mtdna_mutation_rate ?? 0.00006;
    this.ros_base            = opts.ros_base            ?? 0.05;
  }

  static default() { return new DamageParams(); }
  static progeria() { return new DamageParams({ base_damage_rate: 0.0004, ros_feedback_coefficient: 0.25, cilia_decay_rate: 0.0006, spindle_decay_rate: 0.0005 }); }
  static longevity() { return new DamageParams({ base_damage_rate: 0.000048, ros_feedback_coefficient: 0.07, midlife_damage_multiplier: 1.2 }); }
  static senolytics() { return new DamageParams({ ros_feedback_coefficient: 0.08, senescence_threshold: 0.82 }); }
  static nadplus() { return new DamageParams({ mtdna_mutation_rate: 0.00003, ros_base: 0.03, ros_feedback_coefficient: 0.09 }); }
  static caloric_restriction() { return new DamageParams({ base_damage_rate: 0.00006, midlife_damage_multiplier: 1.3, midlife_age: 45 }); }
}

// ─── Cell State ───────────────────────────────────────────────────────────────
export class CellState {
  constructor(params = new DamageParams()) {
    this.params = params;
    this.reset();
  }

  reset() {
    // Inducers
    this.m_inducers = CDATA.INDUCER_M_INIT;
    this.d_inducers = CDATA.INDUCER_D_INIT;
    // Damage
    this.total_damage = 0;
    this.ros_level = this.params.ros_base;
    this.protein_aggregates = 0;
    this.dna_damage = 0;
    // Track A — Cilia
    this.cep164 = 1.0;
    this.cep89 = 1.0;
    this.ciliary_function = 1.0;
    // Track B — Spindle
    this.spindle_fidelity = 1.0;
    this.symmetric_divisions = 0;
    this.stem_cell_pool = 1.0;
    // Track C — Telomere
    this.telomere_length = 1.0;
    this.division_count = 0;
    this.hayflick_arrested = false;
    // Track D — Epigenetic clock
    this.methylation_age = 0;
    this.chrono_age = 0;
    this.clock_acceleration = 1.0;
    // Track E — Mitochondrial
    this.mtdna_mutations = 0;
    this.mito_ros = this.params.ros_base;
    this.fusion_index = 1.0;
    this.membrane_potential = 1.0;
    this.mito_shield = 1.0;
    // Myeloid shift
    this.myeloid_bias = 0.0;
    this.inflammaging_index = 0.0;
    // Tissue
    this.regeneration_tempo = 1.0;
    this.senescent_fraction = 0;
    // Status
    this.age_years = 0;
    this.is_alive = true;
    this.death_cause = null;
    this.step_count = 0;
  }

  get potency() { return getPotency(this.m_inducers, this.d_inducers); }
  get potency_score() { return potencyScore(this.potency); }

  /** Age multiplier — sigmoid around midlife */
  ageMultiplier() {
    const age = this.age_years;
    const p = this.params;
    if (age < p.midlife_age) return 1.0;
    const t = (age - p.midlife_age) / 20;
    return 1.0 + (p.midlife_damage_multiplier - 1.0) * (1 / (1 + Math.exp(-3 * (t - 0.5))));
  }

  /** Mito shield */
  computeMitoShield() {
    return this.fusion_index * 0.4 + this.membrane_potential * 0.35 + (1 - this.mito_ros) * 0.25;
  }

  /** Centrosomal oxygen penetration */
  centrosomalOxygen() {
    return Math.max(0, 1 - this.mito_shield);
  }

  /** Step one simulated day */
  step() {
    if (!this.is_alive) return;
    this.step_count++;
    this.age_years = this.step_count / CDATA.STEPS_PER_YEAR;
    this.chrono_age = this.age_years;

    const p = this.params;
    const ageM = this.ageMultiplier();
    const noise = 1 + (Math.random() - 0.5) * p.noise_scale;

    // ── Track E: Mitochondrial ────────────────────────────────────────────
    this.mtdna_mutations = Math.min(1, this.mtdna_mutations + p.mtdna_mutation_rate * ageM * noise);
    this.mito_ros = Math.min(1, p.ros_base + this.mtdna_mutations * 0.4 + this.protein_aggregates * 0.2);
    this.fusion_index = Math.max(0, 1 - this.mtdna_mutations * 0.5 - this.ros_level * 0.3);
    this.membrane_potential = Math.max(0, 1 - this.mtdna_mutations * 0.6);
    this.mito_shield = this.computeMitoShield();

    // ── ROS (combined) ────────────────────────────────────────────────────
    const inflamm_ros_boost = this.inflammaging_index * 0.15;
    this.ros_level = Math.min(1, this.mito_ros + this.total_damage * p.ros_feedback_coefficient + inflamm_ros_boost);

    // ── Protein aggregates ────────────────────────────────────────────────
    this.protein_aggregates = Math.min(1, this.protein_aggregates + this.ros_level * 0.00003 * ageM);

    // ── DNA damage ────────────────────────────────────────────────────────
    this.dna_damage = Math.min(1, this.dna_damage + this.ros_level * 0.00002 * ageM);

    // ── Inducer detachment (O₂-path) ──────────────────────────────────────
    const o2 = this.centrosomalOxygen();
    const detach_prob = p.base_damage_rate * o2 * ageM * noise * 365; // per day
    if (Math.random() < detach_prob) {
      const mother_prob = CDATA.MOTHER_BIAS;
      if (Math.random() < mother_prob && this.m_inducers > 0) {
        this.m_inducers--;
      } else if (this.d_inducers > 0) {
        this.d_inducers--;
      }
    }

    // ── Track A: Cilia (CEP164/CEP89 decay) ──────────────────────────────
    this.cep164 = Math.max(0, this.cep164 - p.cilia_decay_rate * this.ros_level * ageM * noise);
    this.cep89 = Math.max(0, this.cep89 - p.cilia_decay_rate * 0.8 * this.ros_level * ageM * noise);
    this.ciliary_function = (this.cep164 * 0.6 + this.cep89 * 0.4) * this.potency_score;

    // ── Track B: Spindle fidelity ─────────────────────────────────────────
    this.spindle_fidelity = Math.max(0,
      this.spindle_fidelity - p.spindle_decay_rate * this.ros_level * ageM * noise
    );
    // Symmetric division → pool exhaustion
    if (this.spindle_fidelity < 0.5 && Math.random() < (1 - this.spindle_fidelity) * 0.001) {
      this.symmetric_divisions++;
      this.stem_cell_pool = Math.max(0, this.stem_cell_pool - 0.001);
    }

    // ── Track C: Telomere ─────────────────────────────────────────────────
    const ros_factor = 1 + this.ros_level * 0.5;
    const shorten_rate = p.telomere_shorten * (1 / CDATA.STEPS_PER_YEAR) * this.spindle_fidelity * ros_factor;
    if (!this.hayflick_arrested) {
      this.telomere_length = Math.max(0, this.telomere_length - shorten_rate);
      this.division_count++;
      if (this.telomere_length < 0.3) {
        this.hayflick_arrested = true;
      }
    }

    // ── Track D: Epigenetic clock ─────────────────────────────────────────
    this.clock_acceleration = 1 + this.total_damage * 0.5;
    this.methylation_age += p.epigenetic_rate * this.clock_acceleration;

    // ── Total damage (composite) ──────────────────────────────────────────
    this.total_damage = (
      (1 - this.ciliary_function) * 0.20 +
      (1 - this.spindle_fidelity) * 0.25 +
      (1 - this.telomere_length) * 0.20 +
      Math.min(1, this.methylation_age / 120) * 0.15 +
      this.mtdna_mutations * 0.20
    );

    // ── Myeloid shift ─────────────────────────────────────────────────────
    const sf_term = Math.pow(1 - this.spindle_fidelity, 1.5) * 0.45;
    const cilia_term = (1 - this.ciliary_function) * 0.30;
    const ros_term = this.ros_level * 0.15;
    const agg_term = this.protein_aggregates * 0.10;
    this.myeloid_bias = Math.min(1, sf_term + cilia_term + ros_term + agg_term);
    this.inflammaging_index = this.myeloid_bias * 0.8;

    // ── Tissue ────────────────────────────────────────────────────────────
    const niche_impairment = this.inflammaging_index * 0.08;
    this.regeneration_tempo = Math.max(0,
      this.ciliary_function * 0.5 + this.stem_cell_pool * 0.5 - niche_impairment
    );
    this.senescent_fraction = Math.min(1, (1 - this.spindle_fidelity) * 0.4 + (1 - this.telomere_length) * 0.3 + this.protein_aggregates * 0.3);

    // ── Death check ───────────────────────────────────────────────────────
    if (this.potency === 'Apoptosis') {
      this.is_alive = false;
      this.death_cause = 'Inducer exhaustion → Apoptosis';
    } else if (this.total_damage >= p.senescence_threshold) {
      this.is_alive = false;
      this.death_cause = `Damage threshold (${p.senescence_threshold.toFixed(2)}) reached at ${this.age_years.toFixed(1)} years`;
    } else if (this.stem_cell_pool < 0.05) {
      this.is_alive = false;
      this.death_cause = 'Stem cell pool exhausted';
    }
  }

  /** Run full lifetime simulation. Returns yearly snapshots. */
  runLifetime(max_years = 120) {
    this.reset();
    const snapshots = [];
    const steps_per_year = CDATA.STEPS_PER_YEAR;

    while (this.is_alive && this.age_years < max_years) {
      for (let d = 0; d < steps_per_year && this.is_alive; d++) {
        this.step();
      }
      snapshots.push(this.snapshot());
    }
    return snapshots;
  }

  snapshot() {
    return {
      age: this.age_years,
      total_damage: this.total_damage,
      ros_level: this.ros_level,
      ciliary_function: this.ciliary_function,
      spindle_fidelity: this.spindle_fidelity,
      telomere_length: this.telomere_length,
      methylation_age: this.methylation_age,
      mtdna_mutations: this.mtdna_mutations,
      myeloid_bias: this.myeloid_bias,
      inflammaging_index: this.inflammaging_index,
      stem_cell_pool: this.stem_cell_pool,
      regeneration_tempo: this.regeneration_tempo,
      senescent_fraction: this.senescent_fraction,
      m_inducers: this.m_inducers,
      d_inducers: this.d_inducers,
      potency: this.potency,
      potency_score: this.potency_score,
      mito_shield: this.mito_shield,
      clock_acceleration: this.clock_acceleration,
      is_alive: this.is_alive,
      death_cause: this.death_cause,
    };
  }
}

// ─── Intervention comparison ──────────────────────────────────────────────────
export class InterventionComparison {
  static presets() {
    return {
      'Default (no intervention)': { params: DamageParams.default(), color: '#94a3b8', dash: false },
      'Progeria (×5 damage)':      { params: DamageParams.progeria(), color: '#ef4444', dash: true },
      'Longevity (×0.6 damage)':   { params: DamageParams.longevity(), color: '#34d399', dash: true },
      'Senolytics':                { params: DamageParams.senolytics(), color: '#22d3ee', dash: true },
      'NAD+ Supplementation':      { params: DamageParams.nadplus(), color: '#d4af37', dash: true },
      'Caloric Restriction':       { params: DamageParams.caloric_restriction(), color: '#a78bfa', dash: true },
    };
  }

  static run(selected_names, max_years = 120) {
    const presets = InterventionComparison.presets();
    return selected_names.map(name => {
      const preset = presets[name];
      if (!preset) return null;
      const cell = new CellState(preset.params);
      const snapshots = cell.runLifetime(max_years);
      const death_age = snapshots.find(s => !s.is_alive)?.age ?? max_years;
      return { name, snapshots, death_age, color: preset.color, dash: preset.dash };
    }).filter(Boolean);
  }
}

// ─── Inducer single-cell step-by-step ────────────────────────────────────────
export class InducerSimulator {
  constructor(params = {}) {
    this.m = params.m ?? CDATA.INDUCER_M_INIT;
    this.d = params.d ?? CDATA.INDUCER_D_INIT;
    this.detach_prob = params.detach_prob ?? CDATA.BASE_DETACH_PROB;
    this.mother_bias = params.mother_bias ?? CDATA.MOTHER_BIAS;
    this.history = [{ m: this.m, d: this.d, potency: getPotency(this.m, this.d) }];
    this.divisions = 0;
  }

  step(o2_penetration = 0.5) {
    const eff_prob = this.detach_prob * o2_penetration;
    if (Math.random() < eff_prob) {
      if (this.m > 0 && this.d > 0) {
        if (Math.random() < this.mother_bias) this.m--;
        else this.d--;
      } else if (this.m > 0) {
        this.m--;
      } else if (this.d > 0) {
        this.d--;
      }
    }
    this.history.push({ m: this.m, d: this.d, potency: getPotency(this.m, this.d) });
  }

  divide() {
    // Daughter inherits current counts
    this.divisions++;
    return new InducerSimulator({ m: this.m, d: this.d, detach_prob: this.detach_prob, mother_bias: this.mother_bias });
  }

  get potency() { return getPotency(this.m, this.d); }
}

// ─── Cell cycle phases ────────────────────────────────────────────────────────
export class CellCycleSimulator {
  constructor() {
    this.phase = 'G1';
    this.progress = 0;
    this.cyclin_d = 0.3;
    this.cyclin_e = 0.0;
    this.p21 = 0.1;
    this.p16 = 0.05;
    this.arrested = false;
    this.arrest_cause = null;
    this.division_count = 0;
    this.history = [];
    this.step_count = 0;
  }

  /** damage ∈ [0,1], ros ∈ [0,1] */
  step(damage = 0, ros = 0) {
    this.step_count++;
    if (this.arrested) return;

    // Gene expression response to damage
    this.p21 = Math.min(1, 0.1 + damage * 0.8 + ros * 0.3);
    this.p16 = Math.min(1, 0.05 + damage * 0.6);

    // Arrest checks
    if (this.p21 > 0.7) { this.arrested = true; this.arrest_cause = 'p21→G1 arrest'; return; }
    if (this.p16 > 0.8) { this.arrested = true; this.arrest_cause = 'p16→Senescence'; return; }

    const speed = { G1: 0.015, S: 0.02, G2: 0.025, M: 0.04 };
    const cyclin_boost = this.phase === 'G1' ? this.cyclin_d * 0.3 : 0;
    this.progress += (speed[this.phase] ?? 0.015) + cyclin_boost;

    if (this.progress >= 1) {
      this.progress = 0;
      if (this.phase === 'G1') { this.phase = 'S'; this.cyclin_e = 0.8; }
      else if (this.phase === 'S') { this.phase = 'G2'; this.cyclin_e = 0; }
      else if (this.phase === 'G2') { this.phase = 'M'; }
      else if (this.phase === 'M') {
        this.phase = 'G1';
        this.division_count++;
        this.cyclin_d = Math.max(0.1, this.cyclin_d - damage * 0.05);
      }
    }

    this.history.push({ phase: this.phase, progress: this.progress, p21: this.p21, p16: this.p16, arrested: this.arrested });
  }
}
