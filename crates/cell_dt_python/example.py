#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Пример использования Cell DT из Python
"""

import cell_dt
import numpy as np
import matplotlib.pyplot as plt
from collections import Counter

def basic_simulation():
    """Базовая симуляция"""
    print("=" * 50)
    print("Basic Simulation")
    print("=" * 50)
    
    # Создаем симуляцию
    sim = cell_dt.PySimulation(
        max_steps=500,
        dt=0.1,
        num_threads=4,
        seed=42
    )
    
    # Создаем популяцию клеток
    sim.create_population(100)
    print(f"Created {sim.cell_count()} cells")
    
    # Регистрируем модули
    sim.register_modules(
        enable_centriole=True,
        enable_cell_cycle=True,
        enable_transcriptome=False,
        cell_cycle_params=None  # используем параметры по умолчанию
    )
    
    # Запускаем симуляцию
    print("Running simulation...")
    cells = sim.run()
    print(f"Simulation completed at step {sim.current_step()}")
    
    # Анализируем результаты
    phases = [cell.cell_cycle.phase for cell in cells]
    phase_counts = Counter(phases)
    
    print("\nPhase distribution:")
    for phase, count in phase_counts.items():
        print(f"  {phase}: {count}")
    
    return cells

def advanced_simulation():
    """Продвинутая симуляция с транскриптомом и визуализацией"""
    print("\n" + "=" * 50)
    print("Advanced Simulation with Transcriptome")
    print("=" * 50)
    
    # Настраиваем параметры клеточного цикла
    cell_cycle_params = cell_dt.PyCellCycleParams(
        base_cycle_time=20.0,
        growth_factor_sensitivity=0.3,
        stress_sensitivity=0.2,
        checkpoint_strictness=0.15,
        enable_apoptosis=True,
        nutrient_availability=0.9,
        growth_factor_level=0.85,
        random_variation=0.25
    )
    
    # Создаем симуляцию
    sim = cell_dt.PySimulation(
        max_steps=1000,
        dt=0.05,
        num_threads=4,
        seed=42
    )
    
    # Создаем популяцию с транскриптомом
    sim.create_population_with_transcriptome(50)
    print(f"Created {sim.cell_count()} cells with transcriptome")
    
    # Регистрируем все модули
    sim.register_modules(
        enable_centriole=True,
        enable_cell_cycle=True,
        enable_transcriptome=True,
        cell_cycle_params=cell_cycle_params
    )
    
    # Пошаговый запуск с анализом
    print("\nRunning step-by-step simulation...")
    
    phase_history = []
    
    for step in range(0, 1000, 100):
        cells = sim.step(100)
        print(f"Step {sim.current_step()}: {len(cells)} cells")
        
        # Собираем статистику по фазам
        phases = [cell.cell_cycle.phase for cell in cells]
        phase_counts = Counter(phases)
        phase_history.append(phase_counts)
    
    return sim, phase_history

def visualize_results(sim, phase_history):
    """Визуализация результатов"""
    print("\n" + "=" * 50)
    print("Visualization")
    print("=" * 50)
    
    # Получаем данные из NumPy
    import numpy as np
    
    # Данные центриолей
    centriole_data = sim.get_centriole_data_numpy()
    print(f"Centriole data shape: {centriole_data.shape}")
    
    # Распределение фаз
    phase_dist = sim.get_phase_distribution()
    print("Phase distribution:", dict(phase_dist))
    
    # Создаем графики
    fig, axes = plt.subplots(2, 2, figsize=(12, 10))
    
    # 1. Гистограмма зрелости центриолей
    ax = axes[0, 0]
    ax.hist(centriole_data[:, 0], bins=20, alpha=0.5, label='Mother')
    ax.hist(centriole_data[:, 1], bins=20, alpha=0.5, label='Daughter')
    ax.set_xlabel('Maturity')
    ax.set_ylabel('Count')
    ax.set_title('Centriole Maturity Distribution')
    ax.legend()
    
    # 2. Активность MTOC
    ax = axes[0, 1]
    ax.hist(centriole_data[:, 2], bins=20, color='green', alpha=0.7)
    ax.set_xlabel('MTOC Activity')
    ax.set_ylabel('Count')
    ax.set_title('MTOC Activity Distribution')
    
    # 3. Эволюция фаз во времени
    ax = axes[1, 0]
    steps = list(range(0, 1000, 100))
    for phase in ['G1', 'S', 'G2', 'M']:
        counts = [history.get(phase, 0) for history in phase_history]
        ax.plot(steps, counts, label=phase, marker='o')
    ax.set_xlabel('Step')
    ax.set_ylabel('Number of cells')
    ax.set_title('Cell Cycle Phase Evolution')
    ax.legend()
    ax.grid(True, alpha=0.3)
    
    # 4. Текущее распределение фаз
    ax = axes[1, 1]
    phases = list(phase_dist.keys())
    counts = list(phase_dist.values())
    ax.bar(phases, counts, color=['blue', 'green', 'orange', 'red'])
    ax.set_xlabel('Phase')
    ax.set_ylabel('Count')
    ax.set_title('Current Phase Distribution')
    
    plt.tight_layout()
    plt.savefig('cell_dt_analysis.png', dpi=150)
    plt.show()
    print("Plot saved as 'cell_dt_analysis.png'")

def main():
    """Основная функция"""
    print("Cell DT Python Interface Example")
    print("=" * 50)
    
    # Базовая симуляция
    cells = basic_simulation()
    
    # Продвинутая симуляция
    sim, phase_history = advanced_simulation()
    
    # Визуализация
    visualize_results(sim, phase_history)
    
    # Анализ транскриптома
    print("\n" + "=" * 50)
    print("Transcriptome Analysis")
    print("=" * 50)
    
    cells = sim.get_cell_data()
    stats = cell_dt.analyze_transcriptome(cells)
    
    for key, value in stats.items():
        print(f"{key}: {value:.3f}")

if __name__ == "__main__":
    main()
