"""
CDATA Digital Twin — Figures for the scientific article.
Generates 3 publication-quality figures and saves to the CDATA folder.
"""

import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import matplotlib.patheffects as pe
from matplotlib.patches import FancyArrowPatch, FancyBboxPatch
from matplotlib.gridspec import GridSpec

OUT = "/home/oem/Desktop/CDATA/"
plt.rcParams.update({
    "font.family": "DejaVu Sans",
    "axes.titlesize": 12,
    "axes.labelsize": 11,
    "legend.fontsize": 9,
    "xtick.labelsize": 9,
    "ytick.labelsize": 9,
})

# ─── Simulation data (from actual Cell DT run) ───────────────────────────────
years   = [0,  10,  20,  30,  40,  50,  60,  70,  80.4]
damage  = [0.006, 0.071, 0.137, 0.203, 0.275, 0.386, 0.499, 0.608, 0.697]
cilia   = [0.988, 0.875, 0.765, 0.660, 0.554, 0.403, 0.265, 0.149, 0.074]
spindle = [0.996, 0.951, 0.904, 0.856, 0.803, 0.721, 0.637, 0.552, 0.471]
frailty = [0.016, 0.173, 0.318, 0.450, 0.573, 0.729, 0.848, 0.928, 0.970]
phenos  = [0, 0, 1, 3, 4, 7, 7, 8, 8]

# Stem cell pool = 1 - pool_exhaustion_probability ≈ 1 - 0.8*damage (approx)
stem_pool = [max(0, 1.0 - 0.8 * d) for d in damage]
# Senescent fraction ≈ 0.85 * damage
sen_frac = [min(1.0, 0.85 * d) for d in damage]

# ─── FIGURE 1 — Simulation Trajectory ────────────────────────────────────────
fig1, axes = plt.subplots(2, 2, figsize=(10, 8))
fig1.suptitle(
    "CDATA Digital Twin: Human Lifespan Simulation\n"
    "(Cell DT Platform, Tkemaladze Centriolar Damage Accumulation Theory)",
    fontsize=13, fontweight="bold", y=0.98
)

DEATH_COLOR = "#cc0000"
DEATH_X = 80.4

def add_death_line(ax):
    ax.axvline(DEATH_X, color=DEATH_COLOR, lw=1.2, ls="--", alpha=0.7)
    ax.text(DEATH_X + 0.5, ax.get_ylim()[1] * 0.95, "death\n~80 yr",
            color=DEATH_COLOR, fontsize=7.5, va="top")

# Panel A: Total damage score
ax = axes[0, 0]
ax.plot(years, damage, "o-", color="#8B0000", lw=2, ms=5, label="Total damage score")
ax.axhline(0.75, color="#8B0000", ls=":", lw=1, alpha=0.6, label="Senescence threshold (0.75)")
ax.axvline(40, color="gray", ls="--", lw=1, alpha=0.5, label="Midlife multiplier onset (40 yr)")
ax.set_xlabel("Age (years)")
ax.set_ylabel("Damage score (0–1)")
ax.set_title("A  Total Centriolar Damage Score")
ax.set_xlim(-2, 95)
ax.set_ylim(0, 1.0)
ax.legend(loc="upper left", fontsize=8)
ax.add_patch(mpatches.Rectangle((40, 0), 50, 1.0, alpha=0.06, color="orange"))
ax.text(41, 0.02, "midlife\n×1.6", fontsize=7.5, color="darkorange")
ax.set_ylim(0, 1.0)
add_death_line(ax)

# Panel B: Track A and Track B
ax = axes[0, 1]
ax.plot(years, cilia,   "s-", color="#1a6faf", lw=2, ms=5, label="Ciliary function (Track A)")
ax.plot(years, spindle, "^-", color="#2ca02c", lw=2, ms=5, label="Spindle fidelity (Track B)")
ax.axhline(0.5, color="gray", ls=":", lw=1, alpha=0.5)
ax.set_xlabel("Age (years)")
ax.set_ylabel("Functional integrity (0–1)")
ax.set_title("B  Track A (Cilia) & Track B (Spindle)")
ax.set_xlim(-2, 95)
ax.set_ylim(0, 1.05)
ax.legend(loc="upper right")
add_death_line(ax)

