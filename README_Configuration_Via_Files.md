```markdown
# Cell DT - Configuration Files ⚙️

## 📁 Location

Configuration files are stored in the **`configs/`** directory in the project root:

```
/home/oem/Documents/Projects/rust/cell_dt/configs/
```

## 📋 Available Configuration Files

After running the configuration setup commands, you will have the following files:

| File | Description |
|------|-------------|
| `example.toml` | Example configuration in TOML format |
| `example.yaml` | Example configuration in YAML format |
| `default.toml` | Default configuration (created when running the example) |
| `default.yaml` | Default configuration in YAML format |
| `custom.toml` | Custom user configuration |
| `development.toml` | Configuration for development and debugging |
| `production.toml` | Configuration for production runs |
| `benchmark.toml` | Configuration for performance testing |
| `README.md` | Configuration documentation |

## 🔍 Viewing Configuration Files

```bash
# View a specific configuration file
cat /home/oem/Documents/Projects/rust/cell_dt/configs/example.toml

# Or using the management script
cd /home/oem/Documents/Projects/rust/cell_dt
./manage_configs.sh show configs/example.toml

# List all available configurations
./manage_configs.sh list
```

## 🚀 Using Configuration Files

### Basic Usage

```bash
# Activate virtual environment
cd /home/oem/Documents/Projects/rust/cell_dt
source venv/bin/activate

# Run simulation with a specific configuration
cargo run --bin run_simulation -- --config configs/production.toml --cells 10000

# Run with configuration script
./run_with_config.sh configs/development.toml
```

### Configuration Management Script

```bash
# Show all available configurations
./manage_configs.sh list

# View configuration contents
./manage_configs.sh show configs/example.toml

# Create a new configuration from template
./manage_configs.sh create

# Validate a configuration file
./manage_configs.sh validate configs/example.toml

# Generate default configuration
./manage_configs.sh default
```

## 📝 Configuration File Structure

### TOML Format Example (`example.toml`)

```toml
[simulation]                 # Main simulation parameters
max_steps = 5000             # Number of simulation steps
dt = 0.1                     # Time step
num_threads = 4              # Number of threads
seed = 42                    # Random seed
output_dir = "results/test_run"  # Output directory

[centriole_module]           # Centriole module parameters
enabled = true               # Enable centriole module
acetylation_rate = 0.02      # Acetylation rate
oxidation_rate = 0.01        # Oxidation rate
parallel_cells = true        # Parallel cell processing

[cell_cycle_module]          # Cell cycle module parameters
enabled = true               # Enable cell cycle module
base_cycle_time = 24.0       # Base cycle duration
checkpoint_strictness = 0.15 # Checkpoint strictness
enable_apoptosis = true      # Enable apoptosis
nutrient_availability = 0.9  # Nutrient availability
growth_factor_level = 0.85   # Growth factor level
random_variation = 0.25      # Random variation

[transcriptome_module]       # Transcriptome module parameters
enabled = true               # Enable transcriptome module
mutation_rate = 0.001        # Mutation rate
noise_level = 0.05           # Expression noise level

[io_module]                  # I/O module parameters
enabled = true               # Enable I/O module
output_format = "csv"        # Output format (csv, parquet)
compression = "none"         # Compression type
buffer_size = 1000           # Buffer size
```

### YAML Format Example (`example.yaml`)

```yaml
simulation:
  max_steps: 10000
  dt: 0.05
  num_threads: 8
  seed: 123
  output_dir: "results/experiment_1"

centriole_module:
  enabled: true
  acetylation_rate: 0.03
  oxidation_rate: 0.015
  parallel_cells: true

cell_cycle_module:
  enabled: true
  base_cycle_time: 20.0
  checkpoint_strictness: 0.1
  enable_apoptosis: true
  nutrient_availability: 0.95
  growth_factor_level: 0.9
  random_variation: 0.2

transcriptome_module:
  enabled: true
  mutation_rate: 0.002
  noise_level: 0.03

io_module:
  enabled: true
  output_format: "parquet"
  compression: "snappy"
  buffer_size: 5000
```

## 🎯 Pre-configured Scenarios

### Development Configuration (`development.toml`)

```toml
[simulation]
max_steps = 100
dt = 0.1
num_threads = 2
seed = 42
output_dir = "results/dev"

[cell_cycle_module]
enable_apoptosis = false
nutrient_availability = 1.0
growth_factor_level = 1.0
random_variation = 0.3

