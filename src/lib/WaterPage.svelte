<script lang="ts">
  import type { Water } from "./productivity.svelte";

  let { water }: { water: Water } = $props();

  const s = $derived(water.settings);
  const today = $derived(water.today());
  const pct = $derived(s ? Math.min(100, Math.round((today / s.goalMl) * 100)) : 0);
  const hours = $derived(water.byHour());
  const top = $derived(Math.max(s ? s.glassMl * 2 : 500, ...hours.map((h) => h.ml)));
  const liters = (ml: number) => (ml / 1000).toLocaleString("id-ID", { maximumFractionDigits: 2 });
</script>

{#if s}
  <div class="page">
    <div class="vessel" aria-label={`${pct}% dari target minum hari ini`}>
      <div class="glass">
        <span class="fill" style:height="{pct}%"><i></i></span>
        <span class="tick t25"></span>
        <span class="tick t50"></span>
        <span class="tick t75"></span>
      </div>
      <span class="vessel-label">{pct}%</span>
    </div>

    <div class="summary">
      <div class="eyebrow">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3c3.5 4.2 6 7.4 6 10.5a6 6 0 0 1-12 0C6 10.4 8.5 7.2 12 3z" /></svg>
        Hidrasi
      </div>
      <div class="amount">{liters(today)} <span>L</span></div>
      <div class="meta">
        dari {liters(s.goalMl)} L
        <b>·</b>
        {pct >= 100 ? "target tercapai" : `${pct}% hari ini`}
      </div>
      <div class="bar"><span style:width="{pct}%"></span></div>
      <button onclick={() => water.drink()}>
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3c3.5 4.2 6 7.4 6 10.5a6 6 0 0 1-12 0C6 10.4 8.5 7.2 12 3z" /></svg>
        Minum {s.glassMl} ml
      </button>
    </div>

    <div class="chart" aria-label="Minum per jam hari ini">
      <div class="bars">
        {#each hours as h}
          <i style:height="{(h.ml / top) * 100}%" class:now={h.hour === new Date().getHours()}></i>
        {/each}
      </div>
      <div class="labels">
        <span>{String(hours[0]?.hour ?? 0).padStart(2, "0")}</span>
        <span>{String(hours[Math.floor(hours.length / 2)]?.hour ?? 0).padStart(2, "0")}</span>
        <span>{String(hours.at(-1)?.hour ?? 0).padStart(2, "0")}</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .page {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 8px 42px 18px;
    overflow: hidden;
    --accent: #38bdf8;
    --accent-soft: rgba(56, 189, 248, 0.2);
    --accent-line: rgba(56, 189, 248, 0.45);
  }
  .page::before {
    content: "";
    position: absolute;
    inset: 0;
    pointer-events: none;
    background: radial-gradient(150px 90px at 18% 22%, rgba(56, 189, 248, 0.14), transparent 70%);
  }

  .vessel {
    position: relative;
    z-index: 1;
    flex: none;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }
  .glass {
    position: relative;
    width: 44px;
    height: 78px;
    overflow: hidden;
    border: 0.5px solid rgba(255, 255, 255, 0.2);
    border-radius: 6px 6px 12px 12px;
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.09), rgba(255, 255, 255, 0.02));
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.16),
      inset 0 -10px 18px rgba(56, 189, 248, 0.08);
  }
  .fill {
    position: absolute;
    right: 0;
    bottom: 0;
    left: 0;
    border-radius: 5px 5px 11px 11px;
    background: linear-gradient(180deg, rgba(125, 211, 252, 0.78), rgba(14, 165, 233, 0.55));
    box-shadow: 0 0 14px rgba(56, 189, 248, 0.35);
  }
  .fill i {
    position: absolute;
    top: -4px;
    left: -20%;
    width: 140%;
    height: 8px;
    border-radius: 50%;
    background: rgba(186, 230, 253, 0.7);
    animation: water-wave 3.4s ease-in-out infinite;
  }
  .tick {
    position: absolute;
    right: 4px;
    left: 4px;
    height: 0;
    border-top: 0.5px dashed rgba(255, 255, 255, 0.18);
    z-index: 2;
  }
  .t25 {
    bottom: 25%;
  }
  .t50 {
    bottom: 50%;
  }
  .t75 {
    bottom: 75%;
  }
  .vessel-label {
    font-size: 9px;
    font-weight: 600;
    color: rgba(186, 230, 253, 0.8);
    font-variant-numeric: tabular-nums;
  }

  .summary {
    position: relative;
    z-index: 1;
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
  }
  .eyebrow {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 8px;
    font-weight: 650;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: rgba(186, 230, 253, 0.68);
  }
  .eyebrow svg {
    width: 9px;
    height: 9px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
  }
  .amount {
    margin-top: 1px;
    font-family: "Segoe UI Variable Display", "Segoe UI", sans-serif;
    font-size: 26px;
    font-weight: 700;
    line-height: 1;
    letter-spacing: -0.02em;
    font-variant-numeric: tabular-nums;
    color: #f0f9ff;
  }
  .amount span {
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0;
    color: rgba(255, 255, 255, 0.5);
  }
  .meta {
    margin-top: 3px;
    font-size: 9px;
    color: rgba(255, 255, 255, 0.5);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .meta b {
    margin: 0 2px;
    font-weight: 400;
    color: rgba(255, 255, 255, 0.28);
  }
  .bar {
    width: 100%;
    height: 3px;
    margin: 5px 0 6px;
    overflow: hidden;
    border-radius: 2px;
    background: rgba(56, 189, 248, 0.14);
  }
  .bar span {
    display: block;
    height: 100%;
    border-radius: 2px;
    background: linear-gradient(90deg, #38bdf8, #0ea5e9);
    box-shadow: 0 0 8px rgba(56, 189, 248, 0.45);
  }
  button {
    display: flex;
    align-items: center;
    gap: 4px;
    height: 23px;
    padding: 0 9px;
    border: 0.5px solid var(--accent-line);
    border-radius: 999px;
    background: var(--accent-soft);
    color: #e0f2fe;
    font: inherit;
    font-size: 9.5px;
    font-weight: 600;
    cursor: pointer;
    transition:
      background 0.2s,
      transform 0.18s,
      border-color 0.2s;
  }
  button svg {
    width: 10px;
    height: 10px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.9;
  }
  button:hover {
    border-color: rgba(125, 211, 252, 0.8);
    background: rgba(56, 189, 248, 0.32);
    transform: translateY(-1px);
  }
  button:active {
    transform: scale(0.97);
  }
  button:focus-visible {
    outline: 1.5px solid #7dd3fc;
    outline-offset: 2px;
  }

  .chart {
    position: relative;
    z-index: 1;
    flex: none;
    width: 86px;
    height: 78px;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
  }
  .bars {
    flex: 1;
    display: flex;
    align-items: flex-end;
    gap: 2px;
  }
  .bars i {
    flex: 1;
    min-height: 2px;
    border-radius: 2px 2px 1px 1px;
    background: rgba(56, 189, 248, 0.32);
    transition: background 0.2s;
  }
  .bars i.now {
    background: var(--accent);
    box-shadow: 0 0 7px rgba(56, 189, 248, 0.55);
  }
  .labels {
    display: flex;
    justify-content: space-between;
    margin-top: 4px;
    padding-left: 1px;
    padding-right: 1px;
    font-size: 8px;
    color: rgba(255, 255, 255, 0.38);
    font-variant-numeric: tabular-nums;
  }

  @keyframes water-wave {
    0%,
    100% {
      transform: translateX(-4%) rotate(-1deg);
    }
    50% {
      transform: translateX(4%) rotate(1deg);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .fill i {
      animation: none;
    }
  }
</style>
