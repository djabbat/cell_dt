//! Graphical interface for simulation parameter configuration
//! Extended version with validation, presets, export and history

use cell_dt_config::*;
use eframe::{egui, Frame};
use egui::{CentralPanel, Context, ScrollArea, Slider, Window, ComboBox};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::VecDeque;

// ==================== DATA STRUCTURES ====================

/// Application state
#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigAppState {
    // Main parameters
    pub config_file: String,
    pub config_format: String,
    pub simulation: SimulationConfig,
    
    // Modules
    pub centriole: CentrioleConfig,
    pub cell_cycle: CellCycleConfig,
    pub transcriptome: TranscriptomeConfig,
    pub asymmetric: AsymmetricDivisionConfig,
    pub stem_hierarchy: StemHierarchyConfig,
    pub io: IOConfig,
    pub viz: VisualizationConfig,
    
    // UI state
    pub selected_tab: Tab,
    pub show_save_dialog: bool,
    pub show_load_dialog: bool,
    pub show_preset_dialog: bool,
    pub show_export_dialog: bool,
    pub show_validation_dialog: bool,
    pub message: Option<String>,
    pub validation_errors: Vec<String>,
    
    // History (simplified version without self-references)
    pub history_states: VecDeque<ConfigAppState>,
    pub history_index: usize,
    pub max_history: usize,
    
    // Real-time visualization
    pub realtime_viz: RealtimeVisualization,
}

impl Default for ConfigAppState {
    fn default() -> Self {
        let mut states = VecDeque::new();
        let default_state = Self {
            config_file: "config.toml".to_string(),
            config_format: "toml".to_string(),
            simulation: SimulationConfig::default(),
            centriole: CentrioleConfig::default(),
            cell_cycle: CellCycleConfig::default(),
            transcriptome: TranscriptomeConfig::default(),
            asymmetric: AsymmetricDivisionConfig::default(),
            stem_hierarchy: StemHierarchyConfig::default(),
            io: IOConfig::default(),
            viz: VisualizationConfig::default(),
            selected_tab: Tab::Simulation,
            show_save_dialog: false,
            show_load_dialog: false,
            show_preset_dialog: false,
            show_export_dialog: false,
            show_validation_dialog: false,
            message: None,
            validation_errors: Vec::new(),
            history_states: VecDeque::new(),
            history_index: 0,
            max_history: 50,
            realtime_viz: RealtimeVisualization::default(),
        };
        states.push_back(default_state.clone());
        
        Self {
            history_states: states,
            history_index: 0,
            max_history: 50,
            ..default_state
        }
    }
}

impl ConfigAppState {
    pub fn push_history(&mut self) {
        // Remove states ahead of current index
        while self.history_states.len() > self.history_index + 1 {
            self.history_states.pop_back();
        }
        
        // Add current state to history
        self.history_states.push_back(self.clone());
        
        // Limit history size
        while self.history_states.len() > self.max_history {
            self.history_states.pop_front();
            self.history_index = self.history_index.saturating_sub(1);
        }
        
        self.history_index = self.history_states.len() - 1;
    }
    
    pub fn undo(&mut self) -> Option<ConfigAppState> {
        if self.history_index > 0 {
            self.history_index -= 1;
            Some(self.history_states[self.history_index].clone())
        } else {
            None
        }
    }
    
    pub fn redo(&mut self) -> Option<ConfigAppState> {
        if self.history_index + 1 < self.history_states.len() {
            self.history_index += 1;
            Some(self.history_states[self.history_index].clone())
        } else {
            None
        }
    }
    
    pub fn can_undo(&self) -> bool {
        self.history_index > 0
    }
    
    pub fn can_redo(&self) -> bool {
        self.history_index + 1 < self.history_states.len()
    }
}

// ==================== REAL-TIME VISUALIZATION ====================

/// Data for real-time visualization
#[derive(Clone, Serialize, Deserialize)]
pub struct RealtimeVisualization {
    pub enabled: bool,
    pub parameter_history: VecDeque<ParameterSnapshot>,
    pub max_history: usize,
    pub selected_parameters: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ParameterSnapshot {
    pub time: f64,
    pub values: std::collections::HashMap<String, f64>,
}

impl Default for RealtimeVisualization {
    fn default() -> Self {
        Self {
            enabled: false,
            parameter_history: VecDeque::new(),
            max_history: 100,
            selected_parameters: vec![
                "simulation.max_steps".to_string(),
                "centriole.acetylation_rate".to_string(),
                "cell_cycle.base_cycle_time".to_string(),
            ],
        }
    }
}

impl RealtimeVisualization {
    pub fn add_snapshot(&mut self, values: std::collections::HashMap<String, f64>, time: f64) {
        self.parameter_history.push_back(ParameterSnapshot { time, values });
        
        while self.parameter_history.len() > self.max_history {
            self.parameter_history.pop_front();
        }
    }
    
