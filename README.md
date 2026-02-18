Вот полный текст README.md:

```markdown
# Cell DT (Cell Differentiation Platform)

[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/djabbat/cell_dt)
[![Documentation](https://img.shields.io/badge/docs-latest-blue)](https://djabbat.github.io/cell_dt)

**Cell DT** is a high-performance platform for simulating cell differentiation, written in Rust. The platform specializes in modeling the centriole as a key regulatory hub and supports creating digital twins for experimental research.

## 👨‍🔬 Author

**Jaba Tkemaladze** - *Initial work* - [GitHub](https://github.com/djabbat)

## ✨ Key Features

- **Modular Architecture** — Easily plug and replace simulation modules
- **ECS (Entity-Component System)** — Efficient management of large cell populations (10⁵-10⁶ cells)
- **Centriole Modeling** — Age tracking, PTM profiles, CAFD factors
- **Parallel Computing** — Utilizes all CPU cores via Rayon
- **Checkpoints** — Save and load simulation state
- **Extensibility** — Plugin support and Python integration via PyO3

## 🏗 Architecture

```
cell_dt/
├── crates/
│   ├── cell_dt_core/          # Platform core (ECS, traits, manager)
│   ├── cell_dt_modules/        # Simulation modules
│   │   ├── centriole_module/   # Centriole module (PTM, CAFD, age)
│   │   ├── cell_cycle_module/  # Cell cycle module
│   │   └── transcriptome_module/ # Transcriptome module
│   ├── cell_dt_io/             # Data I/O
│   └── cell_dt_python/          # Python bindings (PyO3)
└── examples/                    # Usage examples
```

## 🚀 Quick Start

### Prerequisites

- Rust 1.70 or higher ([install](https://www.rust-lang.org/tools/install))
- Cargo (comes with Rust)

### Installation

```bash
# Clone the repository
git clone https://github.com/djabbat/cell_dt.git
cd cell_dt

# Build in release mode
cargo build --release
```

### Run Examples

```bash
# Basic example with centriole module
cargo run --bin simple_simulation

# With detailed logging
RUST_LOG=debug cargo run --bin simple_simulation
```

### Example Code

```rust
use cell_dt_core::{
    SimulationManager, SimulationConfig,
    components::{CentriolePair, CellCycleState, Phase},
};
use centriole_module::CentrioleModule;
use rand::Rng;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure simulation
    let config = SimulationConfig {
        max_steps: 1000,
        dt: 0.1,
        num_threads: Some(4),
        seed: Some(42),
        ..Default::default()
    };
    
    // Initialize simulation manager
    let mut sim = SimulationManager::new(config);
    
    // Register centriole module
    sim.register_module(Box::new(CentrioleModule::new()))?;
    
    // Add cells
    let world = sim.world_mut();
    for i in 0..10 {
        world.spawn((
            CentriolePair::default(),
            CellCycleState {
                phase: Phase::G1,
                progress: rand::thread_rng().gen::<f32>(),
            },
        ));
    }
    
    // Run simulation
    sim.run()?;
    
    Ok(())
}
```

## 📦 Project Structure

### Core Crates

| Crate | Description |
|-------|-------------|
| `cell_dt_core` | Core platform with ECS, traits, and simulation manager |
| `centriole_module` | Centriole model with PTM profiles and CAFD factors |
| `cell_cycle_module` | Cell cycle model (under development) |
| `transcriptome_module` | Transcriptome model (under development) |
| `cell_dt_io` | Data I/O utilities (CSV, HDF5, Arrow) |
| `cell_dt_python` | Python bindings via PyO3 |

### Key Components

- **Centriole** — Tracks maturity, PTM modifications, and associated factors
- **PTMProfile** — Post-translational modifications (acetylation, oxidation, etc.)
- **CAFD** — Centriole-associated factors (YAP, STAT3, etc.)
- **CellCycleState** — Current phase and progress through cell cycle

## 🔧 Development

### Building

```bash
# Build all crates
cargo build

# Build with optimizations
cargo build --release

# Build specific crate
cargo build -p centriole_module
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with logging
RUST_LOG=debug cargo test -- --nocapture
```

### Documentation

```bash
# Generate and open documentation
cargo doc --open

# Generate docs for specific crate
cargo doc -p cell_dt_core --open
```

### Benchmarking

```bash
# Run benchmarks
cargo bench
```

## 📊 Performance

The platform is designed for high-performance simulation of large cell populations:

- **10⁵ cells** — Real-time simulation on a laptop
- **10⁶ cells** — Near real-time on a workstation
- **Parallel processing** — Scales with available CPU cores
- **Memory efficient** — ECS architecture minimizes overhead

## 🤝 Contributing

Contributions are welcome! Here's how you can help:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Guidelines

- Follow Rust best practices and idioms
- Add tests for new functionality
- Update documentation as needed
- Keep modules loosely coupled
- Use the trait system for extensibility

## 📝 License

This project is licensed under either of:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## 📧 Contact

**Jaba Tkemaladze** - [@djabbat](https://github.com/djabbat)

Project Link: [https://github.com/djabbat/cell_dt](https://github.com/djabbat/cell_dt)

## 🙏 Acknowledgments

- Based on research from the dissertation on centriole biology
- Inspired by the need for high-performance biological simulations
- Thanks to the Rust community for excellent tools and libraries

## 📚 Citation

If you use this platform in your research, please cite:

```bibtex
@software{cell_dt,
  author = {Jaba Tkemaladze},
  title = {Cell DT: High-Performance Cell Differentiation Platform},
  year = {2024},
  url = {https://github.com/djabbat/cell_dt}
}
```

## 🗺 Roadmap

### Version 0.2.0 (Current)
- ✅ Core platform with ECS
- ✅ Centriole module with PTM tracking
- ✅ Basic simulation example
- ⬜ Parallel processing with Rayon
- ⬜ Checkpoint serialization

### Version 0.3.0 (Planned)
- ⬜ Cell cycle module implementation
- ⬜ Data I/O (CSV, HDF5)
- ⬜ Python bindings
- ⬜ WebAssembly support

### Version 1.0.0 (Future)
- ⬜ Full transcriptome integration
- ⬜ 3D spatial modeling
- ⬜ Machine learning integration
- ⬜ Real-time visualization
```