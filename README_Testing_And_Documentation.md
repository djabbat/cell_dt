```markdown
# Cell DT - Testing & Documentation 🧪📚

## 📋 Overview

This document covers the testing strategy, documentation generation, and quality assurance tools for the Cell DT platform.

## 🧪 Testing

### Test Structure

```
cell_dt/
├── crates/
│   ├── cell_dt_core/
│   │   ├── src/
│   │   │   └── lib.rs           # Unit tests in each module
│   │   └── tests/
│   │       └── integration_test.rs # Integration tests
│   ├── centriole_module/
│   │   └── src/
│   │       └── lib.rs           # Module-specific tests
│   ├── cell_cycle_module/
│   │   └── src/
│   │       └── lib.rs           # Module-specific tests
│   └── ...
└── tests/                         # Global integration tests
```

### Test Types

#### 1. **Unit Tests**
Located in each module's `lib.rs` file, testing individual components:

```bash
# Run all unit tests
cargo test --lib

# Run unit tests for specific module
cargo test -p cell_dt_core --lib
cargo test -p centriole_module --lib
```

#### 2. **Integration Tests**
Testing interactions between modules:

```bash
# Run all integration tests
cargo test --test '*'

# Run specific integration test
cargo test --test integration_test
```

#### 3. **Documentation Tests**
Testing code examples in documentation:

```bash
# Run doc tests
cargo test --doc
```

### Running Tests

#### Quick Test Run
```bash
# Run all tests (unit + integration + doc)
cargo test

# Run with verbose output
cargo test --verbose

# Run with specific filter
cargo test test_centriole_creation
```

#### Comprehensive Test Suite
```bash
# Run the complete test script
./run_tests.sh
```

This script runs:
- ✅ Code formatting check (`cargo fmt -- --check`)
- ✅ Static analysis (`cargo clippy`)
- ✅ Unit tests (`cargo test --lib`)
- ✅ Integration tests (`cargo test --test '*'`)
- ✅ Documentation tests (`cargo test --doc`)
- ✅ Example compilation (`cargo build --examples`)

### Code Coverage

Generate coverage reports using `cargo-tarpaulin`:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML coverage report
cargo tarpaulin --out Html

# Generate XML report for CI
cargo tarpaulin --out Xml

# Run with specific filters
cargo tarpaulin --ignore-tests --exclude-files "examples/*"
```

### Performance Testing

```bash
# Run benchmarks (if implemented)
cargo bench

# Run with flamegraph profiling
cargo flamegraph --bin simple_simulation -- --cells 10000

# Check performance regressions
cargo bench -- --baseline master
```

## 📊 Test Examples

### Component Tests
```rust
#[test]
fn test_centriole_creation() {
    let mother = Centriole::new_mature();
    let daughter = Centriole::new_daughter();
    
    assert_eq!(mother.maturity, 1.0);
    assert_eq!(daughter.maturity, 0.0);
}

#[test]
fn test_phase_enum() {
    let phases = vec![Phase::G1, Phase::S, Phase::G2, Phase::M];
    assert_eq!(phases.len(), 4);
}
```

### Module Tests
```rust
#[test]
fn test_centriole_module_step() {
    let mut module = CentrioleModule::new();
    let mut world = World::new();
    
    // Add test cell
    world.spawn((CentriolePair::default(),));
    
    // Run one step
    module.step(&mut world, 0.1).unwrap();
    
    // Verify changes
    let query = world.query::<&CentriolePair>().iter().next().unwrap();
    assert!(query.1.mother.maturity > 0.0);
}
```

### Integration Tests
```rust
#[test]
fn test_full_simulation() {
    let config = SimulationConfig::default();
    let mut sim = SimulationManager::new(config);
    
    // Add modules and cells
    sim.register_module(Box::new(CentrioleModule::new()));
    sim.world_mut().spawn((CentriolePair::default(),));
    
    // Run simulation
    sim.run().unwrap();
    
    assert_eq!(sim.current_step(), config.max_steps);
}
```

## 📚 Documentation

### API Documentation

Generate and view API documentation:

```bash
# Generate documentation for all crates
cargo doc --no-deps --workspace --document-private-items

# Open in browser
cargo doc --open

# Generate only public API
cargo doc --no-deps

# Generate with examples
cargo doc --examples
```

### Documentation Structure

