```markdown
# Cell DT Python Bindings

Python interface for the Cell Differentiation Simulation Platform.

## 📋 Overview

The Cell DT Python bindings allow you to:
- Run simulations directly from Python and Jupyter notebooks
- Analyze results with NumPy, pandas, and matplotlib
- Integrate with machine learning libraries (scikit-learn, PyTorch, TensorFlow)
- Visualize cell populations and their dynamics
- Export data for further analysis

## ⚠️ Important: Virtual Environment Activation

**Every time you work with Python bindings, you must activate the virtual environment:**

```bash
cd /home/oem/Documents/Projects/rust/cell_dt
source venv/bin/activate
```

**Then you can work with the bindings:**

```bash
cd crates/cell_dt_python
python example.py
# or
jupyter notebook cell_dt_demo.ipynb
```

## 🔧 Installation

### Prerequisites

- Python 3.7 or higher
- Rust (install via [rustup](https://rustup.rs/))
- Git

### Step-by-step Installation

```bash
# 1. Clone the repository
git clone https://github.com/djabbat/cell_dt.git
cd cell_dt

# 2. Create and activate a virtual environment
python3 -m venv venv
source venv/bin/activate  # On Linux/Mac
# or
venv\Scripts\activate     # On Windows

# 3. Install dependencies
pip install --upgrade pip
pip install maturin numpy pandas matplotlib jupyter ipywidgets plotly scikit-learn

# 4. Install Python bindings
cd crates/cell_dt_python
maturin develop --release

# 5. Verify installation
python -c "import cell_dt; print('✅ Cell DT successfully installed!')"
```

## 🚀 Quick Start Guide

### Basic Simulation

```python
import cell_dt

# Create a simulation with 500 steps
sim = cell_dt.PySimulation(
    max_steps=500,
    dt=0.1,
    num_threads=4,
    seed=42
)

# Create 100 cells
sim.create_population(100)

# Register modules (centriole and cell cycle only)
sim.register_modules(
    enable_centriole=True,
    enable_cell_cycle=True,
    enable_transcriptome=False,
    cell_cycle_params=None  # Use default parameters
)

# Run the simulation
cells = sim.run()

# Display results
print(f"Simulation completed at step {sim.current_step()}")
print(f"Total cells: {len(cells)}")

# Analyze cell phases
from collections import Counter
phases = [cell.cell_cycle.phase for cell in cells]
phase_counts = Counter(phases)
print("\nPhase distribution:")
for phase, count in phase_counts.items():
    print(f"  {phase}: {count}")
```

### Advanced Simulation with Transcriptome

```python
import cell_dt
import numpy as np
import matplotlib.pyplot as plt

# Custom cell cycle parameters
params = cell_dt.PyCellCycleParams(
    base_cycle_time=20.0,
    growth_factor_sensitivity=0.3,
    stress_sensitivity=0.2,
    checkpoint_strictness=0.15,
    enable_apoptosis=True,
    nutrient_availability=0.9,
    growth_factor_level=0.85,
    random_variation=0.25
)

# Create simulation with transcriptome
sim = cell_dt.PySimulation(max_steps=1000, dt=0.05, num_threads=4, seed=42)
sim.create_population_with_transcriptome(50)

sim.register_modules(
    enable_centriole=True,
    enable_cell_cycle=True,
    enable_transcriptome=True,
    cell_cycle_params=params
)

# Run step by step and collect data
phase_history = []
for step in range(0, 1000, 100):
    cells = sim.step(100)
    
    # Analyze phases
    phases = [cell.cell_cycle.phase for cell in cells]
    phase_counts = dict(zip(*np.unique(phases, return_counts=True)))
    phase_history.append(phase_counts)
    
    print(f"Step {sim.current_step()}: {len(cells)} cells")

