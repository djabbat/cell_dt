use cell_dt_config::*;
use tempfile::tempdir;

// ==================== DEFAULT / ROUNDTRIP ====================

#[test]
fn test_default_config() {
    let config = FullConfig::default();
    assert!(config.centriole_module.enabled);
    assert!(config.cell_cycle_module.enabled);
    assert_eq!(config.simulation.max_steps, 10000);
}

#[test]
fn test_save_load_toml() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.toml").to_str().unwrap().to_string();

    let config = FullConfig::default();
    ConfigLoader::save_toml(&config, &path).unwrap();

    let loaded = ConfigLoader::from_toml(&path).unwrap();
    assert_eq!(loaded.simulation.max_steps, config.simulation.max_steps);
    assert_eq!(loaded.centriole_module.acetylation_rate, config.centriole_module.acetylation_rate);
}

#[test]
fn test_save_load_yaml() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.yaml").to_str().unwrap().to_string();

    let config = FullConfig::default();
    ConfigLoader::save_yaml(&config, &path).unwrap();

    let loaded = ConfigLoader::from_yaml(&path).unwrap();
    assert_eq!(loaded.simulation.max_steps, config.simulation.max_steps);
    assert_eq!(loaded.transcriptome_module.mutation_rate, config.transcriptome_module.mutation_rate);
}

// ==================== VALIDATE — VALID ====================

#[test]
fn test_validate_default_config_is_valid() {
    let config = FullConfig::default();
    let errors = config.validate();
    assert!(errors.is_empty(), "default config should be valid, got: {:?}", errors);
}

// ==================== VALIDATE — SIMULATION ====================

#[test]
fn test_validate_dt_zero() {
    let mut config = FullConfig::default();
    config.simulation.dt = 0.0;
    let errors = config.validate();
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.contains("dt")));
}

#[test]
fn test_validate_dt_negative() {
    let mut config = FullConfig::default();
    config.simulation.dt = -1.0;
    let errors = config.validate();
    assert!(errors.iter().any(|e| e.contains("dt")));
}

#[test]
fn test_validate_max_steps_zero() {
    let mut config = FullConfig::default();
    config.simulation.max_steps = 0;
    let errors = config.validate();
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.contains("max_steps")));
}

#[test]
fn test_validate_checkpoint_interval_zero() {
    let mut config = FullConfig::default();
    config.simulation.checkpoint_interval = 0;
    let errors = config.validate();
    assert!(errors.iter().any(|e| e.contains("checkpoint_interval")));
}

// ==================== VALIDATE — CENTRIOLE ====================

#[test]
fn test_validate_acetylation_rate_too_high() {
    let mut config = FullConfig::default();
    config.centriole_module.acetylation_rate = 1.5;
    let errors = config.validate();
    assert!(errors.iter().any(|e| e.contains("acetylation_rate")));
}

#[test]
fn test_validate_acetylation_rate_negative() {
    let mut config = FullConfig::default();
    config.centriole_module.acetylation_rate = -0.01;
    let errors = config.validate();
    assert!(errors.iter().any(|e| e.contains("acetylation_rate")));
}

#[test]
fn test_validate_oxidation_rate_too_high() {
    let mut config = FullConfig::default();
    config.centriole_module.oxidation_rate = 2.0;
    let errors = config.validate();
    assert!(errors.iter().any(|e| e.contains("oxidation_rate")));
}

#[test]
fn test_validate_centriole_disabled_skips_checks() {
    let mut config = FullConfig::default();
    config.centriole_module.enabled = false;
    config.centriole_module.acetylation_rate = 999.0; // invalid but ignored
    let errors = config.validate();
    assert!(!errors.iter().any(|e| e.contains("acetylation_rate")));
}

// ==================== VALIDATE — CELL CYCLE ====================

#[test]
fn test_validate_base_cycle_time_zero() {
    let mut config = FullConfig::default();
    config.cell_cycle_module.base_cycle_time = 0.0;
    let errors = config.validate();
    assert!(errors.iter().any(|e| e.contains("base_cycle_time")));
}

#[test]
fn test_validate_checkpoint_strictness_out_of_range() {
    let mut config = FullConfig::default();
    config.cell_cycle_module.checkpoint_strictness = -0.1;
    let errors = config.validate();
    assert!(errors.iter().any(|e| e.contains("checkpoint_strictness")));
}

#[test]
fn test_validate_cell_cycle_disabled_skips_checks() {
    let mut config = FullConfig::default();
    config.cell_cycle_module.enabled = false;
    config.cell_cycle_module.base_cycle_time = -1.0; // invalid but ignored
    let errors = config.validate();
    assert!(!errors.iter().any(|e| e.contains("base_cycle_time")));
}

// ==================== VALIDATE — TRANSCRIPTOME ====================

#[test]
fn test_validate_mutation_rate_out_of_range() {
    let mut config = FullConfig::default();
    config.transcriptome_module.mutation_rate = 1.5;
    let errors = config.validate();
    assert!(errors.iter().any(|e| e.contains("mutation_rate")));
}

#[test]
fn test_validate_noise_level_negative() {
    let mut config = FullConfig::default();
    config.transcriptome_module.noise_level = -0.1;
    let errors = config.validate();
    assert!(errors.iter().any(|e| e.contains("noise_level")));
}

// ==================== VALIDATE — MULTIPLE ERRORS ====================

#[test]
fn test_validate_multiple_errors_accumulated() {
    let mut config = FullConfig::default();
    config.simulation.dt = 0.0;
    config.simulation.max_steps = 0;
    let errors = config.validate();
    assert!(errors.len() >= 2, "expected at least 2 errors, got {}", errors.len());
}

// ==================== FROM_TOML WITH INVALID CONFIG ====================

#[test]
fn test_from_toml_invalid_config_returns_error() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("invalid.toml").to_str().unwrap().to_string();

    // Save a complete config with dt = 0.0 (fails validation)
    let mut config = FullConfig::default();
    config.simulation.dt = 0.0;
    ConfigLoader::save_toml(&config, &path).unwrap();

    let result = ConfigLoader::from_toml(&path);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("dt") || msg.contains("Invalid"), "error message was: {}", msg);
}