[transcriptome_module]
mutation_rate = 0.01
noise_level = 0.1

[io_module]
buffer_size = 100
```

**Use for:** Testing, debugging, quick iterations

### Production Configuration (`production.toml`)

```toml
[simulation]
max_steps = 100000
dt = 0.05
checkpoint_interval = 1000
num_threads = 32
parallel_modules = true
output_dir = "/scratch/results/prod"

[cell_cycle_module]
base_cycle_time = 24.0
checkpoint_strictness = 0.2

[transcriptome_module]
mutation_rate = 0.001
noise_level = 0.05

[io_module]
output_format = "parquet"
compression = "snappy"
buffer_size = 10000
```

**Use for:** Large-scale production runs

### Benchmark Configuration (`benchmark.toml`)

```toml
[simulation]
max_steps = 1000
dt = 0.1
num_threads = 8
parallel_modules = true
output_dir = "results/benchmark"

[centriole_module]
parallel_cells = true

[transcriptome_module]
enabled = false

[io_module]
enabled = false
```

**Use for:** Performance testing and benchmarking

## 🛠️ Creating Custom Configurations

### Method 1: Using the Management Script

```bash
# Create from template
cd /home/oem/Documents/Projects/rust/cell_dt
./manage_configs.sh create
# Creates: configs/new_config_YYYYMMDD_HHMMSS.toml
```

### Method 2: Copy and Edit

```bash
# Copy existing configuration
cp configs/example.toml configs/my_experiment.toml

# Edit with your preferred editor
nano configs/my_experiment.toml
# or
vim configs/my_experiment.toml
# or
code configs/my_experiment.toml
```

### Method 3: Generate from Code

```bash
# Generate default configuration
cargo run --bin config_example
# Creates: configs/default.toml and configs/default.yaml
```

## ✅ Configuration Validation

Always validate your configuration files before running long simulations:

```bash
# Validate configuration
./manage_configs.sh validate configs/my_experiment.toml

# Expected output:
# ✅ Configuration is valid!
#   Simulation steps: 5000
#   Modules enabled:
#     - Centriole
#     - Cell Cycle
#     - Transcriptome
```

## 📊 Configuration Best Practices

1. **Version Control**: Always commit configuration files with your code
2. **Documentation**: Add comments to explain non-obvious parameters
3. **Validation**: Always validate configs before running long simulations
4. **Environment-specific**: Use different configs for dev/prod/benchmark
5. **Experiments**: Create separate config files for different experiments
6. **Backup**: Keep backups of important experiment configurations

## 🔄 Configuration Hierarchy

```
Command Line Arguments (highest priority)
    ↓
Configuration File
    ↓
Default Values (lowest priority)
```

Example:
```bash
# File values will be overridden by command line arguments
cargo run --bin run_simulation \
  --config configs/production.toml \
  --steps 50000 \
  --threads 64 \
  --output /scratch/results/special_run \
  run --cells 2000000
```

## 🚦 Quick Start Commands

```bash
# Navigate to project
cd /home/oem/Documents/Projects/rust/cell_dt

# Activate virtual environment
source venv/bin/activate

# List available configurations
./manage_configs.sh list

# View example configuration
./manage_configs.sh show configs/example.toml

# Create your own configuration
cp configs/example.toml configs/my_experiment.toml
nano configs/my_experiment.toml

# Validate it
./manage_configs.sh validate configs/my_experiment.toml

# Run simulation with your config
./run_with_config.sh configs/my_experiment.toml
```

## 📚 Additional Resources

- [TOML Documentation](https://toml.io/)
- [YAML Documentation](https://yaml.org/)
- [Cell DT Main README](../../README.md)
- [Python Bindings Documentation](../cell_dt_python/README.md)

## 🆘 Troubleshooting

### Common Issues

**Issue:** Configuration file not found
```bash
# Check if file exists
ls -la configs/

# Use absolute path
./run_with_config.sh /home/oem/Documents/Projects/rust/cell_dt/configs/my_config.toml
```

**Issue:** Invalid configuration format
```bash
# Validate the file
./manage_configs.sh validate configs/my_config.toml

# Check for syntax errors
cat configs/my_config.toml
```

**Issue:** Missing required fields
```bash
# Generate default config and compare
./manage_configs.sh default
diff configs/default.toml configs/my_config.toml
```