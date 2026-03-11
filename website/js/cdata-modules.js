/**
 * CDATA Website — Module Renderers
 */

import { CDATA, DamageParams, CellState, getPotency, potencyScore, InterventionComparison, InducerSimulator, CellCycleSimulator } from './cdata-core.js';

function fmt(n, d = 3) { return isFinite(n) ? Number(n).toFixed(d) : '—'; }
function pct(n) { return fmt(n * 100, 1) + '%'; }

function potencyClass(p) {
  return { Totipotent: 'p-totipotent', Pluripotent: 'p-pluripotent', Oligopotent: 'p-oligopotent', Unipotent: 'p-unipotent', Apoptosis: 'p-apoptosis' }[p] ?? '';
}

// ─── Color scale: green → amber → red (damage level) ─────────────────────────
function damageColor(d) {
  if (d < 0.33) {
    const t = d / 0.33;
    const r = Math.round(52 + (251 - 52) * t);
    const g = Math.round(211 + (191 - 211) * t);
    const b = Math.round(153 + (36 - 153) * t);
    return `rgb(${r},${g},${b})`;
  } else {
    const t = (d - 0.33) / 0.67;
    const r = Math.round(251 + (239 - 251) * t);
    const g = Math.round(191 + (68 - 191) * t);
    const b = Math.round(36 + (68 - 36) * t);
    return `rgb(${r},${g},${b})`;
  }
}

// ─── Module 0: Overview ───────────────────────────────────────────────────────
export function initOverview() {
  document.getElementById('m-overview').innerHTML = `
    <div class="module-header">
      <div class="module-title">CDATA — Centriolar Damage Accumulation Theory of Aging</div>
      <div class="module-desc">
        CDATA proposes that the irreversible loss of centriolar inducer molecules — driven by ROS and PTM accumulation — is the
        causally upstream mechanism from which all aging hallmarks emerge. The centriole is the organizing center of both the
        primary cilium and the mitotic spindle. When inducers are lost, these two structures fail — triggering five interlinked aging tracks.
      </div>
    </div>
    <div class="grid-2">
      <div class="card">
        <div class="card-title">The Core Mechanism</div>
        <div style="font-size:13px;color:var(--muted);line-height:1.8">
          <div style="display:flex;flex-direction:column;gap:10px">
            ${[
              ['🔵','Mitochondria at cell periphery absorb O₂ — the mito-shield protects the centriole'],
              ['🔴','ROS and PTMs accumulate → shield degrades → O₂ reaches the centrosome'],
              ['⚡','O₂ irreversibly displaces inducer molecules from M-set (mother) and D-set (daughter) centrioles'],
              ['🧬','Potency is defined by the combined state of M and D inducers (see table)'],
              ['📉','Lower potency → impaired cilia (Track A) → impaired spindle (Track B) → aging'],
              ['♻️','ROS → more damage → more ROS: a positive feedback loop'],
            ].map(([i,t]) => `<div style="display:flex;gap:10px"><span style="font-size:16px;min-width:24px">${i}</span><span>${t}</span></div>`).join('')}
          </div>
        </div>
      </div>
      <div class="card">
        <div class="card-title">Potency Table (M-set × D-set)</div>
        <table class="cdata-table">
          <thead><tr><th>M-set</th><th>D-set</th><th>Potency</th><th>Score</th></tr></thead>
          <tbody>
            <tr><td>Full (10)</td><td>Full (8)</td><td><span class="potency p-totipotent">Totipotent</span></td><td>1.00</td></tr>
            <tr><td>≥1</td><td>≥1</td><td><span class="potency p-pluripotent">Pluripotent</span></td><td>0.75</td></tr>
            <tr><td>0</td><td>≥2 (or vice versa)</td><td><span class="potency p-oligopotent">Oligopotent</span></td><td>0.50</td></tr>
            <tr><td>0</td><td>1 (or vice versa)</td><td><span class="potency p-unipotent">Unipotent</span></td><td>0.25</td></tr>
            <tr><td>0</td><td>0</td><td><span class="potency p-apoptosis">Apoptosis</span></td><td>0.00</td></tr>
          </tbody>
        </table>
        <div class="info-box" style="margin-top:10px">
          <strong>O₂ detachment:</strong> O₂ preferentially attacks the mother centriole (older, more PTMs → weaker bonds).
          Mother bias parameter = 0.5 by default (equal M/D targeting).
          <strong>Daughter centrioles inherit the CURRENT (reduced) inducer count</strong>, not the original maximum.
          This is the mechanism of transgenerational aging.
        </div>
      </div>
      <div class="card card-full">
        <div class="card-title">Five Aging Tracks</div>
        <div class="grid-3" style="margin-top:0">
          ${[
            ['A','Cilia','teal','CEP164↓ CEP89↓ → primary cilium lost → Shh/Wnt/Notch signaling fails → niche collapse → no self-renewal'],
            ['B','Spindle','violet','spindle_fidelity↓ → symmetric divisions → stem cell pool depleted (Hayflick-equivalent)'],
            ['C','Telomere','blue','shortening ∝ spindle_fidelity × ROS × division_rate → length < 0.3 → G1 arrest (Hayflick limit)'],
            ['D','Epigenetic','gold','methylation_age += dt × (1 + damage × 0.5) → clock_acceleration → extra ROS feedback'],
            ['E','Mitochondrial','red','mtDNA mutations → ROS overproduction → mito fragmentation → shield collapse → more O₂ at centrosome'],
          ].map(([t,n,c,desc]) => `
            <div class="card" style="border-color:var(--${c})">
              <div style="display:flex;align-items:center;gap:8px;margin-bottom:8px">
                <span class="track-badge track-${t.toLowerCase()}" style="font-size:14px">Track ${t}</span>
                <span style="font-weight:700;color:var(--${c})">${n}</span>
              </div>
              <div style="font-size:13px;color:var(--muted);line-height:1.6">${desc}</div>
            </div>
          `).join('')}
          <div class="card" style="border-color:var(--orange)">
            <div style="display:flex;align-items:center;gap:8px;margin-bottom:8px">
              <span style="font-weight:700;color:var(--orange)">Myeloid Shift</span>
            </div>
            <div style="font-size:13px;color:var(--muted);line-height:1.6">
              spindle↓ + cilia↓ + ROS↑ → PU.1 > Ikaros → hematopoietic bias toward myeloid → inflammaging → SASP
              <br><span class="eq" style="font-family:var(--mono);color:var(--teal);font-size:11px">myeloid_bias = (1−sf)^1.5×0.45 + (1−cilia)×0.30 + ros×0.15 + agg×0.10</span>
            </div>
          </div>
        </div>
      </div>
      <div class="card card-full">
        <div class="card-title">Key Calibration (Default Parameters)</div>
        <div class="stats-grid">
          <div class="stat-box highlight"><div class="stat-label">Death age</div><div class="stat-value green">~78 yrs</div><div class="stat-sub">default params</div></div>
          <div class="stat-box"><div class="stat-label">Senescence threshold</div><div class="stat-value">0.75</div><div class="stat-sub">damage ≥ this → death</div></div>
          <div class="stat-box"><div class="stat-label">Progeria</div><div class="stat-value red">×5 rate</div><div class="stat-sub">accelerated</div></div>
          <div class="stat-box"><div class="stat-label">Longevity</div><div class="stat-value green">×0.6 rate</div><div class="stat-sub">decelerated</div></div>
          <div class="stat-box"><div class="stat-label">Midlife multiplier</div><div class="stat-value orange">×1.6</div><div class="stat-sub">after age 40</div></div>
          <div class="stat-box"><div class="stat-label">M inducers (initial)</div><div class="stat-value">10</div><div class="stat-sub">mother centriole</div></div>
          <div class="stat-box"><div class="stat-label">D inducers (initial)</div><div class="stat-value">8</div><div class="stat-sub">daughter centriole</div></div>
          <div class="stat-box"><div class="stat-label">Myeloid bias age 70</div><div class="stat-value orange">~0.57</div><div class="stat-sub">ModerateShift</div></div>
        </div>
      </div>
    </div>
  `;
}