# Get NumPy data for analysis
centriole_data = sim.get_centriole_data_numpy()
print(f"\nCentriole data shape: {centriole_data.shape}")
print(f"Average mother maturity: {np.mean(centriole_data[:, 0]):.3f}")
```

## 📊 Working with NumPy

The bindings provide direct NumPy integration for efficient data analysis:

```python
import cell_dt
import numpy as np
import matplotlib.pyplot as plt

sim = cell_dt.PySimulation(max_steps=200, dt=0.1, num_threads=4, seed=42)
sim.create_population(1000)
sim.register_modules(True, True, False, None)
cells = sim.run()

# Get data as NumPy arrays
centriole_data = sim.get_centriole_data_numpy()  # Shape: (n_cells, 5)

# Analyze
plt.figure(figsize=(12, 4))

plt.subplot(1, 3, 1)
plt.hist(centriole_data[:, 0], bins=30, alpha=0.7, label='Mother')
plt.hist(centriole_data[:, 1], bins=30, alpha=0.7, label='Daughter')
plt.xlabel('Maturity')
plt.ylabel('Count')
plt.legend()
plt.title('Centriole Maturity')

plt.subplot(1, 3, 2)
plt.hist(centriole_data[:, 2], bins=30, color='green', alpha=0.7)
plt.xlabel('MTOC Activity')
plt.ylabel('Count')
plt.title('MTOC Activity')

plt.subplot(1, 3, 3)
phase_dist = sim.get_phase_distribution()
phases = list(phase_dist.keys())
counts = list(phase_dist.values())
plt.bar(phases, counts, color=['blue', 'green', 'orange', 'red'])
plt.xlabel('Phase')
plt.ylabel('Count')
plt.title('Phase Distribution')

plt.tight_layout()
plt.show()
```

## 🤖 Machine Learning Integration

Use simulation data to train ML models:

```python
import cell_dt
import numpy as np
from sklearn.ensemble import RandomForestClassifier
from sklearn.model_selection import train_test_split
from sklearn.metrics import classification_report

# Run simulation
sim = cell_dt.PySimulation(max_steps=500, dt=0.1, num_threads=4, seed=42)
sim.create_population(500)
sim.register_modules(True, True, False, None)
cells = sim.run()

# Prepare features and labels
X = []
y = []
for cell in cells:
    features = [
        cell.centriole.mother_maturity,
        cell.centriole.daughter_maturity,
        cell.centriole.mtoc_activity,
        cell.cell_cycle.progress,
        cell.cell_cycle.growth_signal,
        cell.cell_cycle.stress_level
    ]
    X.append(features)
    y.append(cell.cell_cycle.phase)

# Train classifier
X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.3)
clf = RandomForestClassifier(n_estimators=100)
clf.fit(X_train, y_train)

# Evaluate
y_pred = clf.predict(X_test)
print(classification_report(y_test, y_pred))

# Feature importance
features = ['mother_maturity', 'daughter_maturity', 'mtoc_activity', 
            'progress', 'growth_signal', 'stress_level']
importance = dict(zip(features, clf.feature_importances_))
print("\nFeature importance:")
for feat, imp in sorted(importance.items(), key=lambda x: x[1], reverse=True):
    print(f"  {feat}: {imp:.3f}")