```
target/doc/
├── cell_dt/                      # Main crate docs
├── cell_dt_core/                 # Core module docs
├── centriole_module/              # Centriole module docs
├── cell_cycle_module/             # Cell cycle module docs
├── transcriptome_module/          # Transcriptome module docs
├── cell_dt_io/                    # I/O module docs
├── cell_dt_python/                # Python bindings docs
├── cell_dt_viz/                   # Visualization docs
└── cell_dt_config/                # Configuration docs
```

### Documentation Guidelines

#### Code Documentation
```rust
/// # Centriole Module
/// 
/// This module simulates centriole behavior including:
/// * Maturation over time
/// * PTM (Post-Translational Modifications)
/// * CAFD (Centriole-Associated Factors)
/// * MTOC activity
/// 
/// ## Example
/// ```
/// use centriole_module::CentrioleModule;
/// 
/// let module = CentrioleModule::new();
/// assert_eq!(module.name(), "centriole_module");
/// ```
#[derive(Debug)]
pub struct CentrioleModule {
    // ...
}

/// Updates centriole maturity and PTM levels
/// 
/// # Arguments
/// * `centriole` - Mutable reference to the centriole
/// * `dt` - Time step for the update
/// 
/// # Returns
/// * `()` - Updates are applied in-place
fn update_centriole(&self, centriole: &mut Centriole, dt: f32) {
    // ...
}
```

### Generating Documentation

```bash
# Quick documentation generation
./docs/generate_docs.sh
```

## 🔧 Quality Assurance Tools

### Static Analysis

```bash
# Check code formatting
cargo fmt -- --check

# Auto-fix formatting
cargo fmt

# Linting with clippy
cargo clippy
cargo clippy -- -W clippy::pedantic
cargo clippy --fix  # Auto-fix some issues
```

### Dependency Checks

```bash
# Check for outdated dependencies
cargo outdated

# Security vulnerabilities check
cargo audit

# Dependency tree visualization
cargo tree
```

## 🤖 Continuous Integration (CI)

The project includes GitHub Actions workflow (`.github/workflows/ci.yml`) that automatically runs on every push and pull request:

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run tests
        run: cargo test
      - name: Check formatting
        run: cargo fmt -- --check
      - name: Clippy
        run: cargo clippy -- -D warnings
```

## 📊 Test Coverage Goals

| Component | Target Coverage | Current |
|-----------|----------------|---------|
| Core | 90% | 85% |
| Centriole Module | 85% | 80% |
| Cell Cycle Module | 85% | 75% |
| Transcriptome Module | 80% | 70% |
| I/O Module | 80% | 75% |
| Python Bindings | 70% | 60% |
| **Overall** | **85%** | **78%** |

## 🚀 Quick Start Commands

```bash
# Run all tests
./run_tests.sh

# Generate and open documentation
cargo doc --open

# Check code coverage
cargo tarpaulin --out Html

# Run specific test
cargo test test_centriole_creation

# Check formatting
cargo fmt -- --check

# Lint code
cargo clippy

# Security audit
cargo audit
```

## 📈 Test Reports

### HTML Coverage Report
```bash
# Generate coverage report
cargo tarpaulin --out Html
# Open tarpaulin-report.html in browser
```

### JUnit XML for CI
```bash
# Generate JUnit report
cargo test -- -Z unstable-options --format json --report-time | \
  cargo2junit > results.xml
```

## 🎯 Best Practices

1. **Write tests first** (TDD approach)
2. **Keep tests independent** - no shared state
3. **Test edge cases** - empty populations, extreme values
4. **Use property-based testing** for complex logic
5. **Maintain documentation** alongside code
6. **Run tests before committing** (`cargo test`)
7. **Check coverage regularly** (`cargo tarpaulin`)

## 📚 Additional Resources

- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [cargo-tarpaulin Documentation](https://crates.io/crates/cargo-tarpaulin)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)

## 🆘 Troubleshooting

### Common Issues

**Issue:** Tests timeout
```bash
# Increase timeout
cargo test -- --test-threads=1 --nocapture
```

**Issue:** Coverage report not generating
```bash
# Install tarpaulin with proper permissions
cargo install cargo-tarpaulin --force
```

**Issue:** Documentation build failing
```bash
# Check for broken links
cargo doc --no-deps --document-private-items --workspace
```