// ─── Module 1: Inducer Simulation ─────────────────────────────────────────────
export function initInducer() {
  document.getElementById('m-inducer').innerHTML = `
    <div class="module-header">
      <div class="module-title">Centriolar Inducer System</div>
      <div class="module-desc">
        Simulate the step-by-step loss of M-set and D-set inducer molecules from the centriole.
        O₂ penetration (controlled by the mitochondrial shield) determines how quickly inducers are lost.
        When both sets reach zero — apoptosis is triggered.
      </div>
      <div class="module-equation">detach_prob = base_prob × O₂_penetration × age_multiplier &nbsp;·&nbsp; O₂ = 1 − mito_shield</div>
    </div>
    <div class="grid-2">
      <div class="card">
        <div class="card-title">Parameters</div>
        <div class="controls">
          <div class="control-group">
            <label>M-set initial</label>
            <input type="range" class="cdata-input" id="ind-m" min="1" max="20" value="10">
            <span id="ind-m-val">10</span>
          </div>
          <div class="control-group">
            <label>D-set initial</label>
            <input type="range" class="cdata-input" id="ind-d" min="1" max="20" value="8">
            <span id="ind-d-val">8</span>
          </div>
          <div class="control-group">
            <label>O₂ penetration</label>
            <input type="range" class="cdata-input" id="ind-o2" min="0.01" max="1" value="0.5" step="0.01">
            <span id="ind-o2-val">0.50</span>
          </div>
          <div class="control-group">
            <label>Mother bias</label>
            <input type="range" class="cdata-input" id="ind-bias" min="0" max="1" value="0.5" step="0.05">
            <span id="ind-bias-val">0.50</span>
          </div>
        </div>
        <div style="display:flex;gap:8px;flex-wrap:wrap;margin-top:4px">
          <button class="cdata-btn" id="ind-step">Step × 1</button>
          <button class="cdata-btn" id="ind-step10">Step × 10</button>
          <button class="cdata-btn" id="ind-step100">Step × 100</button>
          <button class="cdata-btn secondary" id="ind-divide">🔀 Divide</button>
          <button class="cdata-btn secondary" id="ind-reset">Reset</button>
        </div>
        <div class="stats-grid" style="margin-top:14px">
          <div class="stat-box highlight"><div class="stat-label">M-set</div><div class="stat-value green" id="ind-s-m">10</div><div class="stat-sub">mother</div></div>
          <div class="stat-box highlight"><div class="stat-label">D-set</div><div class="stat-value teal" id="ind-s-d">8</div><div class="stat-sub">daughter</div></div>
          <div class="stat-box"><div class="stat-label">Potency</div><div class="stat-value" id="ind-s-pot" style="font-size:13px">Totipotent</div></div>
          <div class="stat-box"><div class="stat-label">Steps</div><div class="stat-value" id="ind-s-steps">0</div></div>
          <div class="stat-box"><div class="stat-label">Divisions</div><div class="stat-value" id="ind-s-div">0</div></div>
        </div>
        <div class="info-box" style="margin-top:10px">
          <strong>Divide:</strong> Creates a daughter cell inheriting the <em>current</em> (reduced) inducer counts.
          The original cell keeps its counts. This models transgenerational inducer inheritance.
        </div>
      </div>
      <div class="card">
        <div class="card-title">Inducer History</div>
        <div class="canvas-wrap" style="height:320px">
          <canvas id="ind-canvas" height="320"></canvas>
        </div>
        <div id="ind-event-log" style="margin-top:10px;max-height:80px;overflow-y:auto;font-size:11px;color:var(--muted);font-family:var(--mono)"></div>
      </div>
    </div>
  `;

  let sim = new InducerSimulator({
    m: 10, d: 8, detach_prob: CDATA.BASE_DETACH_PROB, mother_bias: 0.5
  });

  const controls = ['ind-m','ind-d','ind-o2','ind-bias'];
  controls.forEach(id => {
    document.getElementById(id).oninput = function() {
      document.getElementById(id+'-val').textContent = parseFloat(this.value).toFixed(2);
    };
  });

  function getParams() {
    return {
      m: parseInt(document.getElementById('ind-m').value),
      d: parseInt(document.getElementById('ind-d').value),
      o2: parseFloat(document.getElementById('ind-o2').value),
      bias: parseFloat(document.getElementById('ind-bias').value),
    };
  }

  function updateStats() {
    document.getElementById('ind-s-m').textContent = sim.m;
    document.getElementById('ind-s-d').textContent = sim.d;
    const pot = sim.potency;
    const el = document.getElementById('ind-s-pot');
    el.textContent = pot;
    el.className = 'stat-value potency ' + potencyClass(pot);
    el.style.fontSize = '12px';
    document.getElementById('ind-s-steps').textContent = sim.history.length - 1;
    document.getElementById('ind-s-div').textContent = sim.divisions;
    drawInducerHistory(sim);
  }

  function doSteps(n) {
    const { o2 } = getParams();
    for (let i = 0; i < n; i++) sim.step(o2);
    updateStats();
  }

  document.getElementById('ind-step').onclick = () => doSteps(1);
  document.getElementById('ind-step10').onclick = () => doSteps(10);
  document.getElementById('ind-step100').onclick = () => doSteps(100);
  document.getElementById('ind-divide').onclick = () => {
    const child = sim.divide();
    const log = document.getElementById('ind-event-log');
    log.innerHTML += `<div>Division #${sim.divisions}: child inherits M=${child.m}, D=${child.d} → <span class="potency ${potencyClass(child.potency)}">${child.potency}</span></div>`;
    log.scrollTop = log.scrollHeight;
    sim = child;
    updateStats();
  };
  document.getElementById('ind-reset').onclick = () => {
    const p = getParams();
    sim = new InducerSimulator({ m: p.m, d: p.d, detach_prob: CDATA.BASE_DETACH_PROB, mother_bias: p.bias });
    document.getElementById('ind-event-log').innerHTML = '';
    updateStats();
  };

  updateStats();
}

