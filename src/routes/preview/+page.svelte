<!-- Browser-only preview of the Frost UI. No Tauri calls: safe to open in Chrome. -->
<div class="sheet">
  <header>
    <div class="brand">
      <h1>Ghyna</h1>
      <span>Frost preview</span>
    </div>
    <p>Kaca asli (DWM acrylic) hanya muncul di app Tauri. Halaman ini memakai token, tipografi, dan radius yang sama tanpa blur desktop.</p>
  </header>

  <section>
    <h2>Notch tertutup</h2>
    <div class="row">
      <div class="pill idle">
        <span class="wx">☁<b>24°</b></span>
        <span class="clock">09:41</span>
        <span class="batt">45%</span>
      </div>

      <div class="pill music">
        <span class="disc"></span>
        <span class="track">
          <b>Midnight City</b>
          <small>M83</small>
        </span>
        <span class="bars"><i></i><i></i><i></i><i></i></span>
      </div>

      <div class="pill timer">
        <svg class="ring" viewBox="0 0 20 20" aria-hidden="true">
          <circle cx="10" cy="10" r="7.5"></circle>
          <circle class="fill" cx="10" cy="10" r="7.5"></circle>
        </svg>
        <span class="mono big">25:00</span>
      </div>

      <div class="pill download">
        <span class="mono name">gta-demo.exe</span>
        <span class="track-bar"><i style="width:51%"></i></span>
        <span class="mono pct">51%</span>
      </div>
    </div>
  </section>

  <section>
    <h2>Halaman terbuka</h2>
    <div class="row">
      <article class="page player">
        <div class="art"></div>
        <div class="body">
          <span class="eyebrow">SEDANG DIPUTAR</span>
          <strong class="display">Midnight City</strong>
          <span class="artist">M83 · Hurry Up, We're Dreaming</span>
          <div class="bar"><i style="width:42%"></i></div>
          <div class="meta">
            <span class="mono">1:32</span>
            <span class="mono">3:41</span>
          </div>
          <div class="controls">
            <span class="glyph">◀◀</span>
            <span class="glyph play">▶</span>
            <span class="glyph">▶▶</span>
            <span class="chip icon on" aria-label="Lirik">
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <g fill="none">
                  <path fill="currentColor" fill-rule="evenodd" opacity="0.2" d="M18.32 17.43c.9 0 1.36 0 1.73-.1a3 3 0 0 0 2.15-2.14c.1-.37.1-.82.1-1.73V9.72c0-2.83 0-4.24-.88-5.12s-2.3-.88-5.12-.88H7.7c-2.82 0-4.24 0-5.12.88S1.7 6.9 1.7 9.72v3.74c0 .9 0 1.36.1 1.73a3 3 0 0 0 2.15 2.15c.37.1.82.1 1.73.1h.56a3 3 0 0 1 1.12.34c.12.06.24.14.48.3l.9.58c1.58 1.02 2.37 1.53 3.26 1.53s1.68-.5 3.26-1.53l.9-.59l.48-.3a3 3 0 0 1 1.12-.32c.13-.02.28-.02.56-.02M15.34 7.28a2.14 2.14 0 1 1 0 4.28a2.14 2.14 0 0 1 0-4.28M10.6 9.42a2.14 2.14 0 1 0-4.28 0a2.14 2.14 0 0 0 4.28 0" clip-rule="evenodd" />
                  <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M17.88 17.43c1.32 0 1.98 0 2.5-.2a3 3 0 0 0 1.71-1.71c.2-.53.2-1.18.2-2.5v-3.3c0-2.83 0-4.24-.87-5.12c-.88-.88-2.3-.88-5.12-.88H7.7c-2.82 0-4.24 0-5.12.88S1.7 6.9 1.7 9.72v3.25c0 1.36 0 2.05.22 2.59a3 3 0 0 0 1.66 1.66c.54.21 1.22.21 2.59.21c.63 0 .94 0 1.24.06a3 3 0 0 1 1.16.51c.24.18.46.41.88.87l.15.16c.88.96 1.32 1.44 1.85 1.6a2 2 0 0 0 1.12 0c.53-.14.97-.62 1.87-1.56l.24-.26c.41-.43.62-.65.85-.82a3 3 0 0 1 1.18-.5c.28-.06.58-.06 1.17-.06" />
                  <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M10.6 9.42a2.13 2.13 0 0 0-2.14-2.14a2.14 2.14 0 1 0 2.14 2.14m6.87 0a2.13 2.13 0 0 0-2.14-2.14a2.14 2.14 0 1 0 2.14 2.14" />
                  <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M10.6 9.45c.04.55.15 1.97-.35 2.88c-.59 1.09-1.4 1.69-2.47 1.7m9.69-4.58c.05.55.15 1.97-.34 2.87c-.6 1.08-1.4 1.68-2.48 1.7" />
                </g>
              </svg>
            </span>
          </div>
        </div>
      </article>

      <article class="page">
        <div class="phead">
          <span class="display title accent">Timer Fokus</span>
          <span class="mono tiny">FOKUS 25</span>
        </div>
        <div class="pbody">
          <span class="mono huge">18:24</span>
          <div class="bar"><i style="width:26%"></i></div>
          <div class="meta">
            <span class="mono">Berjalan</span>
            <span class="mono">25 menit</span>
          </div>
        </div>
        <div class="side">
          <button class="btn primary">Jeda</button>
          <button class="btn">Reset</button>
        </div>
      </article>

      <article class="page">
        <div class="phead">
          <span class="display title accent">Air</span>
          <span class="mono tiny">HARI INI</span>
        </div>
        <div class="pbody">
          <span class="mono huge accent">1,25 <small>L</small></span>
          <div class="bar"><i style="width:62%"></i></div>
          <div class="meta">
            <span class="mono">62% hari ini</span>
            <span class="mono">dari 2 L</span>
          </div>
        </div>
        <div class="side">
          <button class="btn primary">+ 250 ml</button>
        </div>
      </article>

      <article class="page">
        <div class="phead">
          <span class="display title accent">Tugas</span>
          <span class="mono tiny">1 / 3</span>
        </div>
        <ul class="tasks">
          <li class="done"><span class="check">✓</span><span>Review desain Frost</span></li>
          <li><span class="check"></span><span>Kirim laporan</span><em class="mono">14:00</em></li>
          <li><span class="check"></span><span>Uji unduhan browser</span></li>
        </ul>
      </article>
    </div>
  </section>

  <section>
    <h2>Peek notifikasi</h2>
    <div class="row">
      <article class="peek">
        <span class="peek-icon">⚡</span>
        <div class="peek-copy">
          <span class="peek-app">GHYNA</span>
          <strong>Mengisi daya</strong>
          <small class="mono">Baterai 43% · USB-C 45 W</small>
        </div>
      </article>

      <article class="peek">
        <span class="peek-icon down">↓</span>
        <div class="peek-copy">
          <span class="peek-app">UNDUHAN</span>
          <strong>gta-demo.exe selesai</strong>
          <small class="mono">176 MB · buka folder</small>
        </div>
      </article>
    </div>
  </section>

  <section>
    <h2>Frost di atas gambar</h2>
    <p class="lead">
      Ini arti kaca yang sebenarnya: warna dan bentuk di belakang tetap terlihat, tapi terblur.
      Di app Tauri blur-nya datang dari acrylic Windows; di sini dari backdrop-filter karena
      Chrome punya isi di belakang elemen.
    </p>
    <div class="froststage">
      <div class="wall" aria-hidden="true"></div>
      <div class="pill demo">
        <span class="eyes"><i></i><i></i></span>
        <span class="clock">09:41</span>
      </div>
    </div>
  </section>