# Panel C: Stem cell pool and senescent fraction
ax = axes[1, 0]
ax.plot(years, stem_pool, "D-", color="#9467bd", lw=2, ms=5, label="Stem cell pool")
ax.plot(years, sen_frac,  "o-", color="#d62728", lw=2, ms=5, label="Senescent fraction")
ax.set_xlabel("Age (years)")
ax.set_ylabel("Fraction (0–1)")
ax.set_title("C  Stem Cell Pool & Senescent Fraction")
ax.set_xlim(-2, 95)
ax.set_ylim(0, 1.05)
ax.legend(loc="center right")
add_death_line(ax)

# Panel D: Frailty index and aging phenotypes
ax = axes[1, 1]
color_frailty = "#e67c00"
color_phenos  = "#555555"
l1, = ax.plot(years, frailty, "o-", color=color_frailty, lw=2, ms=5, label="Frailty index")
ax.set_xlabel("Age (years)")
ax.set_ylabel("Frailty index (0–1)", color=color_frailty)
ax.tick_params(axis="y", colors=color_frailty)
ax.set_title("D  Frailty Index & Active Aging Phenotypes")
ax.set_xlim(-2, 95)
ax.set_ylim(0, 1.05)

ax2 = ax.twinx()
l2, = ax2.plot(years, phenos, "s--", color=color_phenos, lw=1.5, ms=5, label="Active phenotypes (N)")
ax2.set_ylabel("Phenotype count", color=color_phenos)
ax2.tick_params(axis="y", colors=color_phenos)
ax2.set_ylim(0, 10)
ax.axhline(0.97, color=DEATH_COLOR, ls=":", lw=1, alpha=0.7, label="Death threshold (0.97)")
ax.legend(handles=[l1, l2,
    mpatches.Patch(color=DEATH_COLOR, linestyle=":", fill=False, label="Death threshold")],
    fontsize=8, loc="upper left")
add_death_line(ax)

fig1.tight_layout(rect=[0, 0, 1, 0.95])
fig1.savefig(OUT + "Figure1_SimulationTrajectory.png", dpi=180, bbox_inches="tight")
print("Saved Figure 1")


# ─── FIGURE 2 — CDATA Mechanistic Diagram ────────────────────────────────────
fig2, ax = plt.subplots(figsize=(12, 7))
ax.set_xlim(0, 12)
ax.set_ylim(0, 7)
ax.axis("off")
ax.set_title("CDATA Digital Twin: Centriolar Damage Accumulation Theory — Mechanistic Model",
             fontsize=13, fontweight="bold", pad=12)

def box(ax, x, y, w, h, text, color, fontsize=9, textcolor="white", style="round,pad=0.1"):
    ax.add_patch(FancyBboxPatch((x - w/2, y - h/2), w, h,
                                boxstyle=style, facecolor=color,
                                edgecolor="white", lw=1.5, zorder=3))
    ax.text(x, y, text, ha="center", va="center",
            fontsize=fontsize, color=textcolor, fontweight="bold",
            zorder=4, wrap=True, multialignment="center")

def arrow(ax, x1, y1, x2, y2, color="#333333", label="", lw=1.5):
    ax.annotate("", xy=(x2, y2), xytext=(x1, y1),
                arrowprops=dict(arrowstyle="-|>", color=color,
                                lw=lw, mutation_scale=14), zorder=2)
    if label:
        mx, my = (x1+x2)/2, (y1+y2)/2
        ax.text(mx, my + 0.12, label, ha="center", va="bottom",
                fontsize=7.5, color=color)

# ── Nodes ──────────────────────────────────────────────────────────────────────
# Mother centriole (centre)
box(ax, 6, 5.2, 2.4, 0.8, "Mother Centriole\n(non-renewable template)",
    "#8B0000", fontsize=9)

# Damage sources
box(ax, 2.0, 6.2, 2.2, 0.7, "ROS / oxidative\nstress", "#b5451b", fontsize=8)
box(ax, 6.0, 6.7, 2.2, 0.7, "Age-dependent\nUbiquitin decline", "#b5451b", fontsize=8)
box(ax, 10.0, 6.2, 2.2, 0.7, "SASP /\ninflamm-aging (>45 yr)", "#b5451b", fontsize=8)

# Molecular damage types
box(ax, 2.8, 4.1, 2.0, 0.75,
    "Protein carbonylation\n(SAS-6, CEP135)", "#4a235a", fontsize=7.5)
box(ax, 5.1, 3.3, 2.0, 0.75,
    "Hyperacetylation\n(HDAC6/SIRT2↓)", "#4a235a", fontsize=7.5)
box(ax, 7.4, 4.1, 2.0, 0.75,
    "Protein aggregates\n(CPAP, CEP290)", "#4a235a", fontsize=7.5)