function drawInducerHistory(sim) {
  const canvas = document.getElementById('ind-canvas');
  const W = canvas.offsetWidth || 500; const H = 320;
  canvas.width = W; canvas.height = H;
  const ctx = canvas.getContext('2d');
  ctx.fillStyle = '#0a1f18'; ctx.fillRect(0,0,W,H);

  const maxInducers = Math.max(CDATA.INDUCER_M_INIT, CDATA.INDUCER_D_INIT) + 2;
  const hist = sim.history;
  const N = hist.length;
  if (N < 2) return;

  const pad = {l:36,r:12,t:12,b:30};
  const pw = W-pad.l-pad.r, ph = H-pad.t-pad.b;

  // Grid
  ctx.strokeStyle='#1a3d2e'; ctx.lineWidth=1;
  for (let y = 0; y <= maxInducers; y += 2) {
    const py = pad.t + ph*(1 - y/maxInducers);
    ctx.beginPath(); ctx.moveTo(pad.l,py); ctx.lineTo(pad.l+pw,py); ctx.stroke();
    ctx.fillStyle='#475569'; ctx.font='10px sans-serif'; ctx.fillText(y, 2, py+4);
  }

  // Potency color bands
  const bands = [
    { min: 0, max: 0.25*maxInducers, color: 'rgba(239,68,68,0.05)' },
    { min: 0.25*maxInducers, max: 0.5*maxInducers, color: 'rgba(249,115,22,0.05)' },
  ];
  bands.forEach(b => {
    ctx.fillStyle = b.color;
    ctx.fillRect(pad.l, pad.t+ph*(1-b.max/maxInducers), pw, ph*(b.max-b.min)/maxInducers);
  });

  const xFor = i => pad.l + (i/(N-1))*pw;

  // M line (green)
  ctx.strokeStyle='#10b981'; ctx.lineWidth=2;
  ctx.beginPath();
  hist.forEach((h,i) => {
    const x=xFor(i), y=pad.t+ph*(1-h.m/maxInducers);
    i===0?ctx.moveTo(x,y):ctx.lineTo(x,y);
  });
  ctx.stroke();

  // D line (teal)
  ctx.strokeStyle='#14b8a6'; ctx.lineWidth=2;
  ctx.beginPath();
  hist.forEach((h,i) => {
    const x=xFor(i), y=pad.t+ph*(1-h.d/maxInducers);
    i===0?ctx.moveTo(x,y):ctx.lineTo(x,y);
  });
  ctx.stroke();

  // Current point
  const last = hist[N-1];
  [['#10b981', last.m],['#14b8a6', last.d]].forEach(([c,v]) => {
    ctx.beginPath(); ctx.arc(xFor(N-1), pad.t+ph*(1-v/maxInducers), 5,0,Math.PI*2);
    ctx.fillStyle=c; ctx.fill();
  });

  // Legend
  ctx.fillStyle='#10b981'; ctx.fillRect(pad.l, H-18, 10, 8); ctx.fillStyle='#94a3b8'; ctx.font='10px sans-serif'; ctx.fillText('M-set', pad.l+14, H-10);
  ctx.fillStyle='#14b8a6'; ctx.fillRect(pad.l+60, H-18, 10, 8); ctx.fillText('D-set', pad.l+74, H-10);
  ctx.fillStyle='#94a3b8'; ctx.fillText(`Steps: ${N-1}`, pad.l+pw-60, H-10);
}

// ─── Module 2: Lifetime Simulation ────────────────────────────────────────────
export function initLifetime() {
  document.getElementById('m-lifetime').innerHTML = `
    <div class="module-header">
      <div class="module-title">Full Lifetime Simulation (0–120 years)</div>
      <div class="module-desc">
        Simulate the complete aging trajectory of a stem cell niche from birth to death.
        All five tracks run simultaneously with feedback loops. Adjust parameters and see
        how they affect lifespan and the balance of aging mechanisms.
      </div>
    </div>
    <div class="grid-2">
      <div class="card">
        <div class="card-title">Scenario</div>
        <div class="controls">
          <div class="control-group">
            <label>Preset</label>
            <select class="cdata-select" id="lt-preset">
              <option value="default">Default (normal aging)</option>
              <option value="progeria">Progeria (×5 damage)</option>
              <option value="longevity">Longevity (×0.6 damage)</option>
              <option value="senolytics">Senolytics intervention</option>
              <option value="nadplus">NAD+ Supplementation</option>
              <option value="caloric_restriction">Caloric Restriction</option>
            </select>
          </div>
          <button class="cdata-btn" id="lt-run">▶ Run Simulation</button>
        </div>
        <div class="stats-grid" style="margin-top:12px" id="lt-stats">
          <div class="stat-box highlight"><div class="stat-label">Death age</div><div class="stat-value green" id="lt-death">—</div></div>
          <div class="stat-box"><div class="stat-label">Death cause</div><div class="stat-value" id="lt-cause" style="font-size:11px;line-height:1.3">—</div></div>
          <div class="stat-box"><div class="stat-label">Damage at death</div><div class="stat-value red" id="lt-dmg">—</div></div>
          <div class="stat-box"><div class="stat-label">Final potency</div><div class="stat-value" id="lt-pot" style="font-size:12px">—</div></div>
          <div class="stat-box"><div class="stat-label">Myeloid bias age70</div><div class="stat-value orange" id="lt-myelo">—</div></div>
          <div class="stat-box"><div class="stat-label">Telomere at death</div><div class="stat-value blue" id="lt-telo">—</div></div>
        </div>
        <div class="info-box warn" style="margin-top:10px" id="lt-note">
          Click "Run Simulation" to start.
        </div>
      </div>
      <div class="card">
        <div class="card-title">Damage Accumulation Over Lifetime</div>
        <div class="canvas-wrap" style="height:300px">
          <canvas id="lt-canvas" height="300"></canvas>
        </div>
      </div>
      <div class="card card-full">
        <div class="card-title">All Tracks Over Time</div>
        <div class="canvas-wrap" style="height:260px">
          <canvas id="lt-tracks-canvas" height="260"></canvas>
        </div>
      </div>
    </div>
  `;

  document.getElementById('lt-run').onclick = runLifetime;
  runLifetime();
}

