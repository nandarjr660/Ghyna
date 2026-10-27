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
    gap: 20px;
    padding: 10px 42px 18px;
    overflow: hidden;
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
    font-family: "Segoe UI Variable Display", "Segoe UI", sans-serif;
    font-size: 40px;
    font-weight: 600;
    line-height: 1;
    letter-spacing: -0.045em;
    font-variant-numeric: tabular-nums;
  }
  .mode {
    font-size: 12px;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.55);
  }
  /* Timer focus mode - green accent */
  .mode.green {
    color: var(--accent);
  }
  .progress {
    height: 2.5px;
    margin-top: 12px;
    overflow: hidden;
    border-radius: 1px;
    background: var(--accent-soft);
  }
  .progress i {
    display: block;
    width: 100%;
    height: 100%;
    border-radius: inherit;
    transform-origin: left center;
    transition: transform 0.35s linear;
    background: linear-gradient(90deg, var(--accent-strong), var(--accent));
  }
  .status {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 10px;
    font-size: 11.5px;
    color: rgba(255, 255, 255, 0.4);
  }
  .status b {
    font-weight: 500;
    color: rgba(255, 255, 255, 0.78);
  }
  .side {
    flex: none;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 14px;
  }
  .phases {
    display: flex;
    gap: 14px;
    font-size: 11px;
    color: rgba(255, 255, 255, 0.38);
  }
  .phases span {
    font-variant-numeric: tabular-nums;
  }
  .phases b {
    font-weight: 500;
    color: inherit;
  }
  .phases .on {
    color: #fff;
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
    border-radius: 7px;
    background: #fff;
    color: #000;
    font-size: 10px;
    font-weight: 600;
    transition:
      background 0.18s,
      transform 0.15s;
  }
  .main-button .play {
    fill: currentColor;
    stroke: none;
  }
  .main-button:hover {
    background: rgba(255, 255, 255, 0.86);
  }
  .main-button:active {
    transform: scale(0.97);
  }
  .tool {
    width: 29px;
    height: 29px;
    padding: 0;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 7px;
    background: transparent;
    color: rgba(255, 255, 255, 0.58);
    transition:
      background 0.18s,
      color 0.18s,
      transform 0.15s;
  }
  .tool:hover {
    background: rgba(255, 255, 255, 0.06);
    color: #fff;
  }
  .tool:active {
    transform: scale(0.95);
  }
  button:focus-visible {
    outline: 1.5px solid rgba(255, 255, 255, 0.8);
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