    pub fn extract_values(state: &ConfigAppState) -> std::collections::HashMap<String, f64> {
        let mut values = std::collections::HashMap::new();
        
        values.insert("simulation.max_steps".to_string(), state.simulation.max_steps as f64);
        values.insert("simulation.dt".to_string(), state.simulation.dt);
        values.insert("centriole.acetylation_rate".to_string(), state.centriole.acetylation_rate as f64);
        values.insert("centriole.oxidation_rate".to_string(), state.centriole.oxidation_rate as f64);
        values.insert("cell_cycle.base_cycle_time".to_string(), state.cell_cycle.base_cycle_time as f64);
        values.insert("cell_cycle.checkpoint_strictness".to_string(), state.cell_cycle.checkpoint_strictness as f64);
        values.insert("transcriptome.mutation_rate".to_string(), state.transcriptome.mutation_rate as f64);
        values.insert("asymmetric.asymmetric_probability".to_string(), state.asymmetric.asymmetric_probability as f64);
        
        values
    }
}

// ==================== PARAMETER VALIDATION ====================

/// Parameter validator
pub struct ParameterValidator;

impl ParameterValidator {
    pub fn validate_all(state: &ConfigAppState) -> Vec<String> {
        let mut errors = Vec::new();
        
        // Simulation
        if state.simulation.max_steps == 0 {
            errors.push("❌ Number of steps must be greater than 0".to_string());
        }
        if state.simulation.dt <= 0.0 {
            errors.push("❌ Time step must be positive".to_string());
        }
        if state.simulation.dt > 1.0 {
            errors.push("⚠️ Time step > 1.0 may cause instability".to_string());
        }
        
        // Centriole
        if state.centriole.enabled {
            if state.centriole.acetylation_rate < 0.0 || state.centriole.acetylation_rate > 0.1 {
                errors.push("❌ Acetylation rate must be in range [0, 0.1]".to_string());
            }
            if state.centriole.oxidation_rate < 0.0 || state.centriole.oxidation_rate > 0.1 {
                errors.push("❌ Oxidation rate must be in range [0, 0.1]".to_string());
            }
        }
        
        // Cell cycle
        if state.cell_cycle.enabled {
            if state.cell_cycle.base_cycle_time <= 0.0 {
                errors.push("❌ Cycle duration must be positive".to_string());
            }
            if state.cell_cycle.checkpoint_strictness < 0.0 || state.cell_cycle.checkpoint_strictness > 1.0 {
                errors.push("❌ Checkpoint strictness must be in [0, 1]".to_string());
            }
        }
        
        // Transcriptome
        if state.transcriptome.enabled
            && (state.transcriptome.mutation_rate < 0.0 || state.transcriptome.mutation_rate > 0.1)
        {
            errors.push("❌ Mutation rate must be in [0, 0.1]".to_string());
        }
        
        // Asymmetric division
        if state.asymmetric.enabled {
            let sum = state.asymmetric.asymmetric_probability + 
                     state.asymmetric.renewal_probability + 
                     state.asymmetric.diff_probability;
            if (sum - 1.0).abs() > 0.01 {
                errors.push("⚠️ Sum of division probabilities should be ~1.0".to_string());
            }
            if state.asymmetric.niche_capacity == 0 {
                errors.push("❌ Niche capacity must be > 0".to_string());
            }
        }
        
        errors
    }
    