function runLifetime() {
  const preset = document.getElementById('lt-preset').value;
  const paramsMap = {
    default: DamageParams.default(), progeria: DamageParams.progeria(),
    longevity: DamageParams.longevity(), senolytics: DamageParams.senolytics(),
    nadplus: DamageParams.nadplus(), caloric_restriction: DamageParams.caloric_restriction()
  };
  const cell = new CellState(paramsMap[preset]);
  const snapshots = cell.runLifetime(120);

  const last = snapshots[snapshots.length - 1];
  const at70 = snapshots.find(s => s.age >= 70);

  document.getElementById('lt-death').textContent = last.age.toFixed(1) + ' yrs';
  document.getElementById('lt-death').style.color = last.age > 90 ? 'var(--green3)' : last.age > 70 ? 'var(--gold)' : 'var(--red)';
  document.getElementById('lt-cause').textContent = last.death_cause ?? 'Max age reached';
  document.getElementById('lt-dmg').textContent = pct(last.total_damage);
  document.getElementById('lt-pot').textContent = last.potency;
  document.getElementById('lt-pot').className = 'stat-value potency ' + potencyClass(last.potency);
  document.getElementById('lt-myelo').textContent = at70 ? fmt(at70.myeloid_bias) : '—';
  document.getElementById('lt-telo').textContent = pct(last.telomere_length);

  drawLifetimeDamage(snapshots);
  drawLifetimeTracks(snapshots);
}

function drawLifetimeDamage(snaps) {
  const canvas = document.getElementById('lt-canvas');
  const W = canvas.offsetWidth||700, H=300;
  canvas.width=W; canvas.height=H;
  const ctx=canvas.getContext('2d');
  ctx.fillStyle='#0a1f18'; ctx.fillRect(0,0,W,H);
  const pad={l:40,r:16,t:16,b:30};
  const pw=W-pad.l-pad.r, ph=H-pad.t-pad.b;
  const maxAge = snaps[snaps.length-1].age;

  // Threshold line
  const thy = pad.t + ph*(1-0.75);
  ctx.strokeStyle='rgba(239,68,68,0.5)'; ctx.lineWidth=1; ctx.setLineDash([4,4]);
  ctx.beginPath(); ctx.moveTo(pad.l,thy); ctx.lineTo(pad.l+pw,thy); ctx.stroke();
  ctx.setLineDash([]);
  ctx.fillStyle='rgba(239,68,68,0.7)'; ctx.font='10px sans-serif';
  ctx.fillText('death threshold 0.75', pad.l+4, thy-4);

  // Grid
  ctx.strokeStyle='#1a3d2e'; ctx.lineWidth=1;
  [0,0.25,0.5,0.75,1].forEach(y=>{
    const py=pad.t+ph*(1-y);
    ctx.beginPath(); ctx.moveTo(pad.l,py); ctx.lineTo(pad.l+pw,py); ctx.stroke();
    ctx.fillStyle='#475569'; ctx.font='10px sans-serif'; ctx.fillText(y.toFixed(2),2,py+4);
  });
  [0,20,40,60,80,100,120].forEach(age=>{
    const x=pad.l+(age/maxAge)*pw;
    ctx.beginPath(); ctx.moveTo(x,pad.t); ctx.lineTo(x,pad.t+ph); ctx.stroke();
    ctx.fillStyle='#475569'; ctx.fillText(age,x-6,H-4);
  });

  // Damage fill
  ctx.beginPath();
  snaps.forEach((s,i)=>{
    const x=pad.l+(s.age/maxAge)*pw, y=pad.t+ph*(1-s.total_damage);
    i===0?ctx.moveTo(x,y):ctx.lineTo(x,y);
  });
  ctx.lineTo(pad.l+pw,pad.t+ph); ctx.lineTo(pad.l,pad.t+ph); ctx.closePath();
  ctx.fillStyle='rgba(16,185,129,0.08)'; ctx.fill();

  // Damage line (color coded)
  for (let i=1;i<snaps.length;i++) {
    const s=snaps[i], sp=snaps[i-1];
    const x1=pad.l+(sp.age/maxAge)*pw, y1=pad.t+ph*(1-sp.total_damage);
    const x2=pad.l+(s.age/maxAge)*pw, y2=pad.t+ph*(1-s.total_damage);
    ctx.strokeStyle=damageColor(s.total_damage); ctx.lineWidth=2;
    ctx.beginPath(); ctx.moveTo(x1,y1); ctx.lineTo(x2,y2); ctx.stroke();
  }

  // Midlife marker
  const midX=pad.l+(40/maxAge)*pw;
  ctx.strokeStyle='rgba(249,115,22,0.4)'; ctx.lineWidth=1; ctx.setLineDash([2,4]);
  ctx.beginPath(); ctx.moveTo(midX,pad.t); ctx.lineTo(midX,pad.t+ph); ctx.stroke(); ctx.setLineDash([]);
  ctx.fillStyle='rgba(249,115,22,0.7)'; ctx.font='10px sans-serif'; ctx.fillText('midlife', midX+3, pad.t+14);

  ctx.fillStyle='#94a3b8'; ctx.fillText('Age (years) →', pad.l+pw/2-30, H-4);
  ctx.fillStyle='#10b981'; ctx.fillText('Total damage', pad.l+4, pad.t+14);
}

