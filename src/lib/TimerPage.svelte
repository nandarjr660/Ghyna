<script lang="ts">
  import { formatTime } from "./media";
  import type { Pomodoro } from "./productivity.svelte";

  let { pomo, now }: { pomo: Pomodoro; now: number } = $props();

  const left = $derived(pomo.remaining(now));
  const done = $derived(Math.min(1, Math.max(0, pomo.progress(now))));
  const focus = $derived(pomo.phase === "focus");
  const minutes = $derived(focus ? pomo.focusMin : pomo.breakMin);
  const status = $derived(pomo.running ? "Berjalan" : left < pomo.length() - 0.5 ? "Dijeda" : "Siap");
</script>

<div class="page">
  <div class="main">
    <div class="top">
      <span class="time">{formatTime(Math.ceil(left))}</span>
      <span class="mode" class:green={focus}>{focus ? "Fokus" : "Istirahat"}</span>
    </div>
    <div class="progress" aria-hidden="true"><i style:transform="scaleX({done})"></i></div>
    <div class="status"><b>{status}</b><span>·</span>{minutes} menit</div>
  </div>

  <div class="side">
    <div class="phases" aria-label="Fase timer">
      <span class:on={focus}>Fokus <b>{pomo.focusMin}</b></span>
      <span class:on={!focus}>Jeda <b>{pomo.breakMin}</b></span>
    </div>

    <div class="actions">
      <button class="main-button" onclick={() => (pomo.running ? pomo.pause() : pomo.start())}>
        {#if pomo.running}
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M9 5v14M15 5v14" /></svg>
        {:else}
          <svg class="play" viewBox="0 0 24 24" aria-hidden="true"><path d="M8 5l11 7-11 7z" /></svg>
        {/if}
        {pomo.running ? "Jeda" : left < pomo.length() ? "Lanjut" : "Mulai"}
      </button>
      <button class="tool" onclick={() => pomo.skip()} aria-label={focus ? "Mulai istirahat" : "Kembali fokus"} title={focus ? "Mulai istirahat" : "Kembali fokus"}>
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 5l9 7-9 7zM18 5v14" /></svg>
      </button>
      <button class="tool" onclick={() => pomo.reset()} aria-label="Ulang timer" title="Ulang timer">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 10a8 8 0 1 1 1.7 5M4 4v6h6" /></svg>
      </button>
    </div>
  </div>
</div>

<style>
  .page {
    position: absolute;
    inset: 0;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    gap: 24px;
    padding: 9px 42px 18px;
    overflow: hidden;
    font-family: var(--font-ui);
  }
  .main {
    flex: 1;
    min-width: 0;
  }
  .top {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .time {
    font-family: var(--font-mono);
    font-size: 38px;
    font-weight: 600;
    line-height: 1;
    letter-spacing: -0.045em;
    font-variant-numeric: tabular-nums;
  }
  .mode {
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .mode.green {
    color: var(--accent);
  }
  .progress {
    height: 2px;
    margin-top: 10px;
    overflow: hidden;
    border-radius: 1px;
    background: rgba(255, 255, 255, 0.13);
  }
  .progress i {
    display: block;
    width: 100%;
    height: 100%;
    border-radius: inherit;
    transform-origin: left center;
    transition: transform 0.35s linear;
    background: var(--accent);
  }
  .status {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 9px;
    font-family: var(--font-mono);
    font-size: 9.5px;
    letter-spacing: 0.04em;
    color: var(--muted);
  }
  .status b {
    font-weight: 600;
    color: var(--muted-strong);
  }
  .side {
    flex: none;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 11px;
  }
  .phases {
    display: flex;
    gap: 12px;
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .phases span {
    font-variant-numeric: tabular-nums;
  }
  .phases b {
    font-weight: 600;
    color: inherit;
  }
  .phases .on {
    color: var(--text);
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    border: 0;
    font: inherit;
    cursor: pointer;
    -webkit-tap-highlight-color: transparent;
  }
  button svg {
    width: 13px;
    height: 13px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2.1;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .main-button {
    height: 29px;
    padding: 0 12px;
    border-radius: var(--radius-sm);
    background: var(--accent);
    color: var(--accent-ink);
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    box-shadow: inset 0 0 0 0.5px rgba(255, 255, 255, 0.3);
    transition: background var(--motion-ui) var(--ease-out), transform var(--motion-fast) var(--ease-out);
  }
  .main-button .play {
    fill: currentColor;
    stroke: none;
  }
  .main-button:hover {
    background: var(--accent-strong);
  }
  .main-button:active {
    transform: scale(0.97);
  }
  .tool {
    width: 29px;
    height: 29px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.06);
    box-shadow: inset 0 0 0 0.5px var(--line);
    color: var(--muted-strong);
    transition:
      background 0.18s,
      color 0.18s,
      transform 0.15s;
  }
  .tool:hover {
    background: rgba(255, 255, 255, 0.13);
    color: var(--text);
  }
  .tool:active {
    transform: scale(0.95);
  }
  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  @media (prefers-reduced-motion: reduce) {
    .progress i,
    .main-button,
    .tool {
      transition: none;
    }
  }
</style>