    pub fn is_valid(state: &ConfigAppState) -> bool {
        Self::validate_all(state).is_empty()
    }
}

// ==================== CONFIGURATION PRESETS ====================

/// Configuration presets for different experiments
#[derive(Debug, Clone)]
pub struct ConfigPreset {
    pub name: String,
    pub description: String,
    pub icon: &'static str,
    pub apply: fn(&mut ConfigAppState),
}

impl ConfigPreset {
    pub fn get_all() -> Vec<Self> {
        vec![
            Self {
                name: "Quick Test".to_string(),
                description: "Minimal configuration for quick testing".to_string(),
                icon: "⚡",
                apply: |state| {
                    state.simulation.max_steps = 100;
                    state.simulation.dt = 0.1;
                    state.centriole.enabled = true;
                    state.cell_cycle.enabled = true;
                    state.transcriptome.enabled = false;
                },
            },
            Self {
                name: "Standard Experiment".to_string(),
                description: "Standard parameters for typical experiments".to_string(),
                icon: "🔬",
                apply: |state| {
                    state.simulation.max_steps = 10000;
                    state.simulation.dt = 0.05;
                    state.simulation.num_threads = Some(8);
                    state.centriole.enabled = true;
                    state.cell_cycle.enabled = true;
                    state.transcriptome.enabled = true;
                },
            },
            Self {
                name: "High Performance".to_string(),
                description: "Optimized for large populations".to_string(),
                icon: "🚀",
                apply: |state| {
                    state.simulation.max_steps = 100000;
                    state.simulation.dt = 0.1;
                    state.simulation.num_threads = Some(16);
                    state.simulation.parallel_modules = true;
                    state.centriole.parallel_cells = true;
                    state.io.save_checkpoints = false;
                    state.viz.enabled = false;
                },
            },
            Self {
                name: "Stem Cells".to_string(),
                description: "Focus on asymmetric division and hierarchy".to_string(),
                icon: "🌱",
                apply: |state| {
                    state.simulation.max_steps = 50000;
                    state.asymmetric.enabled = true;
                    state.asymmetric.asymmetric_probability = 0.4;
                    state.stem_hierarchy.enabled = true;
                    state.stem_hierarchy.initial_potency = "Pluripotent".to_string();
                    state.transcriptome.enabled = true;
                },
            },
            Self {
                name: "Cell Cycle".to_string(),
                description: "Detailed cell cycle study".to_string(),
                icon: "🔄",
                apply: |state| {
                    state.simulation.max_steps = 20000;
                    state.simulation.dt = 0.02;
                    state.cell_cycle.enabled = true;
                    state.cell_cycle.checkpoint_strictness = 0.3;
                    state.cell_cycle.enable_apoptosis = true;
                    state.centriole.enabled = true;
                    state.transcriptome.enabled = false;
                },
            },
            Self {
                name: "Transcriptome Analysis".to_string(),
                description: "Detailed gene expression analysis".to_string(),
                icon: "🧬",
                apply: |state| {
                    state.simulation.max_steps = 5000;
                    state.simulation.dt = 0.05;
                    state.transcriptome.enabled = true;
                    state.transcriptome.mutation_rate = 0.001;
                    state.transcriptome.noise_level = 0.02;
                    state.io.format = "parquet".to_string();
                    state.io.compression = "snappy".to_string();
                },
            },
        ]
    }
}

// ==================== PYTHON EXPORT ====================

/// Python script generator
pub struct PythonExporter;

impl PythonExporter {
    pub fn generate_script(state: &ConfigAppState) -> String {
        let mut script = String::new();
        
        script.push_str("#!/usr/bin/env python3\n");
        script.push_str("# -*- coding: utf-8 -*-\n");
        script.push_str("\"\"\"\n");
        script.push_str("Automatically generated script for Cell DT\n");
        script.push_str("Usage: python3 script.py\n");
        script.push_str("\"\"\"\n\n");
        
        script.push_str("import cell_dt\n");
        script.push_str("import numpy as np\n");
        script.push_str("import matplotlib.pyplot as plt\n\n");
        
        // Simulation setup
        script.push_str("# Simulation setup\n");
        script.push_str("sim = cell_dt.PySimulation(\n");
        script.push_str(&format!("    max_steps={},\n", state.simulation.max_steps));
        script.push_str(&format!("    dt={},\n", state.simulation.dt));
        script.push_str(&format!("    num_threads={},\n", state.simulation.num_threads.unwrap_or(1)));
        script.push_str(&format!("    seed={}\n", state.simulation.seed.unwrap_or(42)));
        script.push_str(")\n\n");
        
        // Create cells
        script.push_str("# Create cells\n");
        if state.transcriptome.enabled {
            script.push_str("sim.create_population_with_transcriptome(100)\n");
        } else {
            script.push_str("sim.create_population(100)\n");
        }
        script.push('\n');
        
        // Cell cycle parameters
        if state.cell_cycle.enabled {
            script.push_str("# Cell cycle parameters\n");
            script.push_str("cell_cycle_params = cell_dt.PyCellCycleParams(\n");
            script.push_str(&format!("    base_cycle_time={},\n", state.cell_cycle.base_cycle_time));
            script.push_str(&format!("    checkpoint_strictness={},\n", state.cell_cycle.checkpoint_strictness));
            script.push_str(&format!("    enable_apoptosis={},\n", state.cell_cycle.enable_apoptosis));
            script.push_str(&format!("    nutrient_availability={},\n", state.cell_cycle.nutrient_availability));
            script.push_str(&format!("    growth_factor_level={},\n", state.cell_cycle.growth_factor_level));
            script.push_str(&format!("    random_variation={}\n", state.cell_cycle.random_variation));
            script.push_str(")\n\n");
        } else {
            script.push_str("cell_cycle_params = None\n\n");
        }
        
        // Register modules
        script.push_str("# Register modules\n");
        script.push_str("sim.register_modules(\n");
        script.push_str(&format!("    enable_centriole={},\n", state.centriole.enabled));
        script.push_str(&format!("    enable_cell_cycle={},\n", state.cell_cycle.enabled));
        script.push_str(&format!("    enable_transcriptome={},\n", state.transcriptome.enabled));
        script.push_str("    cell_cycle_params=cell_cycle_params\n");
        script.push_str(")\n\n");
        
        // Run simulation
        script.push_str("# Run simulation\n");
        script.push_str("print(\"🚀 Starting simulation...\")\n");
        script.push_str("cells = sim.run()\n");
        script.push_str("print(f\"✅ Simulation completed in {sim.current_step()} steps\")\n\n");
        
        // Analyze results
        script.push_str("# Analyze results\n");
        script.push_str("print(f\"\\nTotal cells: {len(cells)}\")\n\n");
        
        script.push_str("# Get centriole data\n");
        script.push_str("centriole_data = sim.get_centriole_data_numpy()\n");
        script.push_str("if len(centriole_data) > 0:\n");
        script.push_str("    print(f\"Average mother centriole maturity: {np.mean(centriole_data[:, 0]):.3f}\")\n\n");
        
        script.push_str("# Phase distribution\n");
        script.push_str("phase_dist = sim.get_phase_distribution()\n");
        script.push_str("print(\"\\nPhase distribution:\")\n");
        script.push_str("for phase, count in phase_dist.items():\n");
        script.push_str("    print(f\"  {phase}: {count}\")\n\n");
        
        script.push_str("# Visualization\n");
        script.push_str("if len(phase_dist) > 0:\n");
        script.push_str("    plt.figure(figsize=(10, 6))\n");
        script.push_str("    plt.bar(phase_dist.keys(), phase_dist.values())\n");
        script.push_str("    plt.title('Cell Cycle Phase Distribution')\n");
        script.push_str("    plt.xlabel('Phase')\n");
        script.push_str("    plt.ylabel('Number of Cells')\n");
        script.push_str("    plt.savefig('phase_distribution.png')\n");
        script.push_str("    plt.show()\n");
        
        script
    }
}

// ==================== TABS ====================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tab {
    Simulation,
    Centriole,
    CellCycle,
    Transcriptome,
    Asymmetric,
    StemHierarchy,
    IO,
    Visualization,
}