function drawLifetimeTracks(snaps) {
  const canvas = document.getElementById('lt-tracks-canvas');
  const W = canvas.offsetWidth||900, H=260;
  canvas.width=W; canvas.height=H;
  const ctx=canvas.getContext('2d');
  ctx.fillStyle='#0a1f18'; ctx.fillRect(0,0,W,H);
  const pad={l:40,r:16,t:16,b:30};
  const pw=W-pad.l-pad.r, ph=H-pad.t-pad.b;
  const maxAge = snaps[snaps.length-1].age;

  const tracks = [
    { key:'ciliary_function', label:'Track A: Cilia', color:'#14b8a6' },
    { key:'spindle_fidelity', label:'Track B: Spindle', color:'#8b5cf6' },
    { key:'telomere_length', label:'Track C: Telomere', color:'#3b82f6' },
    { key:'mito_shield', label:'Track E: Mito Shield', color:'#ef4444' },
    { key:'myeloid_bias', label:'Myeloid Bias', color:'#f97316' },
    { key:'stem_cell_pool', label:'Stem Cell Pool', color:'#10b981' },
  ];

  ctx.strokeStyle='#1a3d2e'; ctx.lineWidth=1;
  [0,0.5,1].forEach(y=>{
    const py=pad.t+ph*(1-y);
    ctx.beginPath(); ctx.moveTo(pad.l,py); ctx.lineTo(pad.l+pw,py); ctx.stroke();
    ctx.fillStyle='#475569'; ctx.font='10px sans-serif'; ctx.fillText(y.toFixed(1),2,py+4);
  });

  tracks.forEach(tr => {
    ctx.strokeStyle=tr.color; ctx.lineWidth=1.5;
    ctx.beginPath();
    snaps.forEach((s,i)=>{
      const x=pad.l+(s.age/maxAge)*pw;
      const v = s[tr.key] ?? 0;
      const y=pad.t+ph*(1-Math.min(1,Math.max(0,v)));
      i===0?ctx.moveTo(x,y):ctx.lineTo(x,y);
    });
    ctx.stroke();
  });

  // Legend
  const lx=pad.l, ly=H-12;
  tracks.forEach((tr,i)=>{
    const x=lx+i*145;
    ctx.fillStyle=tr.color; ctx.fillRect(x, ly-8, 12, 8);
    ctx.fillStyle='#94a3b8'; ctx.font='10px sans-serif'; ctx.fillText(tr.label.split(':')[1]?.trim()??tr.label, x+14, ly);
  });
  ctx.fillStyle='#94a3b8'; ctx.fillText('Age →', pad.l+pw-28, H-4);
}

// ─── Module 3: Five Tracks ────────────────────────────────────────────────────
export function initTracks() {
  document.getElementById('m-tracks').innerHTML = `
    <div class="module-header">
      <div class="module-title">Five Aging Tracks — Live Monitor</div>
      <div class="module-desc">
        Watch all five tracks evolve simultaneously. Each track feeds back into the others through ROS, inflammaging, and myeloid shift.
      </div>
    </div>
    <div class="grid-2">
      <div class="card">
        <div class="card-title">Simulation Control</div>
        <div class="controls">
          <div class="control-group">
            <label>Simulation speed</label>
            <select class="cdata-select" id="tr-speed">
              <option value="1">1×</option>
              <option value="5">5×</option>
              <option value="30" selected>30× (1 month/frame)</option>
              <option value="365">365× (1 year/frame)</option>
            </select>
          </div>
          <button class="cdata-btn" id="tr-start">▶ Start</button>
          <button class="cdata-btn secondary" id="tr-pause">⏸ Pause</button>
          <button class="cdata-btn secondary" id="tr-reset">↺ Reset</button>
        </div>
        <div style="margin-top:12px">
          <div style="font-size:12px;color:var(--muted);margin-bottom:6px">Current age: <span id="tr-age" style="color:var(--green3);font-weight:700;font-size:18px">0.0</span> years</div>
          <div class="progress-bar" style="margin-bottom:4px">
            <div class="progress-fill" id="tr-progress" style="width:0%;background:var(--green2)"></div>
          </div>
        </div>
        <div class="stats-grid" style="margin-top:12px" id="tr-stats">
          ${[
            ['tr-cilia','Track A: Cilia','teal'],
            ['tr-spindle','Track B: Spindle','violet'],
            ['tr-telo','Track C: Telomere','blue'],
            ['tr-epi','Track D: Epi Clock','gold'],
            ['tr-mito','Track E: Mito','red'],
            ['tr-myelo','Myeloid Bias','orange'],
            ['tr-dmg','Total Damage',''],
            ['tr-pot','Potency',''],
          ].map(([id,label,color])=>`
            <div class="stat-box">
              <div class="stat-label">${label}</div>
              <div class="stat-value ${color}" id="${id}">—</div>
            </div>
          `).join('')}
        </div>
      </div>
      <div class="card">
        <div class="card-title">Real-time Track Monitor</div>
        <div class="canvas-wrap" style="height:420px">
          <canvas id="tr-canvas" height="420"></canvas>
        </div>
      </div>
    </div>
  `;

  let cell = new CellState();
  let running = false;
  let animId = null;
  const trackHist = { A:[], B:[], C:[], D:[], E:[], dmg:[], myelo:[] };
  const MAX_HIST = 500;

  function updateDisplay() {
    const s = cell.snapshot();
    document.getElementById('tr-age').textContent = s.age.toFixed(1);
    document.getElementById('tr-progress').style.width = Math.min(100, s.age/120*100) + '%';
    document.getElementById('tr-cilia').textContent = pct(s.ciliary_function);
    document.getElementById('tr-spindle').textContent = pct(s.spindle_fidelity);
    document.getElementById('tr-telo').textContent = pct(s.telomere_length);
    document.getElementById('tr-epi').textContent = fmt(s.methylation_age,1) + ' yrs';
    document.getElementById('tr-mito').textContent = pct(s.mito_shield);
    document.getElementById('tr-myelo').textContent = pct(s.myeloid_bias);
    document.getElementById('tr-dmg').textContent = pct(s.total_damage);
    const potEl = document.getElementById('tr-pot');
    potEl.textContent = s.potency;
    potEl.className = 'stat-value potency ' + potencyClass(s.potency);
    potEl.style.fontSize = '11px';

    trackHist.A.push(s.ciliary_function); if(trackHist.A.length>MAX_HIST) trackHist.A.shift();
    trackHist.B.push(s.spindle_fidelity); if(trackHist.B.length>MAX_HIST) trackHist.B.shift();
    trackHist.C.push(s.telomere_length);  if(trackHist.C.length>MAX_HIST) trackHist.C.shift();
    trackHist.D.push(Math.min(1,s.methylation_age/120)); if(trackHist.D.length>MAX_HIST) trackHist.D.shift();
    trackHist.E.push(s.mito_shield);      if(trackHist.E.length>MAX_HIST) trackHist.E.shift();
    trackHist.dmg.push(s.total_damage);  if(trackHist.dmg.length>MAX_HIST) trackHist.dmg.shift();
    trackHist.myelo.push(s.myeloid_bias);if(trackHist.myelo.length>MAX_HIST) trackHist.myelo.shift();
    drawTrackMonitor(trackHist);

    if (!cell.is_alive) {
      running = false;
      document.getElementById('tr-age').style.color = 'var(--red)';
    }
  }

  function tick() {
    if (!running || !cell.is_alive) { running = false; return; }
    const speed = parseInt(document.getElementById('tr-speed').value);
    for (let i = 0; i < speed && cell.is_alive; i++) cell.step();
    updateDisplay();
    animId = requestAnimationFrame(tick);
  }

  document.getElementById('tr-start').onclick = () => {
    if (cell.is_alive) { running = true; tick(); }
  };
  document.getElementById('tr-pause').onclick = () => { running = false; if(animId) cancelAnimationFrame(animId); };
  document.getElementById('tr-reset').onclick = () => {
    running = false; if(animId) cancelAnimationFrame(animId);
    cell = new CellState();
    Object.values(trackHist).forEach(a=>a.length=0);
    document.getElementById('tr-age').style.color='var(--green3)';
    document.getElementById('tr-age').textContent = '0.0';
    document.getElementById('tr-progress').style.width='0%';
    updateDisplay();
  };

  updateDisplay();
}