```

## 📓 Jupyter Notebook Integration

The package includes a comprehensive Jupyter notebook demo:

```bash
# From the bindings directory
jupyter notebook cell_dt_demo.ipynb
```

The notebook demonstrates:
- Interactive parameter tuning
- Real-time visualization
- Batch simulations
- Data export to pandas DataFrames
- Statistical analysis

## 🧪 Available Python Classes

### `PySimulation`
Main simulation controller
- `__init__(max_steps, dt, num_threads, seed)` - Create simulation
- `create_population(count)` - Create cells without transcriptome
- `create_population_with_transcriptome(count)` - Create cells with transcriptome
- `register_modules(enable_centriole, enable_cell_cycle, enable_transcriptome, cell_cycle_params)` - Register modules
- `run()` - Run complete simulation
- `step(steps)` - Run specified number of steps
- `get_cell_data()` - Get all cell data
- `get_centriole_data_numpy()` - Get centriole data as NumPy array
- `get_phase_distribution()` - Get phase distribution as dictionary
- `current_step()` - Get current simulation step
- `current_time()` - Get current simulation time
- `cell_count()` - Get number of cells

### `PyCellCycleParams`
Parameters for cell cycle module
- `base_cycle_time` - Base cycle duration
- `growth_factor_sensitivity` - Sensitivity to growth factors
- `stress_sensitivity` - Sensitivity to stress
- `checkpoint_strictness` - How strict checkpoints are
- `enable_apoptosis` - Enable/disable apoptosis
- `nutrient_availability` - Nutrient level (0-1)
- `growth_factor_level` - Growth factor level (0-1)
- `random_variation` - Random variation in cycle duration

### Data Classes
- `PyCellData` - Complete cell data
- `PyCentrioleData` - Centriole-specific data
- `PyCellCycleData` - Cell cycle-specific data
- `PyTranscriptomeData` - Transcriptome-specific data

## 📊 Performance Considerations

| Number of Cells | Memory Usage | Simulation Time (1000 steps) |
|----------------|--------------|------------------------------|
| 1,000 | ~50 MB | 2-3 seconds |
| 10,000 | ~500 MB | 20-30 seconds |
| 100,000 | ~5 GB | 3-4 minutes |
| 1,000,000 | ~50 GB | 30-40 minutes |

**Tips for large simulations:**
- Use `step()` instead of `run()` to process data incrementally
- Export data periodically rather than keeping all in memory
- Increase `num_threads` to match your CPU cores
- Consider using a machine with sufficient RAM

## 🔍 Troubleshooting

### Common Issues

**Issue:** `ImportError: No module named 'cell_dt'`
**Solution:** Virtual environment not activated. Run `source venv/bin/activate` first.

**Issue:** `maturin develop` fails with Rust errors
**Solution:** Update Rust: `rustup update`

**Issue:** Slow performance
**Solution:** Use `--release` flag with maturin and increase `num_threads`

**Issue:** Jupyter notebook doesn't see the module
**Solution:** Install Jupyter in the virtual environment and launch from there

## 📁 Project Structure

```
cell_dt/
├── crates/
│   └── cell_dt_python/
│       ├── src/
│       │   └── lib.rs          # Rust source
│       ├── Cargo.toml          # Rust dependencies
│       ├── example.py          # Python example
│       ├── cell_dt_demo.ipynb  # Jupyter notebook
│       └── README.md           # This file
├── venv/                        # Virtual environment
└── ...
```

## 🛠️ Development

To modify the Python bindings:

1. Edit `src/lib.rs`
2. Rebuild with `maturin develop --release`
3. Test with `python example.py`

To add new features:
1. Define new Rust structs with `#[pyclass]`
2. Implement methods with `#[pymethods]`
3. Add to module in `#[pymodule]` function

## 📝 License

This project is licensed under either of:
- MIT License
- Apache License, Version 2.0

at your option.

## 🤝 Contributing

Contributions are welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Submit a pull request

## 📧 Contact

**Jaba Tkemaladze** - [@djabbat](https://github.com/djabbat)

Project Link: [https://github.com/djabbat/cell_dt](https://github.com/djabbat/cell_dt)

## 🎯 Quick Start Checklist

```bash
# 1. Open terminal
# 2. Activate venv
cd /home/oem/Documents/Projects/rust/cell_dt
source venv/bin/activate

# 3. Navigate to bindings
cd crates/cell_dt_python

# 4. Run example
python example.py

# 5. Launch Jupyter (optional)
jupyter notebook cell_dt_demo.ipynb
```

## 📚 Additional Resources

- [Rust Documentation](https://doc.rust-lang.org/)
- [PyO3 User Guide](https://pyo3.rs/)
- [NumPy Documentation](https://numpy.org/doc/)
- [Jupyter Documentation](https://jupyter.org/documentation)
```