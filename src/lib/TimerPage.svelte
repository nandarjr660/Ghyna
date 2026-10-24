<script lang="ts">
  import { formatTime } from "./media";
  import type { Pomodoro } from "./productivity.svelte";

  let { pomo, now }: { pomo: Pomodoro; now: number } = $props();

  const left = $derived(pomo.remaining(now));
  const done = $derived(Math.min(1, Math.max(0, pomo.progress(now))));
  const focus = $derived(pomo.phase === "focus");
  const minutes = $derived(focus ? pomo.focusMin : pomo.breakMin);
</script>

<div class="page" class:rest={!focus}>
  <div class="dial" class:running={pomo.running}>
    <span class="ticks" aria-hidden="true"></span>
    <svg class="ring" viewBox="0 0 100 100" aria-hidden="true">
      <circle class="track" cx="50" cy="50" r="43" />
      <circle
        class="fill"
        cx="50"
        cy="50"
        r="43"
        pathLength="100"
        stroke-dasharray="{(1 - done) * 100} 100"
      />
    </svg>
    <div class="core">
      <span class="time">{formatTime(Math.ceil(left))}</span>
      <span class="mode">{focus ? "FOKUS" : "JEDA"}</span>
    </div>
  </div>

  <div class="side">
    <div class="status">
      <i></i>
      <span>{focus ? "Sesi fokus" : "Sesi istirahat"}</span>
      <b>{minutes} mnt</b>
    </div>

    <div class="phases" aria-label="Fase timer">
      <div class="phase" class:on={focus}>
        <span>Fokus</span>
        <b>{pomo.focusMin}</b>
      </div>
      <div class="phase" class:on={!focus}>
        <span>Jeda</span>
        <b>{pomo.breakMin}</b>
      </div>
    </div>

    <div class="actions">
      <button class="main" onclick={() => (pomo.running ? pomo.pause() : pomo.start())}>
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
    --accent: #fb7185;
    --accent-soft: rgba(251, 113, 133, 0.13);
    --accent-line: rgba(251, 113, 133, 0.35);
    --ink: #2b0a12;
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 8px 42px 16px;
    overflow: hidden;
  }
  .page::before {
    content: "";
    position: absolute;
    inset: -40% -12% auto;
    height: 150px;
    background: radial-gradient(52% 90% at 18% 0%, rgba(251, 113, 133, 0.16), transparent 72%);
    pointer-events: none;
  }
  .page.rest {
    --accent: #34d399;
    --accent-soft: rgba(52, 211, 153, 0.12);
    --accent-line: rgba(52, 211, 153, 0.32);
    --ink: #04251b;
  }
  .page.rest::before {
    background: radial-gradient(52% 90% at 18% 0%, rgba(52, 211, 153, 0.14), transparent 72%);
  }

  .dial {
    position: relative;
    z-index: 1;
    flex: none;
    width: 94px;
    height: 94px;
    display: grid;
    place-items: center;
  }
  .dial::before {
    content: "";
    position: absolute;
    inset: 14%;
    border-radius: 50%;
    background: var(--accent);
    opacity: 0.18;
    filter: blur(18px);
    transition: opacity 0.4s;
  }
  .dial.running::before {
    opacity: 0.34;
    animation: focus-breathe 4s ease-in-out infinite;
  }
  .ticks {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    background: repeating-conic-gradient(
      from -90deg,
      rgba(255, 255, 255, 0.32) 0deg 0.7deg,
      transparent 0.7deg 6deg
    );
    mask: radial-gradient(circle, transparent 61%, #000 62%, #000 67%, transparent 68%);
    opacity: 0.5;
  }
  .ring {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    transform: rotate(-90deg);
  }
  .track,
  .fill {
    fill: none;
    stroke-width: 5.4;
  }
  .track {
    stroke: rgba(255, 255, 255, 0.09);
  }
  .fill {
    stroke: var(--accent);
    stroke-linecap: round;
    filter: drop-shadow(0 0 5px var(--accent));
    transition: stroke-dasharray 0.35s linear;
  }
  .core {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }
  .time {
    font-family: "Segoe UI Variable Display", "Segoe UI", sans-serif;
    font-size: 28px;
    font-weight: 700;
    line-height: 1;
    letter-spacing: -0.045em;
    font-variant-numeric: tabular-nums;
    color: #fff;
  }
  .mode {
    font-size: 7.5px;
    font-weight: 700;
    letter-spacing: 0.16em;
    color: var(--accent);
  }
  .dial.running .core {
    animation: focus-core 4s ease-in-out infinite;
  }

  .side {
    position: relative;
    z-index: 1;
    flex: 1;
    min-width: 0;
    align-self: stretch;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 7px;
  }
  .status {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 9.5px;
    font-weight: 650;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: rgba(255, 255, 255, 0.58);
  }
  .status i {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--accent);
    box-shadow: 0 0 8px var(--accent);
  }
  .status b {
    margin-left: auto;
    font-size: 9.5px;
    font-weight: 600;
    letter-spacing: 0.04em;
    color: var(--accent);
    text-transform: none;
  }
  .phases {
    display: flex;
    gap: 6px;
  }
  .phase {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 24px;
    padding: 0 8px;
    border: 0.5px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.035);
    font-size: 10px;
    color: rgba(255, 255, 255, 0.48);
    transition:
      background 0.2s,
      border-color 0.2s,
      color 0.2s;
  }
  .phase b {
    font-size: 10px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    color: rgba(255, 255, 255, 0.38);
  }
  .phase.on {
    border-color: var(--accent-line);
    background: var(--accent-soft);
    color: #fff;
  }
  .phase.on b {
    color: var(--accent);
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
    stroke-width: 2.2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .main {
    height: 30px;
    padding: 0 12px;
    border-radius: 9px;
    background: var(--accent);
    color: var(--ink);
    font-size: 10.5px;
    font-weight: 750;
    letter-spacing: 0.01em;
    box-shadow: 0 6px 18px -8px var(--accent);
    transition:
      transform 0.15s,
      filter 0.15s;
  }
  .main .play {
    fill: currentColor;
    stroke: none;
  }
  .main:hover {
    filter: brightness(1.08);
    transform: translateY(-1px);
  }
  .main:active {
    transform: translateY(0) scale(0.97);
  }
  .tool {
    width: 30px;
    height: 30px;
    border: 0.5px solid rgba(255, 255, 255, 0.09);
    border-radius: 9px;
    background: rgba(255, 255, 255, 0.055);
    color: rgba(255, 255, 255, 0.68);
    transition:
      background 0.18s,
      color 0.18s,
      transform 0.15s;
  }
  .tool:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #fff;
  }
  .tool:active {
    transform: scale(0.94);
  }
  button:focus-visible {
    outline: 1.5px solid var(--accent);
    outline-offset: 2px;
  }

  @keyframes focus-breathe {
    0%,
    100% {
      transform: scale(0.9);
      opacity: 0.24;
    }
    50% {
      transform: scale(1.08);
      opacity: 0.4;
    }
  }
  @keyframes focus-core {
    0%,
    100% {
      transform: scale(1);
    }
    50% {
      transform: scale(1.025);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .dial.running::before,
    .dial.running .core {
      animation: none;
    }
  }
</style>