impl Tab {
    pub fn name(&self) -> &'static str {
        match self {
            Tab::Simulation => "⚙️ Simulation",
            Tab::Centriole => "🔬 Centriole",
            Tab::CellCycle => "🔄 Cell Cycle",
            Tab::Transcriptome => "🧬 Transcriptome",
            Tab::Asymmetric => "⚖️ Asymmetric Division",
            Tab::StemHierarchy => "🌱 Stem Hierarchy",
            Tab::IO => "💾 I/O",
            Tab::Visualization => "📊 Visualization",
        }
    }
}

// ==================== CONFIGURATIONS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsymmetricDivisionConfig {
    pub enabled: bool,
    pub asymmetric_probability: f32,
    pub renewal_probability: f32,
    pub diff_probability: f32,
    pub niche_capacity: usize,
    pub max_niches: usize,
    pub enable_polarity: bool,
    pub enable_fate_determinants: bool,
}

impl Default for AsymmetricDivisionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            asymmetric_probability: 0.3,
            renewal_probability: 0.4,
            diff_probability: 0.3,
            niche_capacity: 10,
            max_niches: 100,
            enable_polarity: true,
            enable_fate_determinants: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StemHierarchyConfig {
    pub enabled: bool,
    pub initial_potency: String,
    pub enable_plasticity: bool,
    pub plasticity_rate: f32,
    pub differentiation_threshold: f32,
}

impl Default for StemHierarchyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            initial_potency: "Pluripotent".to_string(),
            enable_plasticity: true,
            plasticity_rate: 0.01,
            differentiation_threshold: 0.7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IOConfig {
    pub enabled: bool,
    pub output_dir: String,
    pub format: String,
    pub compression: String,
    pub buffer_size: usize,
    pub save_checkpoints: bool,
    pub checkpoint_interval: u64,
    pub max_checkpoints: usize,
}

impl Default for IOConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            output_dir: "results".to_string(),
            format: "csv".to_string(),
            compression: "none".to_string(),
            buffer_size: 1000,
            save_checkpoints: true,
            checkpoint_interval: 100,
            max_checkpoints: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub enabled: bool,
    pub update_interval: u64,
    pub output_dir: String,
    pub save_plots: bool,
    pub phase_distribution: bool,
    pub maturity_histogram: bool,
    pub heatmap: bool,
    pub timeseries: bool,
    pub three_d_enabled: bool,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            update_interval: 10,
            output_dir: "viz_output".to_string(),
            save_plots: true,
            phase_distribution: true,
            maturity_histogram: true,
            heatmap: true,
            timeseries: true,
            three_d_enabled: false,
        }
    }
}

// ==================== MAIN APPLICATION ====================

pub struct ConfigApp {
    state: ConfigAppState,
}

impl ConfigApp {
    pub fn new() -> Self {
        Self {
            state: ConfigAppState::default(),
        }
    }
}

impl Default for ConfigApp {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for ConfigApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // Top panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🧬 Cell DT - Simulation Configurator");
                ui.separator();
                
                // History buttons
                ui.add_enabled_ui(self.state.can_undo(), |ui| {
                    if ui.button("↩️ Undo").clicked() {
                        if let Some(prev_state) = self.state.undo() {
                            self.state = prev_state;
                        }
                    }
                });
                
