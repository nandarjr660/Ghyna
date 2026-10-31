<script lang="ts">
  import type { Water } from "./productivity.svelte";

  let { water, enabled, onToggle }: { water: Water; enabled: boolean; onToggle: () => void } = $props();

  const s = $derived(water.settings);
  const today = $derived(water.today());
  const pct = $derived(s ? Math.min(100, Math.round((today / s.goalMl) * 100)) : 0);
  const hours = $derived(water.byHour());
  const top = $derived(Math.max(s ? s.glassMl * 2 : 500, ...hours.map((h) => h.ml)));
  const status = $derived(!enabled ? "Pengingat nonaktif" : pct >= 100 ? "Target tercapai" : `${pct}% hari ini`);
  const liters = (ml: number) => (ml / 1000).toLocaleString("id-ID", { maximumFractionDigits: 2 });
</script>

{#if s}
  <div class="page">
    <div class="header">
      <span class="title">Air</span>
      <span class="count">Hari ini</span>
      <button class="reminder" class:on={enabled} role="switch" aria-checked={enabled} onclick={onToggle} aria-label={enabled ? "Nonaktifkan pengingat minum" : "Aktifkan pengingat minum"}>
        <span>Pengingat</span>
        <i></i>
      </button>
      <button class="drink" onclick={() => water.drink()} aria-label="Minum">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3c3.5 4.2 6 7.4 6 10.5a6 6 0 0 1-12 0C6 10.4 8.5 7.2 12 3z" /></svg>
        Minum {s.glassMl} ml
      </button>
    </div>

    <div class="body">
      <div class="main">
        <div class="amount-row">
          <span class="amount">{liters(today)}</span>
          <span class="unit">L</span>
          <span class="goal">dari {liters(s.goalMl)} L</span>
        </div>
        <div class="progress" aria-hidden="true"><i style:width="{pct}%"></i></div>
        <div class="bottom">
          <span class="status" class:quiet={!enabled}>{status}</span>
        </div>
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
  </div>
{/if}

<style>
  .page {
    position: absolute;
    inset: 0;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    padding: 9px 42px 18px;
    overflow: hidden;
  }
  .header {
    flex: none;
    display: flex;
    align-items: center;
    gap: 10px;
    height: 22px;
  }
  .title {
    font-size: 16px;
    font-weight: 600;
    color: var(--accent);
  }
  .count {
    font-size: 12.5px;
    color: rgba(255, 255, 255, 0.42);
    font-variant-numeric: tabular-nums;
  }
  .reminder {
    margin-left: auto;
    margin-right: 4px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 4px;
    border: 0;
    background: none;
    color: rgba(255, 255, 255, 0.42);
    font: inherit;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: color 0.18s;
  }
  .reminder:hover,
  .reminder.on {
    color: rgba(255, 255, 255, 0.8);
  }
  .reminder i {
    position: relative;
    width: 24px;
    height: 14px;
    border-radius: 7px;
    background: rgba(255, 255, 255, 0.14);
    transition: background 0.2s;
  }
  .reminder i::after {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.55);
    transition:
      transform 0.2s,
      background 0.2s;
  }
  .reminder.on i {
    background: var(--accent);
  }
  .reminder.on i::after {
    background: var(--accent-ink);
    transform: translateX(10px);
  }

  .body {
    flex: 1;
    min-height: 0;
    display: flex;
    align-items: center;
    gap: 18px;
    margin-top: 8px;
  }
  .main {
    flex: 1;
    min-width: 0;
  }
  .amount-row {
    display: flex;
    align-items: baseline;
    gap: 6px;
  }
  .amount {
    font-family: "Segoe UI Variable Display", "Segoe UI", sans-serif;
    font-size: 35px;
    font-weight: 600;
    line-height: 1;
    letter-spacing: -0.04em;
    font-variant-numeric: tabular-nums;
  }
  .unit {
    font-size: 13px;
    color: rgba(255, 255, 255, 0.48);
  }
  .goal {
    margin-left: 6px;
    font-size: 11.5px;
    color: rgba(255, 255, 255, 0.38);
    white-space: nowrap;
  }
  .progress {
    height: 2.5px;
    margin-top: 10px;
    overflow: hidden;
    border-radius: 1px;
    background: rgba(255, 255, 255, 0.12);
  }
  .progress i {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: var(--accent);
  }
  .bars i.now {
    background: var(--accent);
  }
  .amount {
    color: var(--accent);
  }
  .bottom {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 8px;
  }
  .status {
    flex: 1;
    min-width: 0;
    font-size: 11.5px;
    color: rgba(255, 255, 255, 0.38);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .status.quiet {
    color: rgba(255, 255, 255, 0.62);
  }
  button {
    font: inherit;
    cursor: pointer;
  }
  .drink {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 28px;
    padding: 0 14px;
    border: 0;
    border-radius: 8px;
    background: var(--accent);
    color: var(--accent-ink);
    font-size: 11px;
    font-weight: 600;
    white-space: nowrap;
    transition:
      background 0.18s,
      transform 0.15s;
  }
  .drink svg {
    width: 11px;
    height: 11px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
  }
  .drink:hover {
    background: var(--accent-strong);
  }
  .drink:active {
    transform: scale(0.97);
  }
  button:focus-visible {
    outline: 1.5px solid var(--accent);
    outline-offset: 2px;
  }

  .chart {
    flex: none;
    width: 74px;
    height: 56px;
    display: flex;
    flex-direction: column;
  }
  .bars {
    flex: 1;
    display: flex;
    align-items: flex-end;
    gap: 2px;
  }
  .bars i {
    flex: 1;
    min-height: 1px;
    border-radius: 1px;
    background: rgba(255, 255, 255, 0.16);
  }
  .bars i.now {
    background: rgba(255, 255, 255, 0.72);
  }
  .bars i {
    transform-origin: bottom center;
    animation: water-bar 2.8s ease-in-out infinite alternate;
  }
  .bars i:nth-child(3n) { animation-delay: -0.8s; }
  .bars i:nth-child(4n) { animation-delay: -1.6s; }
  .labels {
    display: flex;
    justify-content: space-between;
    margin-top: 5px;
    font-size: 10px;
    color: rgba(255, 255, 255, 0.32);
    font-variant-numeric: tabular-nums;
  }

  @media (prefers-reduced-motion: reduce) {
    .reminder,
    .reminder i,
    .reminder i::after,
    .drink {
      transition: none;
    }
    .bars i { animation: none; }
  }
  @keyframes water-bar { from { transform: scaleY(.96); } to { transform: scaleY(1); } }
</style>