box(ax, 9.4, 3.3, 2.0, 0.75,
    "Phospho-dysregul.\n(PLK4/NEK2/PP1)", "#4a235a", fontsize=7.5)

# Appendage loss
box(ax, 6.0, 2.35, 3.0, 0.65,
    "Distal appendage loss  CEP164 · CEP89 · Ninein · CEP170",
    "#1a3a6f", fontsize=7.5)

# Track A
box(ax, 3.2, 1.3, 2.4, 0.65,
    "Track A: Primary cilium\nloss → Shh/Wnt deaf",
    "#1a6faf", fontsize=8)

# Track B
box(ax, 8.8, 1.3, 2.4, 0.65,
    "Track B: Spindle errors\n→ stem pool exhaustion",
    "#2c7a2c", fontsize=8)

# Death
box(ax, 6.0, 0.45, 3.0, 0.65,
    "Organ failure / Death  (~80 yr)",
    DEATH_COLOR, fontsize=9)

# ROS feedback loop label
ax.annotate("", xy=(2.0, 5.85), xytext=(4.0, 5.0),
            arrowprops=dict(arrowstyle="-|>", color="#ff6600",
                            lw=1.5, connectionstyle="arc3,rad=-0.3",
                            mutation_scale=12), zorder=2)
ax.text(1.2, 5.35, "ROS positive\nfeedback\n(×0.12 per unit)", fontsize=7.5,
        color="#ff6600", ha="center")

# ── Arrows ─────────────────────────────────────────────────────────────────────
# Damage sources → mother centriole
arrow(ax, 2.5, 5.85, 5.0, 5.3, "#b5451b")
arrow(ax, 6.0, 6.35, 6.0, 5.6, "#b5451b")
arrow(ax, 9.5, 5.85, 7.2, 5.3, "#b5451b")

# Mother centriole → damage types
arrow(ax, 5.4, 4.85, 3.5, 4.45, "#4a235a")
arrow(ax, 5.7, 4.85, 5.1, 3.65, "#4a235a")
arrow(ax, 6.3, 4.85, 7.4, 4.45, "#4a235a")
arrow(ax, 6.8, 4.85, 9.0, 3.65, "#4a235a")

# Damage types → appendage loss
arrow(ax, 5.0, 3.72, 5.5, 2.68, "#1a3a6f")
arrow(ax, 7.4, 3.72, 6.8, 2.68, "#1a3a6f")

# Appendage loss → Track A and B
arrow(ax, 5.0, 2.35, 4.0, 1.62, "#1a6faf")
arrow(ax, 7.0, 2.35, 8.2, 1.62, "#2c7a2c")

# Track A/B → death
arrow(ax, 4.2, 1.0, 5.1, 0.65, DEATH_COLOR)
arrow(ax, 7.8, 1.0, 6.9, 0.65, DEATH_COLOR)

# Midlife label
ax.add_patch(mpatches.FancyArrowPatch((6.0, 5.2), (6.0, 4.8),
    arrowstyle="->", color="darkorange", lw=0))
ax.text(8.3, 5.05, "Midlife multiplier ×1.6\n(antagonistic pleiotropy, >40 yr)",
        fontsize=7.5, color="darkorange", ha="left")

fig2.tight_layout()
fig2.savefig(OUT + "Figure2_MechanisticModel.png", dpi=180, bbox_inches="tight")
print("Saved Figure 2")


# ─── FIGURE 3 — Individual Damage Components (simulated) ─────────────────────
# Numerically integrate the CDATA damage ODE to get component trajectories
dt_years = 1.0 / 365.25
t_max = 80.5

# Params (from DamageParams::default())
base_ros   = 0.0076
acetyl_r   = 0.0059
aggreg_r   = 0.0059
phospho_r  = 0.0042
cep164_r   = 0.0113
cep89_r    = 0.0084
ninein_r   = 0.0084
cep170_r   = 0.0067
ros_fb     = 0.12
midlife_m  = 1.6

# State
pc = 0.0   # protein_carbonylation
ha = 0.0   # hyperacetylation
ag = 0.0   # aggregates
ph = 0.0   # phospho-dysregulation
c164 = 1.0
c89  = 1.0
nin  = 1.0
c170 = 1.0
ros  = 0.05

ts, pcs, has_, ags, phs, c164s, c89s, nins, c170s, ross = [], [], [], [], [], [], [], [], [], []