                ui.add_enabled_ui(self.state.can_redo(), |ui| {
                    if ui.button("↪️ Redo").clicked() {
                        if let Some(next_state) = self.state.redo() {
                            self.state = next_state;
                        }
                    }
                });
                
                ui.separator();
                
                if ui.button("📂 Load").clicked() {
                    self.state.show_load_dialog = true;
                }
                
                if ui.button("💾 Save").clicked() {
                    self.state.show_save_dialog = true;
                }
                
                if ui.button("📋 Presets").clicked() {
                    self.state.show_preset_dialog = true;
                }
                
                if ui.button("🐍 Export to Python").clicked() {
                    self.state.show_export_dialog = true;
                }
                
                if ui.button("✓ Validate").clicked() {
                    self.state.validation_errors = ParameterValidator::validate_all(&self.state);
                    self.state.show_validation_dialog = true;
                }
                
                ui.separator();
                
                if ui.button("❌ Exit").clicked() {
                    std::process::exit(0);
                }
            });
            
            if let Some(msg) = &self.state.message {
                ui.horizontal(|ui| {
                    ui.label(format!("Status: {}", msg));
                });
            }
        });
        
        // Left panel with tabs
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Modules");
                ui.separator();
                
                let tabs = [
                    Tab::Simulation,
                    Tab::Centriole,
                    Tab::CellCycle,
                    Tab::Transcriptome,
                    Tab::Asymmetric,
                    Tab::StemHierarchy,
                    Tab::IO,
                    Tab::Visualization,
                ];
                
                for tab in tabs {
                    if ui.selectable_value(&mut self.state.selected_tab, tab, tab.name()).clicked() {
                        self.state.push_history();
                    }
                }
            });
        });
        
        // Right panel with real-time visualization
        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("📈 Real-time Visualization");
                ui.separator();
                
                ui.checkbox(&mut self.state.realtime_viz.enabled, "Enable");
                
                if self.state.realtime_viz.enabled {
                    // Extract values and add snapshot
                    let values = RealtimeVisualization::extract_values(&self.state);
                    self.state.realtime_viz.add_snapshot(values, 0.0);
                    
                    // Display graphs
                    for param in &self.state.realtime_viz.selected_parameters {
                        ui.label(format!("📊 {}", param));
                        
                        // Collect data for graph
                        let mut values = Vec::new();
                        for snapshot in &self.state.realtime_viz.parameter_history {
                            if let Some(value) = snapshot.values.get(param) {
                                values.push(*value);
                            }
                        }
                        
                        if !values.is_empty() {
                            // Simple line graph
                            ui.horizontal(|ui| {
                                ui.label(format!("Current: {:.3}", values.last().unwrap()));
                            });
                        }
                    }
                    
                    ui.collapsing("⚙️ Settings", |ui| {
                        ui.label("Select parameters to display:");
                        // Here you can add checkboxes for parameter selection
                    });
                }
            });
        });
        
        // Central panel
        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                match self.state.selected_tab {
                    Tab::Simulation => self.show_simulation_tab(ui),
                    Tab::Centriole => self.show_centriole_tab(ui),
                    Tab::CellCycle => self.show_cell_cycle_tab(ui),
                    Tab::Transcriptome => self.show_transcriptome_tab(ui),
                    Tab::Asymmetric => self.show_asymmetric_tab(ui),
                    Tab::StemHierarchy => self.show_stem_hierarchy_tab(ui),
                    Tab::IO => self.show_io_tab(ui),
                    Tab::Visualization => self.show_visualization_tab(ui),
                }
            });
        });
        
        // Dialogs
        if self.state.show_save_dialog {
            self.show_save_dialog(ctx);
        }
        
        if self.state.show_load_dialog {
            self.show_load_dialog(ctx);
        }
        
        if self.state.show_preset_dialog {
            self.show_preset_dialog(ctx);
        }
        
        if self.state.show_export_dialog {
            self.show_export_dialog(ctx);
        }
        
        if self.state.show_validation_dialog {
            self.show_validation_dialog(ctx);
        }
    }
}

// ==================== TAB IMPLEMENTATIONS ====================

