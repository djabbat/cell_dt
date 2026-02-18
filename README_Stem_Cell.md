# Cell DT - Stem Cell Biology Modules 🧬

## 📋 Overview

This document covers the advanced stem cell biology modules implemented in the Cell DT platform, including asymmetric division and stem cell hierarchy.

## 🔬 Modules Overview

### 11. **Asymmetric Division Module**

The asymmetric division module simulates how stem cells divide to produce both stem cells and differentiated progeny.

#### Key Features

- **Division Types**: Symmetric, asymmetric, self-renewal, and differentiation
- **Stem Cell Niches**: Spatial organization of stem cell microenvironments
- **Fate Determinants**: Molecular factors that determine cell fate after division
- **Polarity**: Cellular polarity mechanisms that guide asymmetric division

#### Architecture

```
asymmetric_division_module/
├── DivisionType (enum)
├── AsymmetricDivisionComponent (struct)
├── AsymmetricDivisionParams (struct)
└── AsymmetricDivisionModule (struct)
```

#### Usage Example

```rust
use asymmetric_division_module::{AsymmetricDivisionModule, AsymmetricDivisionParams};

// Configure the module
let params = AsymmetricDivisionParams {
    asymmetric_division_probability: 0.4,
    symmetric_renewal_probability: 0.4,
    symmetric_diff_probability: 0.2,
    stem_cell_niche_capacity: 5,
    max_niches: 10,
};

// Create module
let mut module = AsymmetricDivisionModule::with_params(params);

// Create stem cell niches
let niche_id = module.create_niche(0.0, 0.0, 0.0, 5.0);
```

### 12. **Stem Cell Hierarchy Module**

The stem cell hierarchy module models different levels of stem cell potency and differentiation pathways.

#### Key Features

- **Potency Levels**: Totipotent → Pluripotent → Multipotent → Oligopotent → Unipotent → Differentiated
- **Cell Lineages**: Embryonic, hematopoietic, neural, and other lineages
- **Master Regulators**: OCT4, NANOG, SOX2 and other transcription factors
- **Differentiation Pathways**: Step-by-step differentiation along specific lineages

#### Potency Levels

| Level | Description | Examples |
|-------|-------------|----------|
| **Totipotent** | Can form all cell types + extraembryonic tissues | Zygote, early blastomeres |
| **Pluripotent** | Can form all cell types of the organism | Embryonic stem cells |
| **Multipotent** | Limited to specific lineages | Hematopoietic stem cells |
| **Oligopotent** | Can form a few cell types | Myeloid progenitor |
| **Unipotent** | Can form one cell type | Spermatogonial stem cells |
| **Differentiated** | Terminally differentiated | Neuron, muscle cell |

#### Cell Lineages

- **EmbryonicStem**: Embryonic stem cells
- **HematopoieticStem**: Hematopoietic (blood) stem cells
- **NeuralStem**: Neural stem cells
- More lineages can be added as needed

#### Usage Example

```rust
use stem_cell_hierarchy_module::{
    StemCellHierarchyModule, StemCellHierarchyParams,
    PotencyLevel, factories
};

// Create different stem cell types
let embryonic_sc = factories::create_embryonic_stem_cell();
let hematopoietic_sc = factories::create_hematopoietic_stem_cell();
let neural_sc = factories::create_neural_stem_cell();

// Configure module
let params = StemCellHierarchyParams {
    initial_potency: PotencyLevel::Pluripotent,
    enable_plasticity: true,
    plasticity_rate: 0.01,
    differentiation_threshold: 0.7,
};

let module = StemCellHierarchyModule::with_params(params);
```

## 🚀 Combined Example

Here's a complete example showing both modules working together:

```rust
use cell_dt_core::{
    SimulationManager, SimulationConfig,
    components::{CentriolePair, CellCycleStateExtended},
};
use centriole_module::CentrioleModule;
use cell_cycle_module::{CellCycleModule, CellCycleParams};
use asymmetric_division_module::{AsymmetricDivisionModule, AsymmetricDivisionParams};
use stem_cell_hierarchy_module::{
    StemCellHierarchyModule, StemCellHierarchyParams, 
    PotencyLevel, factories
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize simulation
    let config = SimulationConfig::default();
    let mut sim = SimulationManager::new(config);
    
    // Register modules
    sim.register_module(Box::new(CentrioleModule::with_parallel(true)))?;
    sim.register_module(Box::new(CellCycleModule::new()))?;
    
    // Asymmetric division module
    let mut asym_module = AsymmetricDivisionModule::with_params(
        AsymmetricDivisionParams::default()
    );
    asym_module.create_niche(0.0, 0.0, 0.0, 5.0);
    sim.register_module(Box::new(asym_module))?;
    
    // Stem cell hierarchy module
    sim.register_module(Box::new(
        StemCellHierarchyModule::with_params(StemCellHierarchyParams::default())
    ))?;
    
    // Create stem cells
    let world = sim.world_mut();
    for i in 0..10 {
        let hierarchy = if i < 3 {
            factories::create_embryonic_stem_cell()
        } else if i < 6 {
            factories::create_hematopoietic_stem_cell()
        } else {
            factories::create_neural_stem_cell()
        };
        
        world.spawn((
            CentriolePair::default(),
            CellCycleStateExtended::new(),
            hierarchy,
        ));
    }
    
    // Run simulation
    sim.run()?;
    
    Ok(())
}
```

## 📊 Data Structures

### AsymmetricDivisionComponent

```rust
pub struct AsymmetricDivisionComponent {
    pub division_type: DivisionType,  // Type of division
    pub niche_id: Option<u64>,         // ID of stem cell niche
    pub stemness_potential: f32,       // 0-1, stem cell potential
}
```

### StemCellHierarchyState

```rust
pub struct StemCellHierarchyState {
    pub potency_level: PotencyLevel,           // Current potency level
    pub potency_score: f32,                     // 0-1 potency score
    pub lineage: Option<CellLineage>,           // Current lineage
    pub master_regulator_levels: HashMap<String, f32>, // OCT4, NANOG, SOX2 levels
}
```

## 🎯 Configuration Parameters

### AsymmetricDivisionParams

| Parameter | Default | Description |
|-----------|---------|-------------|
| `asymmetric_division_probability` | 0.3 | Probability of asymmetric division |
| `symmetric_renewal_probability` | 0.4 | Probability of symmetric self-renewal |
| `symmetric_diff_probability` | 0.3 | Probability of symmetric differentiation |
| `stem_cell_niche_capacity` | 10 | Maximum cells per niche |
| `max_niches` | 100 | Maximum number of niches |

### StemCellHierarchyParams

| Parameter | Default | Description |
|-----------|---------|-------------|
| `initial_potency` | Pluripotent | Starting potency level |
| `enable_plasticity` | true | Allow dedifferentiation |
| `plasticity_rate` | 0.01 | Rate of plasticity changes |
| `differentiation_threshold` | 0.7 | Threshold for terminal differentiation |

## 🔧 Factory Functions

The module provides factory functions to create common stem cell types:

```rust
// Create embryonic stem cell (pluripotent)
let esc = factories::create_embryonic_stem_cell();

// Create hematopoietic stem cell (multipotent)
let hsc = factories::create_hematopoietic_stem_cell();

// Create neural stem cell (multipotent)
let nsc = factories::create_neural_stem_cell();
```

## 📈 Future Extensions

Planned features for future releases:

1. **More Lineages**: Endoderm, mesoderm, ectoderm specific lineages
2. **Inducer Molecules**: Chemical inducers for directed differentiation
3. **Epigenetic Memory**: Maintenance of cell identity through divisions
4. **Reprogramming**: Induced pluripotency (iPSC) mechanisms
5. **Cancer Stem Cells**: Modeling of cancer stem cell behavior
6. **Niche Signaling**: Complex signaling between niche and stem cells

## 🧪 Testing

Run the stem cell example to see both modules in action:

```bash
cargo run --bin stem_cell_example
```

## 📚 References

1. Morrison, S. J., & Spradling, A. C. (2008). Stem cells and niches: mechanisms that promote stem cell maintenance throughout life.
2. Knoblich, J. A. (2008). Mechanisms of asymmetric stem cell division.
3. Waddington, C. H. (1957). The strategy of the genes.
4. Takahashi, K., & Yamanaka, S. (2006). Induction of pluripotent stem cells from mouse embryonic and adult fibroblast cultures.

## 🤝 Contributing

We welcome contributions to extend the stem cell biology modules! Potential areas:

- Adding new lineages
- Implementing differentiation inducer molecules
- Creating more sophisticated niche models
- Adding epigenetic regulation
- Modeling disease states (cancer, degeneration)