</div>

<style>
  .sheet {
    min-height: 100vh;
    padding: 40px clamp(20px, 5vw, 64px) 72px;
    background:
      radial-gradient(90% 60% at 50% -10%, rgba(143, 230, 180, 0.1), transparent 62%),
      linear-gradient(0deg, #0d1315, var(--surface-solid) 55%);
    color: var(--text);
    font-family: var(--font-ui);
    -webkit-font-smoothing: antialiased;
  }
  header {
    max-width: 880px;
  }
  .brand {
    display: flex;
    align-items: baseline;
    gap: 10px;
  }
  h1 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 30px;
    font-weight: 600;
    letter-spacing: 0.01em;
  }
  .brand span {
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--muted);
  }
  header p {
    margin: 8px 0 0;
    max-width: 62ch;
    font-size: 12.5px;
    line-height: 1.55;
    color: var(--muted-strong);
  }
  section {
    margin-top: 34px;
  }
  h2 {
    margin: 0 0 12px;
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
    gap: 14px;
  }
  .mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
  .display {
    font-family: var(--font-display);
  }

  /* Closed notch pills */
  .pill {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 30px;
    padding: 0 12px;
    border-radius: 15px;
    background:
      linear-gradient(0deg, rgba(255, 255, 255, 0.09), rgba(255, 255, 255, 0) 46%),
      var(--surface);
    box-shadow:
      0 2px 5px rgba(0, 0, 0, 0.32),
      0 18px 40px -20px rgba(0, 0, 0, 0.95);
  }
  .idle {
    width: 150px;
    position: relative;
    justify-content: center;
    gap: 9px;
  }
  .wx {
    position: absolute;
    left: 12px;
    display: flex;
    align-items: center;
    gap: 3px;
    font-size: 10px;
    color: var(--muted);
  }
  .wx b {
    font-size: 11px;
    font-weight: 600;
    color: var(--text);
  }
  .batt {
    position: absolute;
    right: 12px;
    font-size: 11px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    color: var(--muted);
  }
  .eyes {
    display: flex;
    gap: 3px;
  }
  .eyes i {
    width: 4px;
    height: 10px;
    border-radius: 2px;
    background: linear-gradient(to bottom, #e3ffe6, #42c763);
    box-shadow: 0 0 5px rgba(66, 199, 99, 0.5);
  }
  .clock {
    font-family: var(--font-display);
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.02em;
    font-variant-numeric: tabular-nums;
  }
  .music {
    width: 200px;
  }
  .disc {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: conic-gradient(from 210deg, #8fe6b4, #8fd0e8, #1b2326, #8fe6b4);
    box-shadow: inset 0 0 0 0.5px rgba(255, 255, 255, 0.24);
  }
  .track {
    display: flex;
    flex-direction: column;
    min-width: 0;
    line-height: 1.15;
  }
  .track b {
    font-size: 11px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .track small {
    font-size: 8.5px;
    color: var(--muted);
  }
  .bars {
    display: flex;
    align-items: flex-end;
    gap: 2px;
    margin-left: auto;
  }
  .bars i {
    width: 2px;
    border-radius: 1px;
    background: var(--accent);
  }
  .bars i:nth-child(1) { height: 5px; }
  .bars i:nth-child(2) { height: 10px; }
  .bars i:nth-child(3) { height: 7px; }
  .bars i:nth-child(4) { height: 12px; }
  .timer {
    width: 104px;
    justify-content: center;
  }
  .ring {
    width: 16px;
    height: 16px;
    transform: rotate(-90deg);
  }
  .ring circle {
    fill: none;
    stroke: rgba(255, 255, 255, 0.14);
    stroke-width: 2.4;
  }
  .ring .fill {
    stroke: var(--accent);
    stroke-dasharray: 47;
    stroke-dashoffset: 12;
    stroke-linecap: round;
  }
  .big {
    font-size: 12px;
    font-weight: 600;
    letter-spacing: -0.02em;
  }
  .download {
    width: 260px;
  }
  .name {
    font-size: 10.5px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .track-bar {
    flex: 1;
    height: 2px;
    border-radius: 2px;
    background: rgba(255, 255, 255, 0.14);
    overflow: hidden;
  }
  .track-bar i {
    display: block;
    height: 100%;
    background: var(--accent);
  }
  .pct {
    font-size: 10.5px;
    font-weight: 600;
    color: var(--accent);
  }

  /* Open pages */
  .page {
    position: relative;
    display: flex;
    gap: 14px;
    width: 400px;
    height: 132px;
    box-sizing: border-box;
    padding: 14px;
    border-radius: 26px;
    background:
      linear-gradient(0deg, rgba(255, 255, 255, 0.08), rgba(255, 255, 255, 0) 48%),
      var(--surface);
    box-shadow:
      inset 0 -1px 0 var(--edge-lit),
      inset 0 0 0 0.5px rgba(255, 255, 255, 0.05),
      0 2px 5px rgba(0, 0, 0, 0.3),
      0 24px 48px -24px rgba(0, 0, 0, 0.95);
  }
  .player {
    align-items: center;
  }
  .art {
    flex: none;
    width: 64px;
    height: 88px;
    border-radius: var(--radius-md);
    background:
      radial-gradient(80% 60% at 30% 20%, #ffb199, transparent 70%),
      linear-gradient(160deg, #8fe6b4, #74282d 60%, #1b2326);
    box-shadow: 0 0 0 0.5px rgba(255, 255, 255, 0.16), 0 10px 20px -10px #000;
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
    flex: 1;
  }
  .eyebrow {
    font-family: var(--font-mono);
    font-size: 8px;
    font-weight: 600;
    letter-spacing: 0.1em;
    color: var(--muted);
  }
  .body strong {
    font-size: 14px;
    font-weight: 600;
    letter-spacing: 0.01em;
  }
  .artist {
    font-size: 10px;
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .bar {
    height: 2px;
    margin-top: 5px;
    border-radius: 2px;
    background: rgba(255, 255, 255, 0.14);
    overflow: hidden;
  }
  .bar i {
    display: block;
    height: 100%;
    background: var(--accent);
  }
  .meta {
    display: flex;
    justify-content: space-between;
    font-size: 9px;
    color: var(--muted-strong);
  }
  .controls {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 2px;
  }
  .glyph {
    font-size: 9px;
    color: var(--text);
  }
  .glyph.play {
    font-size: 12px;
    color: var(--accent);
  }
  .chip {
    margin-left: auto;
    padding: 3px 7px;
    border-radius: var(--radius-xs);
    background: rgba(255, 255, 255, 0.07);
    box-shadow: inset 0 0 0 0.5px var(--line);
    font-family: var(--font-mono);
    font-size: 8px;
    font-weight: 600;
    letter-spacing: 0.08em;
    color: var(--muted);
  }
  .chip.on {
    background: var(--accent-soft);
    box-shadow: inset 0 0 0 0.5px var(--accent-line);
    color: var(--accent);
  }
  .chip.icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    width: 20px;
    height: 20px;
  }
  .chip.icon svg {
    width: 13px;
    height: 13px;
  }
  .phead {
    position: absolute;
    top: 14px;
    left: 16px;
    right: 16px;
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }
  .title {
    font-size: 14px;
    font-weight: 600;
    letter-spacing: 0.015em;
  }
  .accent { color: var(--accent); }
  .tiny {
    font-size: 8.5px;
    letter-spacing: 0.06em;
    color: var(--muted);
  }
  .pbody {
    display: flex;
    flex-direction: column;
    gap: 3px;
    margin-top: 22px;
    flex: 1;
  }
  .huge {
    font-size: 30px;
    font-weight: 600;
    letter-spacing: -0.04em;
    line-height: 1;
  }
  .huge small {
    font-size: 12px;
    color: var(--muted);
  }
  .side {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 6px;
  }
  .btn {
    padding: 6px 10px;
    border: 0;
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.06);
    box-shadow: inset 0 0 0 0.5px var(--line);
    color: var(--muted-strong);
    font-family: var(--font-mono);
    font-size: 8.5px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    cursor: pointer;
  }
  .btn.primary {
    background: var(--accent);
    color: var(--accent-ink);
    box-shadow: inset 0 0 0 0.5px rgba(255, 255, 255, 0.24);
  }
  .tasks {
    list-style: none;
    margin: 30px 0 0;
    padding: 0;
    width: 100%;
  }
  .tasks li {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 0;
    border-top: 1px solid var(--line);
    font-size: 11px;
    color: rgba(238, 246, 248, 0.88);
  }
  .tasks li:first-child {
    border-top: 0;
  }
  .tasks .check {
    display: grid;
    place-items: center;
    width: 12px;
    height: 12px;
    flex: none;
    border-radius: 3px;
    box-shadow: inset 0 0 0 1.5px rgba(255, 255, 255, 0.28);
    font-size: 8px;
  }
  .tasks .done {
    color: rgba(238, 246, 248, 0.36);
    text-decoration: line-through;
  }
  .tasks .done .check {
    background: var(--accent);
    box-shadow: none;
    color: var(--accent-ink);
    text-decoration: none;
  }
  .tasks em {
    margin-left: auto;
    font-style: normal;
    font-size: 9px;
    color: var(--muted);
  }

  /* Peek */
  .peek {
    display: flex;
    align-items: center;
    gap: 11px;
    width: 280px;
    padding: 11px 13px;
    border-radius: var(--radius-md);
    background:
      linear-gradient(0deg, rgba(255, 255, 255, 0.08), rgba(255, 255, 255, 0) 52%),
      var(--surface-raised);
    box-shadow:
      inset 0 -1px 0 var(--edge-lit),
      inset 0 0 0 0.5px rgba(255, 255, 255, 0.05),
      0 16px 32px -20px #000;
  }
  .peek-icon {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    flex: none;
    border-radius: var(--radius-md);
    background: var(--accent-soft);
    box-shadow: inset 0 0 0 0.5px var(--accent-line);
    color: var(--accent);
    font-size: 14px;
  }
  .peek-icon.down {
    color: var(--info);
    background: rgba(143, 208, 232, 0.12);
    box-shadow: inset 0 0 0 0.5px rgba(143, 208, 232, 0.32);
  }
  .peek-copy {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }
  .peek-app {
    font-family: var(--font-mono);
    font-size: 8px;
    font-weight: 600;
    letter-spacing: 0.1em;
    color: var(--muted);
  }
  .peek-copy strong {
    font-family: var(--font-display);
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.015em;
  }
  .peek-copy small {
    font-size: 9px;
    color: var(--muted);
  }
  .lead {
    margin: -6px 0 10px;
    max-width: 62ch;
    font-size: 11.5px;
    line-height: 1.55;
    color: var(--muted-strong);
  }
  .froststage {
    position: relative;
    height: 150px;
    overflow: hidden;
    border-radius: var(--radius-lg);
    box-shadow: inset 0 0 0 1px var(--line);
  }
  .wall {
    position: absolute;
    inset: 0;
    background:
      radial-gradient(180px 120px at 14% 18%, #ffb27a, transparent 70%),
      radial-gradient(210px 150px at 78% 26%, #6fb6c9, transparent 72%),
      radial-gradient(240px 170px at 52% 96%, #2f6b58, transparent 72%),
      repeating-linear-gradient(115deg, rgba(255, 255, 255, 0.055) 0 2px, transparent 2px 11px),
      linear-gradient(135deg, #1b2a2f, #3a2b26);
  }
  .pill.demo {
    --surface: rgba(10, 14, 16, 0.55);
    position: absolute;
    left: 50%;
    bottom: 22px;
    transform: translateX(-50%);
    justify-content: center;
    width: 150px;
    height: 30px;
    gap: 8px;
    padding: 0 10px;
    background:
      linear-gradient(0deg, rgba(255, 255, 255, 0.1), rgba(255, 255, 255, 0) 48%),
      var(--surface);
    -webkit-backdrop-filter: blur(10px) saturate(150%);
    backdrop-filter: blur(10px) saturate(150%);
  }
</style>