function drawTrackMonitor(hist) {
  const canvas = document.getElementById('tr-canvas');
  if (!canvas) return;
  const W = canvas.offsetWidth||500, H=420;
  canvas.width=W; canvas.height=H;
  const ctx=canvas.getContext('2d');
  ctx.fillStyle='#0a1f18'; ctx.fillRect(0,0,W,H);
  const pad={l:8,r:8,t:8,b:8};
  const pw=W-pad.l-pad.r, ph=H-pad.t-pad.b;
  const N=hist.A.length;
  if(N<2)return;

  const tracks=[
    {data:hist.A,color:'#14b8a6',label:'A:Cilia'},
    {data:hist.B,color:'#8b5cf6',label:'B:Spindle'},
    {data:hist.C,color:'#3b82f6',label:'C:Telo'},
    {data:hist.E,color:'#ef4444',label:'E:Mito'},
    {data:hist.dmg,color:'#f87171',label:'Damage',dash:true},
    {data:hist.myelo,color:'#f97316',label:'Myeloid'},
  ];

  // Threshold
  const thy=pad.t+ph*0.25;
  ctx.strokeStyle='rgba(239,68,68,0.3)'; ctx.lineWidth=1; ctx.setLineDash([3,3]);
  ctx.beginPath(); ctx.moveTo(pad.l,thy); ctx.lineTo(pad.l+pw,thy); ctx.stroke();
  ctx.setLineDash([]);

  tracks.forEach(tr=>{
    ctx.strokeStyle=tr.color; ctx.lineWidth=1.8;
    if(tr.dash) ctx.setLineDash([4,4]);
    ctx.beginPath();
    tr.data.forEach((v,i)=>{
      const x=pad.l+(i/(N-1))*pw, y=pad.t+ph*(1-Math.min(1,Math.max(0,v)));
      i===0?ctx.moveTo(x,y):ctx.lineTo(x,y);
    });
    ctx.stroke(); ctx.setLineDash([]);
  });

  // Legend
  tracks.forEach((tr,i)=>{
    const row=Math.floor(i/3), col=i%3;
    const lx=pad.l+col*(pw/3), ly=pad.t+10+row*16;
    ctx.fillStyle=tr.color; ctx.fillRect(lx,ly-7,10,7);
    ctx.fillStyle='#94a3b8'; ctx.font='10px sans-serif'; ctx.fillText(tr.label,lx+13,ly);
  });
}

// ─── Module 4: Interventions ──────────────────────────────────────────────────
export function initInterventions() {
  const presets = Object.keys(InterventionComparison.presets());
  document.getElementById('m-interv').innerHTML = `
    <div class="module-header">
      <div class="module-title">Interventions — Lifespan Comparison</div>
      <div class="module-desc">
        Compare the effect of different anti-aging interventions against the CDATA baseline.
        Each intervention modifies specific damage parameters. Run all simultaneously to see lifespan differences.
      </div>
      <div class="module-equation">healthspan_years = age_at(damage > 0.5) &nbsp;·&nbsp; max_lifespan = age_at_death</div>
    </div>
    <div class="grid-2">
      <div class="card">
        <div class="card-title">Select Scenarios</div>
        <div style="display:flex;flex-direction:column;gap:8px;margin-bottom:14px">
          ${presets.map(n=>`
            <label class="toggle-row">
              <input type="checkbox" class="cdata-toggle interv-check" value="${n}" ${n.includes('Default')||n.includes('Longevity')||n.includes('Senolytics')?'checked':''}>
              ${n}
            </label>
          `).join('')}
        </div>
        <button class="cdata-btn" id="interv-run">▶ Compare All</button>
        <div id="interv-results" style="margin-top:14px"></div>
      </div>
      <div class="card">
        <div class="card-title">Lifespan Comparison</div>
        <div class="canvas-wrap" style="height:380px">
          <canvas id="interv-canvas" height="380"></canvas>
        </div>
      </div>
    </div>
  `;

  document.getElementById('interv-run').onclick = runInterventions;
  runInterventions();
}

function runInterventions() {
  const selected = [...document.querySelectorAll('.interv-check:checked')].map(el=>el.value);
  if (!selected.length) return;
  const results = InterventionComparison.run(selected, 120);

  // Results table
  const tbody = results.map(r=>
    `<tr>
      <td><span style="display:inline-block;width:10px;height:10px;background:${r.color};border-radius:50%;margin-right:4px"></span>${r.name}</td>
      <td style="color:${r.death_age>90?'var(--green3)':r.death_age>70?'var(--gold)':'var(--red)'}">${r.death_age.toFixed(1)}</td>
      <td>${r.snapshots.length > 0 ? pct(r.snapshots[r.snapshots.length-1].total_damage) : '—'}</td>
    </tr>`
  ).join('');
  document.getElementById('interv-results').innerHTML = `
    <table class="cdata-table">
      <thead><tr><th>Scenario</th><th>Death age (yrs)</th><th>Final damage</th></tr></thead>
      <tbody>${tbody}</tbody>
    </table>
  `;

  drawInterventionChart(results);
}