age = 0.0
while age < t_max:
    def total_dmg():
        mol = (pc + ha + ag + ph) / 4.0
        app = (1.0 - c164 + 1.0 - c89 + 1.0 - nin + 1.0 - c170) / 4.0
        return 0.5 * mol + 0.5 * app

    am = midlife_m if age > 40.0 else 1.0
    ros_boost = 1.0 + ros_fb * total_dmg()
    eff = dt_years * am * ros_boost

    pc   = min(1.0, pc  + base_ros * ros * eff)
    ha   = min(1.0, ha  + acetyl_r * eff)
    ag   = min(1.0, ag  + aggreg_r * eff)
    ph   = min(1.0, ph  + phospho_r * eff)
    c164 = max(0.0, c164 - cep164_r * eff)
    c89  = max(0.0, c89  - cep89_r  * eff)
    nin  = max(0.0, nin  - ninein_r  * eff)
    c170 = max(0.0, c170 - cep170_r * eff)
    ros  = min(1.0, 0.05 + age * 0.005 + ros_fb * total_dmg())

    if int(age) % 1 == 0:
        ts.append(age)
        pcs.append(pc); has_.append(ha); ags.append(ag); phs.append(ph)
        c164s.append(c164); c89s.append(c89); nins.append(nin); c170s.append(c170)
        ross.append(ros)

    age += dt_years

ts = np.array(ts)

fig3, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 5))
fig3.suptitle("CDATA Digital Twin: Molecular Damage Component Trajectories",
              fontsize=13, fontweight="bold")

# Panel A: Molecular damage
ax1.plot(ts, pcs,  lw=2, label="Protein carbonylation\n(SAS-6, CEP135 via ROS)", color="#8B0000")
ax1.plot(ts, has_, lw=2, label="Tubulin hyperacetylation\n(HDAC6/SIRT2↓)", color="#d62728")
ax1.plot(ts, ags,  lw=2, label="Protein aggregates\n(CPAP, CEP290)", color="#e67c00")
ax1.plot(ts, phs,  lw=2, label="Phospho-dysregulation\n(PLK4/NEK2/PP1)", color="#9467bd")
ax1.plot(ts, ross, lw=1.5, ls="--", label="ROS level", color="#aaaaaa")
ax1.axvline(40, color="darkorange", ls=":", lw=1, alpha=0.7)
ax1.axvline(DEATH_X, color=DEATH_COLOR, ls="--", lw=1, alpha=0.7)
ax1.text(40.5, 0.96, "×1.6", color="darkorange", fontsize=8)
ax1.text(DEATH_X + 0.3, 0.89, "death", color=DEATH_COLOR, fontsize=8)
ax1.set_xlabel("Age (years)")
ax1.set_ylabel("Damage / Level (0–1)")
ax1.set_title("A  Molecular Damage Components")
ax1.set_xlim(0, 85)
ax1.set_ylim(0, 1.05)
ax1.legend(loc="upper left", fontsize=7.5)

# Panel B: Distal appendage integrity
ax2.plot(ts, c164s, lw=2, label="CEP164 integrity\n(ciliary initiation)", color="#1a6faf")
ax2.plot(ts, c89s,  lw=2, label="CEP89 integrity",  color="#17becf")
ax2.plot(ts, nins,  lw=2, label="Ninein integrity\n(subdistal)",         color="#2ca02c")
ax2.plot(ts, c170s, lw=2, label="CEP170 integrity",  color="#98df8a")

# Track A shading: ciliary function = (c164 + c89) / 2
cilia_fn = [(a + b) / 2 for a, b in zip(c164s, c89s)]
ax2.fill_between(ts, cilia_fn, alpha=0.1, color="#1a6faf", label="Track A (ciliary function)")

ax2.axvline(40, color="darkorange", ls=":", lw=1, alpha=0.7)
ax2.axvline(DEATH_X, color=DEATH_COLOR, ls="--", lw=1, alpha=0.7)
ax2.text(40.5, 0.04, "×1.6", color="darkorange", fontsize=8)
ax2.text(DEATH_X + 0.3, 0.12, "death", color=DEATH_COLOR, fontsize=8)
ax2.set_xlabel("Age (years)")
ax2.set_ylabel("Structural integrity (0–1, 1=intact)")
ax2.set_title("B  Distal Appendage Integrity (Track A)")
ax2.set_xlim(0, 85)
ax2.set_ylim(0, 1.05)
ax2.legend(loc="upper right", fontsize=7.5)

fig3.tight_layout()
fig3.savefig(OUT + "Figure3_DamageComponents.png", dpi=180, bbox_inches="tight")
print("Saved Figure 3")

print("All figures saved to", OUT)