impl ConfigApp {
    fn show_simulation_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚙️ Main Simulation Parameters");
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("Number of steps:");
            if ui.add(Slider::new(&mut self.state.simulation.max_steps, 1..=1_000_000)).changed() {
                self.state.push_history();
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("Time step (dt):");
            if ui.add(Slider::new(&mut self.state.simulation.dt, 0.001..=1.0).logarithmic(true)).changed() {
                self.state.push_history();
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("Checkpoint interval:");
            if ui.add(Slider::new(&mut self.state.simulation.checkpoint_interval, 1..=10_000)).changed() {
                self.state.push_history();
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("Number of threads:");
            let mut threads = self.state.simulation.num_threads.unwrap_or(1);
            if ui.add(Slider::new(&mut threads, 1..=64)).changed() {
                self.state.simulation.num_threads = Some(threads);
                self.state.push_history();
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("Random seed:");
            let mut seed = self.state.simulation.seed.unwrap_or(42);
            if ui.add(Slider::new(&mut seed, 0..=999_999)).changed() {
                self.state.simulation.seed = Some(seed);
                self.state.push_history();
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("Output directory:");
            let output_str = self.state.simulation.output_dir.to_string_lossy().to_string();
            let mut output = output_str.clone();
            if ui.text_edit_singleline(&mut output).changed()
                && output != output_str
            {
                self.state.simulation.output_dir = PathBuf::from(output);
                self.state.push_history();
            }
        });
        
        if ui.checkbox(&mut self.state.simulation.parallel_modules, "Parallel module execution").changed() {
            self.state.push_history();
        }
    }
    
    fn show_centriole_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔬 Centriole Module");
        ui.separator();
        
        if ui.checkbox(&mut self.state.centriole.enabled, "Enable module").changed() {
            self.state.push_history();
        }
        
        if self.state.centriole.enabled {
            ui.horizontal(|ui| {
                ui.label("Acetylation rate:");
                if ui.add(Slider::new(&mut self.state.centriole.acetylation_rate, 0.0..=0.1)).changed() {
                    self.state.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Oxidation rate:");
                if ui.add(Slider::new(&mut self.state.centriole.oxidation_rate, 0.0..=0.1)).changed() {
                    self.state.push_history();
                }
            });
            
            if ui.checkbox(&mut self.state.centriole.parallel_cells, "Parallel cell processing").changed() {
                self.state.push_history();
            }
        }
    }
    
    fn show_cell_cycle_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔄 Cell Cycle Module");
        ui.separator();
        
        if ui.checkbox(&mut self.state.cell_cycle.enabled, "Enable module").changed() {
            self.state.push_history();
        }
        
        if self.state.cell_cycle.enabled {
            ui.horizontal(|ui| {
                ui.label("Base cycle duration:");
                if ui.add(Slider::new(&mut self.state.cell_cycle.base_cycle_time, 1.0..=100.0)).changed() {
                    self.state.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Checkpoint strictness:");
                if ui.add(Slider::new(&mut self.state.cell_cycle.checkpoint_strictness, 0.0..=1.0)).changed() {
                    self.state.push_history();
                }
            });
            
            if ui.checkbox(&mut self.state.cell_cycle.enable_apoptosis, "Enable apoptosis").changed() {
                self.state.push_history();
            }
            
            ui.horizontal(|ui| {
                ui.label("Nutrient availability:");
                if ui.add(Slider::new(&mut self.state.cell_cycle.nutrient_availability, 0.0..=1.0)).changed() {
                    self.state.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Growth factor level:");
                if ui.add(Slider::new(&mut self.state.cell_cycle.growth_factor_level, 0.0..=1.0)).changed() {
                    self.state.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Random variation:");
                if ui.add(Slider::new(&mut self.state.cell_cycle.random_variation, 0.0..=1.0)).changed() {
                    self.state.push_history();
                }
            });
        }
    }
    
    fn show_transcriptome_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🧬 Transcriptome Module");
        ui.separator();
        
        if ui.checkbox(&mut self.state.transcriptome.enabled, "Enable module").changed() {
            self.state.push_history();
        }
        
        if self.state.transcriptome.enabled {
            ui.horizontal(|ui| {
                ui.label("Mutation rate:");
                if ui.add(Slider::new(&mut self.state.transcriptome.mutation_rate, 0.0..=0.01).logarithmic(true)).changed() {
                    self.state.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Noise level:");
                if ui.add(Slider::new(&mut self.state.transcriptome.noise_level, 0.0..=0.5)).changed() {
                    self.state.push_history();
                }
            });
        }
    }
    
    fn show_asymmetric_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚖️ Asymmetric Division Module");
        ui.separator();
        
        if ui.checkbox(&mut self.state.asymmetric.enabled, "Enable module").changed() {
            self.state.push_history();
        }
        
        if self.state.asymmetric.enabled {
            ui.horizontal(|ui| {
                ui.label("Asymmetric division probability:");
                if ui.add(Slider::new(&mut self.state.asymmetric.asymmetric_probability, 0.0..=1.0)).changed() {
                    self.state.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Self-renewal probability:");
                if ui.add(Slider::new(&mut self.state.asymmetric.renewal_probability, 0.0..=1.0)).changed() {
                    self.state.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Differentiation probability:");
                if ui.add(Slider::new(&mut self.state.asymmetric.diff_probability, 0.0..=1.0)).changed() {
                    self.state.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Niche capacity:");
                if ui.add(Slider::new(&mut self.state.asymmetric.niche_capacity, 1..=100)).changed() {
                    self.state.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Maximum niches:");
                if ui.add(Slider::new(&mut self.state.asymmetric.max_niches, 1..=1000)).changed() {
                    self.state.push_history();
                }
            });
            
            if ui.checkbox(&mut self.state.asymmetric.enable_polarity, "Enable polarity").changed() {
                self.state.push_history();
            }
            
            if ui.checkbox(&mut self.state.asymmetric.enable_fate_determinants, "Enable fate determinants").changed() {
                self.state.push_history();
            }
        }
    }
    
    fn show_stem_hierarchy_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🌱 Stem Cell Hierarchy Module");
        ui.separator();
        
        if ui.checkbox(&mut self.state.stem_hierarchy.enabled, "Enable module").changed() {
            self.state.push_history();
        }
        
        if self.state.stem_hierarchy.enabled {
            ui.horizontal(|ui| {
                ui.label("Initial potency level:");
                ComboBox::from_id_source("potency")
                    .selected_text(&self.state.stem_hierarchy.initial_potency)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.state.stem_hierarchy.initial_potency, 
                            "Totipotent".to_string(), "Totipotent");
                        ui.selectable_value(&mut self.state.stem_hierarchy.initial_potency, 
                            "Pluripotent".to_string(), "Pluripotent");
                        ui.selectable_value(&mut self.state.stem_hierarchy.initial_potency, 
                            "Multipotent".to_string(), "Multipotent");
                        ui.selectable_value(&mut self.state.stem_hierarchy.initial_potency, 
                            "Differentiated".to_string(), "Differentiated");
                    });
                self.state.push_history();
            });
            
            if ui.checkbox(&mut self.state.stem_hierarchy.enable_plasticity, "Enable plasticity").changed() {
                self.state.push_history();
            }
            
            ui.horizontal(|ui| {
                ui.label("Plasticity rate:");
                if ui.add(Slider::new(&mut self.state.stem_hierarchy.plasticity_rate, 0.0..=0.1).logarithmic(true)).changed() {
                    self.state.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Differentiation threshold:");
                if ui.add(Slider::new(&mut self.state.stem_hierarchy.differentiation_threshold, 0.0..=1.0)).changed() {
                    self.state.push_history();
                }
            });
        }
    }
    
    fn show_io_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("💾 I/O Module");
        ui.separator();
        
        if ui.checkbox(&mut self.state.io.enabled, "Enable module").changed() {
            self.state.push_history();
        }
        
        if self.state.io.enabled {
            ui.horizontal(|ui| {
                ui.label("Output directory:");
                if ui.text_edit_singleline(&mut self.state.io.output_dir).changed() {
                    self.state.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Format:");
                ComboBox::from_id_source("format")
                    .selected_text(&self.state.io.format)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.state.io.format, "csv".to_string(), "CSV");
                        ui.selectable_value(&mut self.state.io.format, "parquet".to_string(), "Parquet");
                        ui.selectable_value(&mut self.state.io.format, "hdf5".to_string(), "HDF5");
                    });
                self.state.push_history();
            });
            
            ui.horizontal(|ui| {
                ui.label("Compression:");
                ComboBox::from_id_source("compression")
                    .selected_text(&self.state.io.compression)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.state.io.compression, "none".to_string(), "None");
                        ui.selectable_value(&mut self.state.io.compression, "snappy".to_string(), "Snappy");
                        ui.selectable_value(&mut self.state.io.compression, "gzip".to_string(), "Gzip");
                    });
                self.state.push_history();
            });
            