function drawInterventionChart(results) {
  const canvas = document.getElementById('interv-canvas');
  const W = canvas.offsetWidth||600, H=380;
  canvas.width=W; canvas.height=H;
  const ctx=canvas.getContext('2d');
  ctx.fillStyle='#0a1f18'; ctx.fillRect(0,0,W,H);
  const pad={l:40,r:16,t:16,b:50};
  const pw=W-pad.l-pad.r, ph=H-pad.t-pad.b;

  // Grid
  ctx.strokeStyle='#1a3d2e'; ctx.lineWidth=1;
  [0,0.25,0.5,0.75,1].forEach(y=>{
    const py=pad.t+ph*(1-y);
    ctx.beginPath(); ctx.moveTo(pad.l,py); ctx.lineTo(pad.l+pw,py); ctx.stroke();
    ctx.fillStyle='#475569'; ctx.font='10px sans-serif'; ctx.fillText(y.toFixed(2),2,py+4);
  });
  [0,20,40,60,80,100,120].forEach(age=>{
    const x=pad.l+(age/120)*pw;
    ctx.beginPath(); ctx.moveTo(x,pad.t); ctx.lineTo(x,pad.t+ph); ctx.stroke();
    ctx.fillStyle='#475569'; ctx.fillText(age,x-6,H-32);
  });

  // Threshold
  const thy=pad.t+ph*0.25;
  ctx.strokeStyle='rgba(239,68,68,0.4)'; ctx.lineWidth=1; ctx.setLineDash([4,4]);
  ctx.beginPath(); ctx.moveTo(pad.l,thy); ctx.lineTo(pad.l+pw,thy); ctx.stroke(); ctx.setLineDash([]);
  ctx.fillStyle='rgba(239,68,68,0.6)'; ctx.font='9px sans-serif'; ctx.fillText('death threshold', pad.l+4, thy-3);

  results.forEach((r,ri) => {
    ctx.strokeStyle=r.color; ctx.lineWidth=2;
    if(r.dash) ctx.setLineDash([5,4]);
    ctx.beginPath();
    r.snapshots.forEach((s,i)=>{
      const x=pad.l+(s.age/120)*pw, y=pad.t+ph*(1-Math.min(1,s.total_damage));
      i===0?ctx.moveTo(x,y):ctx.lineTo(x,y);
    });
    ctx.stroke(); ctx.setLineDash([]);

    // Death marker
    const last=r.snapshots[r.snapshots.length-1];
    const dx=pad.l+(last.age/120)*pw;
    ctx.beginPath(); ctx.arc(dx, pad.t+ph*(1-Math.min(1,last.total_damage)), 5,0,Math.PI*2);
    ctx.fillStyle=r.color; ctx.fill();

    // Legend
    const lrow=Math.floor(ri/2), lcol=ri%2;
    const lx=pad.l+lcol*220, ly=H-22+lrow*12;
    ctx.fillStyle=r.color; ctx.fillRect(lx,ly-8,12,8);
    ctx.fillStyle='#94a3b8'; ctx.font='10px sans-serif';
    const name=r.name.length>22?r.name.substring(0,22)+'…':r.name;
    ctx.fillText(`${name} (${r.death_age.toFixed(0)}y)`,lx+15,ly);
  });
  ctx.fillStyle='#94a3b8'; ctx.fillText('Age (years) →', pad.l+pw/2-30, H-5);
}

// ─── Module 5: Cell Cycle ────────────────────────────────────────────────────
export function initCellCycle() {
  document.getElementById('m-cycle').innerHTML = `
    <div class="module-header">
      <div class="module-title">Cell Cycle — G1/S/G2/M Progression</div>
      <div class="module-desc">
        Simulate the cell cycle with damage-dependent checkpoint activation.
        p21 overexpression (from DNA damage) causes G1 arrest. p16 overexpression causes permanent senescence.
        The Hayflick limit is enforced via telomere shortening → G1 block.
      </div>
      <div class="module-equation">p21 > 0.7 → G1 arrest &nbsp;·&nbsp; p16 > 0.8 → Senescence &nbsp;·&nbsp; telomere < 0.3 → Hayflick</div>
    </div>
    <div class="grid-2">
      <div class="card">
        <div class="card-title">Damage Parameters</div>
        <div class="controls">
          <div class="control-group">
            <label>DNA damage level</label>
            <input type="range" class="cdata-input" id="cc-dmg" min="0" max="1" value="0.1" step="0.01">
            <span id="cc-dmg-val">0.10</span>
          </div>
          <div class="control-group">
            <label>ROS level</label>
            <input type="range" class="cdata-input" id="cc-ros" min="0" max="1" value="0.1" step="0.01">
            <span id="cc-ros-val">0.10</span>
          </div>
          <button class="cdata-btn" id="cc-step">Step</button>
          <button class="cdata-btn" id="cc-run100">Run 100 steps</button>
          <button class="cdata-btn secondary" id="cc-reset">Reset</button>
        </div>
        <div class="stats-grid" style="margin-top:12px">
          <div class="stat-box highlight"><div class="stat-label">Phase</div><div class="stat-value green" id="cc-phase">G1</div></div>
          <div class="stat-box"><div class="stat-label">Progress</div><div class="stat-value" id="cc-prog">0%</div></div>
          <div class="stat-box"><div class="stat-label">p21</div><div class="stat-value" id="cc-p21">—</div></div>
          <div class="stat-box"><div class="stat-label">p16</div><div class="stat-value" id="cc-p16">—</div></div>
          <div class="stat-box"><div class="stat-label">Divisions</div><div class="stat-value green" id="cc-div">0</div></div>
          <div class="stat-box"><div class="stat-label">Status</div><div class="stat-value" id="cc-status" style="font-size:11px">Active</div></div>
        </div>
        <div class="info-box" style="margin-top:10px">
          <strong>Checkpoints:</strong><br>
          G1/S restriction: cyclin D + Rb phosphorylation<br>
          DNA damage: ATM/ATR → p53 → p21 → CDK2 inhibition → arrest<br>
          Senescence: p16 → Rb hypophosphorylation → permanent exit<br>
          <br><strong>CDATA link:</strong> Spindle fidelity loss → more symmetric divisions → pool exhaustion = Hayflick at the tissue level.
        </div>
      </div>
      <div class="card">
        <div class="card-title">Phase Wheel + Checkpoint Status</div>
        <div class="canvas-wrap" style="height:340px">
          <canvas id="cc-canvas" height="340"></canvas>
        </div>
      </div>
    </div>
  `;

  let sim = new CellCycleSimulator();

  ['cc-dmg','cc-ros'].forEach(id=>{
    document.getElementById(id).oninput=function(){
      document.getElementById(id+'-val').textContent=parseFloat(this.value).toFixed(2);
    };
  });

  function getVals(){
    return {
      damage: parseFloat(document.getElementById('cc-dmg').value),
      ros: parseFloat(document.getElementById('cc-ros').value),
    };
  }

  function updateDisplay(){
    const phases=['G1','S','G2','M'];
    const phaseColors={G1:'#10b981',S:'#14b8a6',G2:'#8b5cf6',M:'#3b82f6'};
    const el=document.getElementById('cc-phase');
    el.textContent=sim.phase; el.style.color=phaseColors[sim.phase]??'white';
    document.getElementById('cc-prog').textContent=pct(sim.progress);
    document.getElementById('cc-p21').textContent=fmt(sim.p21);
    document.getElementById('cc-p21').style.color=sim.p21>0.7?'var(--red)':'var(--text)';
    document.getElementById('cc-p16').textContent=fmt(sim.p16);
    document.getElementById('cc-p16').style.color=sim.p16>0.8?'var(--red)':'var(--text)';
    document.getElementById('cc-div').textContent=sim.division_count;
    const statusEl=document.getElementById('cc-status');
    statusEl.textContent=sim.arrested?(sim.arrest_cause??'Arrested'):'Active';
    statusEl.style.color=sim.arrested?'var(--red)':'var(--green3)';
    drawCellCycle(sim);
  }

  document.getElementById('cc-step').onclick=()=>{
    const {damage,ros}=getVals(); sim.step(damage,ros); updateDisplay();
  };
  document.getElementById('cc-run100').onclick=()=>{
    const {damage,ros}=getVals();
    for(let i=0;i<100;i++) sim.step(damage,ros);
    updateDisplay();
  };
  document.getElementById('cc-reset').onclick=()=>{ sim=new CellCycleSimulator(); updateDisplay(); };
  updateDisplay();
}