            ui.horizontal(|ui| {
                ui.label("Buffer size:");
                if ui.add(Slider::new(&mut self.state.io.buffer_size, 100..=10000)).changed() {
                    self.state.push_history();
                }
            });
            
            if ui.checkbox(&mut self.state.io.save_checkpoints, "Save checkpoints").changed() {
                self.state.push_history();
            }
            
            if self.state.io.save_checkpoints {
                ui.horizontal(|ui| {
                    ui.label("Checkpoint interval:");
                    if ui.add(Slider::new(&mut self.state.io.checkpoint_interval, 10..=1000)).changed() {
                        self.state.push_history();
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Maximum checkpoints:");
                    if ui.add(Slider::new(&mut self.state.io.max_checkpoints, 1..=100)).changed() {
                        self.state.push_history();
                    }
                });
            }
        }
    }
    
    fn show_visualization_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("📊 Visualization Module");
        ui.separator();
        
        if ui.checkbox(&mut self.state.viz.enabled, "Enable module").changed() {
            self.state.push_history();
        }
        
        if self.state.viz.enabled {
            ui.horizontal(|ui| {
                ui.label("Update interval:");
                if ui.add(Slider::new(&mut self.state.viz.update_interval, 1..=100)).changed() {
                    self.state.push_history();
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Output directory:");
                if ui.text_edit_singleline(&mut self.state.viz.output_dir).changed() {
                    self.state.push_history();
                }
            });
            
            if ui.checkbox(&mut self.state.viz.save_plots, "Save plots").changed() {
                self.state.push_history();
            }
            
            ui.collapsing("📈 Plot types", |ui| {
                if ui.checkbox(&mut self.state.viz.phase_distribution, "Phase distribution").changed() {
                    self.state.push_history();
                }
                if ui.checkbox(&mut self.state.viz.maturity_histogram, "Maturity histogram").changed() {
                    self.state.push_history();
                }
                if ui.checkbox(&mut self.state.viz.heatmap, "Heatmap").changed() {
                    self.state.push_history();
                }
                if ui.checkbox(&mut self.state.viz.timeseries, "Time series").changed() {
                    self.state.push_history();
                }
                if ui.checkbox(&mut self.state.viz.three_d_enabled, "3D visualization").changed() {
                    self.state.push_history();
                }
            });
        }
    }
    
    // ==================== DIALOGS ====================
    
    fn show_save_dialog(&mut self, ctx: &Context) {
        let mut open = true;
        Window::new("💾 Save Configuration")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Filename:");
                    ui.text_edit_singleline(&mut self.state.config_file);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Format:");
                    ui.radio_value(&mut self.state.config_format, "toml".to_string(), "TOML");
                    ui.radio_value(&mut self.state.config_format, "yaml".to_string(), "YAML");
                    ui.radio_value(&mut self.state.config_format, "json".to_string(), "JSON");
                });
                
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        self.state.message = Some(format!("✅ Saved: {}", self.state.config_file));
                        self.state.show_save_dialog = false;
                    }
                    
                    if ui.button("Cancel").clicked() {
                        self.state.show_save_dialog = false;
                    }
                });
            });
        
        if !open {
            self.state.show_save_dialog = false;
        }
    }
    
    fn show_load_dialog(&mut self, ctx: &Context) {
        let mut open = true;
        Window::new("📂 Load Configuration")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Filename:");
                    ui.text_edit_singleline(&mut self.state.config_file);
                });
                
                ui.collapsing("📁 Available configurations", |ui| {
                    ui.label("configs/example.toml");
                    ui.label("configs/development.toml");
                    ui.label("configs/production.toml");
                    ui.label("configs/benchmark.toml");
                });
                
                ui.horizontal(|ui| {
                    if ui.button("Load").clicked() {
                        self.state.message = Some(format!("✅ Loaded: {}", self.state.config_file));
                        self.state.show_load_dialog = false;
                        self.state.push_history();
                    }
                    
                    if ui.button("Cancel").clicked() {
                        self.state.show_load_dialog = false;
                    }
                });
            });
        
        if !open {
            self.state.show_load_dialog = false;
        }
    }
    
    fn show_preset_dialog(&mut self, ctx: &Context) {
        let mut open = true;
        Window::new("📋 Configuration Presets")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label("Select a preset configuration:");
                ui.separator();
                
                let presets = ConfigPreset::get_all();
                
                for preset in presets {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} {}", preset.icon, preset.name));
                        if ui.button("Apply").clicked() {
                            (preset.apply)(&mut self.state);
                            self.state.message = Some(format!("✅ Applied preset: {}", preset.name));
                            self.state.show_preset_dialog = false;
                            self.state.push_history();
                        }
                    });
                    ui.label(format!("   {}", preset.description));
                    ui.separator();
                }
                
                if ui.button("Close").clicked() {
                    self.state.show_preset_dialog = false;
                }
            });
        
        if !open {
            self.state.show_preset_dialog = false;
        }
    }
    
    fn show_export_dialog(&mut self, ctx: &Context) {
        let mut open = true;
        let script = PythonExporter::generate_script(&self.state);
        
        Window::new("🐍 Export to Python")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label("Generated Python script:");
                ui.separator();
                
                ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        ui.label(script.as_str());
                    });
                
                ui.horizontal(|ui| {
                    if ui.button("📋 Copy to clipboard").clicked() {
                        ui.ctx().copy_text(script);
                        self.state.message = Some("✅ Script copied to clipboard".to_string());
                    }
                    
                    if ui.button("💾 Save as script.py").clicked() {
                        // Here you would save to file
                        self.state.message = Some("✅ Script saved".to_string());
                    }
                    
                    if ui.button("Close").clicked() {
                        self.state.show_export_dialog = false;
                    }
                });
            });
        
        if !open {
            self.state.show_export_dialog = false;
        }
    }
    
    fn show_validation_dialog(&mut self, ctx: &Context) {
        let mut open = true;
        let errors = &self.state.validation_errors.clone();
        
        Window::new("✓ Parameter Validation")
            .open(&mut open)
            .show(ctx, |ui| {
                if errors.is_empty() {
                    ui.label("✅ All parameters are valid!");
                } else {
                    ui.label("❌ Found issues:");
                    ui.separator();
                    for error in errors {
                        ui.label(error);
                    }
                }
                
                ui.separator();
                
                if ui.button("Close").clicked() {
                    self.state.show_validation_dialog = false;
                }
            });
        
        if !open {
            self.state.show_validation_dialog = false;
        }
    }
}