function drawCellCycle(sim) {
  const canvas=document.getElementById('cc-canvas');
  const W=canvas.offsetWidth||500, H=340;
  canvas.width=W; canvas.height=H;
  const ctx=canvas.getContext('2d');
  ctx.fillStyle='#0a1f18'; ctx.fillRect(0,0,W,H);

  const cx=W/2, cy=H/2, R=Math.min(W,H)*0.35;
  const phases=[
    {name:'G1',color:'#10b981',start:-Math.PI/2,end:Math.PI/4,label:'G1\n(growth)'},
    {name:'S',color:'#14b8a6',start:Math.PI/4,end:Math.PI,label:'S\n(DNA synthesis)'},
    {name:'G2',color:'#8b5cf6',start:Math.PI,end:1.5*Math.PI,label:'G2\n(prep)'},
    {name:'M',color:'#3b82f6',start:1.5*Math.PI,end:2.5*Math.PI,label:'M\n(mitosis)'},
  ];

  // Draw phase arcs
  phases.forEach(ph=>{
    ctx.beginPath();
    ctx.moveTo(cx,cy);
    ctx.arc(cx,cy,R,ph.start,ph.end);
    ctx.closePath();
    const isActive=ph.name===sim.phase && !sim.arrested;
    ctx.fillStyle=isActive?ph.color+'33':ph.color+'11';
    ctx.fill();
    ctx.strokeStyle=ph.name===sim.phase?ph.color:'#1a3d2e';
    ctx.lineWidth=isActive?3:1;
    ctx.stroke();

    // Label
    const midAngle=(ph.start+ph.end)/2;
    const lx=cx+Math.cos(midAngle)*R*0.65;
    const ly=cy+Math.sin(midAngle)*R*0.65;
    ctx.fillStyle=ph.name===sim.phase?ph.color:'#475569';
    ctx.font=`bold ${ph.name===sim.phase?'14':'11'}px sans-serif`;
    ctx.textAlign='center'; ctx.textBaseline='middle';
    ctx.fillText(ph.name,lx,ly);
  });

  // Progress indicator
  if (!sim.arrested) {
    const phases=['G1','S','G2','M'];
    const phaseAngles={G1:-Math.PI/2,S:Math.PI/4,G2:Math.PI,M:1.5*Math.PI};
    const phaseEnds={G1:Math.PI/4,S:Math.PI,G2:1.5*Math.PI,M:2.5*Math.PI};
    const start=phaseAngles[sim.phase]??-Math.PI/2;
    const end=phaseEnds[sim.phase]??Math.PI/4;
    const progAngle=start+(end-start)*sim.progress;
    const px=cx+Math.cos(progAngle)*R;
    const py=cy+Math.sin(progAngle)*R;
    ctx.beginPath(); ctx.arc(px,py,8,0,Math.PI*2);
    ctx.fillStyle='#d4af37'; ctx.fill();
    ctx.strokeStyle='#fff'; ctx.lineWidth=2; ctx.stroke();
  }

  // Center status
  ctx.textAlign='center'; ctx.textBaseline='middle';
  if (sim.arrested) {
    ctx.fillStyle='rgba(239,68,68,0.15)';
    ctx.beginPath(); ctx.arc(cx,cy,R*0.4,0,Math.PI*2); ctx.fill();
    ctx.fillStyle='#ef4444'; ctx.font='bold 14px sans-serif';
    ctx.fillText('ARRESTED',cx,cy-8);
    ctx.font='10px sans-serif'; ctx.fillStyle='#fca5a5';
    ctx.fillText(sim.arrest_cause??'',cx,cy+10);
  } else {
    ctx.fillStyle='#10b981'; ctx.font='bold 16px sans-serif';
    ctx.fillText(sim.phase,cx,cy-8);
    ctx.fillStyle='#94a3b8'; ctx.font='11px sans-serif';
    ctx.fillText(`div: ${sim.division_count}`,cx,cy+10);
  }

  // Checkpoint indicators
  ctx.textAlign='left'; ctx.textBaseline='alphabetic';
  const checks=[
    {label:'p21',val:sim.p21,threshold:0.7,x:8,y:20},
    {label:'p16',val:sim.p16,threshold:0.8,x:8,y:40},
  ];
  checks.forEach(ch=>{
    const color=ch.val>ch.threshold?'#ef4444':'#10b981';
    ctx.fillStyle=color; ctx.font='11px sans-serif';
    ctx.fillText(`${ch.label}: ${fmt(ch.val,3)} ${ch.val>ch.threshold?'⚠ ABOVE THRESHOLD':'✓'}`,ch.x,ch.y);
    ctx.fillStyle=color+'44';
    ctx.fillRect(8,ch.y+2,Math.min(ch.val,1)*(W-16),6);
    ctx.strokeStyle=color; ctx.lineWidth=1;
    ctx.strokeRect(8,ch.y+2,W-16,6);
    // Threshold line
    ctx.strokeStyle='rgba(239,68,68,0.5)'; ctx.setLineDash([2,2]);
    const tx=8+ch.threshold*(W-16);
    ctx.beginPath(); ctx.moveTo(tx,ch.y+2); ctx.lineTo(tx,ch.y+8); ctx.stroke(); ctx.setLineDash([]);
  });
}
