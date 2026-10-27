<script lang="ts">
  // The closed notch while things run (ghyna.md §5F.1, style 1B): the main
  // activity, then up to two more at the right end behind a thin divider.
  import { invoke } from "@tauri-apps/api/core";
  import { untrack } from "svelte";
  import Bars from "./Bars.svelte";
  import { bytes, duration, type Activity } from "./island";
  import { formatTime } from "./media";
  import type { NowPlaying } from "./media";
  import type { Pomodoro } from "./productivity.svelte";

  let {
    main,
    side,
    media,
    line,
    time,
    pomo,
    now,
    levels,
    micMuted,
    active,
    camera = false,
    mic = false,
  }: {
    main: Activity;
    side: Activity[];
    media: NowPlaying | null;
    /** The sung line, or "" (then the clock shows). */
    line: string;
    time: string;
    pomo: Pomodoro;
    now: number;
    levels: number[] | null;
    micMuted: boolean;
    /** On screen: animations may run. */
    active: boolean;
    /** Camera / mic in use (the mic dot is left out during a call, which already says so). */
    camera?: boolean;
    mic?: boolean;
  } = $props();

  const playing = $derived(active && !!media?.playing);
  const timerLeft = $derived(formatTime(Math.ceil(pomo.remaining(now))));

  // With the meter running, the disc turns a step on each level update (5 a second)
  // instead of on its own CSS clock, so bars and disc redraw together.
  let angle = $state(0);
  $effect(() => {
    if (levels) angle = (untrack(() => angle) + 12) % 360;
  });
  const discStyle = $derived(
    (media?.art ? `background-image:url(${media.art});` : "") + (playing && levels ? `transform:rotate(${angle}deg)` : ""),
  );

  // Real percentage when browser history knows the final size; 0 otherwise
  // (the meter then waits without pretending to know).
  const downloadProgress = $derived(
    main.kind === "download" && main.download.total > 0
      ? Math.min(1, main.download.bytes / main.download.total)
      : 0,
  );
  const downloadPercent = $derived(Math.round(downloadProgress * 100));
</script>

{#snippet ring(progress: number, rest: boolean)}
  <svg class="ring" class:rest viewBox="0 0 20 20">
    <circle cx="10" cy="10" r="8" />
    <circle cx="10" cy="10" r="8" class="fill" pathLength="100" stroke-dasharray="{progress * 100} 100" />
  </svg>
{/snippet}

{#snippet phone()}
  <svg class="icon" viewBox="0 0 24 24"><path d="M22 16.9v3a2 2 0 0 1-2.2 2 19.8 19.8 0 0 1-8.6-3.1 19.5 19.5 0 0 1-6-6A19.8 19.8 0 0 1 2.1 4.2 2 2 0 0 1 4.1 2h3a2 2 0 0 1 2 1.7c.1.9.4 1.8.7 2.7a2 2 0 0 1-.5 2.1L8 9.8a16 16 0 0 0 6 6l1.3-1.3a2 2 0 0 1 2.1-.4c.9.3 1.8.6 2.7.7a2 2 0 0 1 1.7 2z" /></svg>
{/snippet}

{#snippet arrow()}
  <svg class="icon" viewBox="0 0 24 24"><path d="M12 3v12M7 10l5 5 5-5M4 20h16" /></svg>
{/snippet}

<div class="island">
  <div class="main {main.kind}">
    {#if main.kind === "music"}
      <div class="disc" class:spin={playing && !levels} style={discStyle}></div>
      {#key line}<span class="line" class:clock-line={!line}>{line || time}</span>{/key}
      <Bars live={playing} {levels} />
    {:else if main.kind === "timer"}
      {@render ring(pomo.progress(now), pomo.phase === "break")}
      <span class="big" class:rest={pomo.phase === "break"}>{timerLeft}</span>
    {:else if main.kind === "call"}
      <span class="badge">{@render phone()}</span>
      <span class="label">{main.call.app}</span>
      <span class="green num">{duration(now - main.call.start)}</span>
      <button
        class="mute"
        class:off={micMuted}
        onclick={() => invoke("toggle_mic")}
        aria-label={micMuted ? "Nyalakan mikrofon" : "Matikan mikrofon"}
        title={micMuted ? "Nyalakan mikrofon" : "Matikan mikrofon"}
      >
        <svg class="icon" viewBox="0 0 24 24">
          {#if micMuted}
            <path d="M9 9v3a3 3 0 0 0 5.1 2.1M15 9.3V6a3 3 0 0 0-5.7-1.3M19 11a7 7 0 0 1-1.2 3.9M5 11a7 7 0 0 0 10.6 6M12 18v3M3 3l18 18" />
          {:else}
            <path d="M12 3a3 3 0 0 0-3 3v6a3 3 0 0 0 6 0V6a3 3 0 0 0-3-3zM19 11a7 7 0 0 1-14 0M12 18v3" />
          {/if}
        </svg>
      </button>
    {:else if main.kind === "download"}
      <span class="dl-arrow">{@render arrow()}</span>
      <span class="dl-text">
        <span class="dl-name">{main.download.name}</span>
        <span class="dl-meta">
          {#if main.download.total > 0}
            {bytes(main.download.bytes)} <span class="dl-of">/</span> {bytes(main.download.total)}
          {:else}
            {bytes(main.download.bytes)}
          {/if}
          {#if main.download.speed > 0}<span class="dl-speed"> · {bytes(main.download.speed)}/s</span>{/if}
        </span>
      </span>
      {#if main.download.total > 0}
        <span
          class="dl-meter"
          title="{bytes(main.download.bytes)} dari {bytes(main.download.total)}"
        >
          <span class="dl-pct">{downloadPercent}%</span>
          <span class="dl-bar" aria-hidden="true"><i style:transform={`scaleX(${downloadProgress})`}></i></span>
        </span>
      {:else}
        <span class="dl-meter waiting" title="Ukuran akhir belum diketahui">
          <span class="dl-pct">…</span>
          <span class="dl-bar" aria-hidden="true"><i></i></span>
        </span>
      {/if}
    {/if}
  </div>

  {#each side as a (a.kind)}
    <i class="sep"></i>
    <div class="side {a.kind}">
      {#if a.kind === "music"}
        <div class="disc small" class:spin={playing && !levels} style={discStyle}></div>
        <Bars live={playing} {levels} />
      {:else if a.kind === "timer"}
        {@render ring(pomo.progress(now), pomo.phase === "break")}
        <span class="num" class:red={pomo.phase !== "break"} class:green={pomo.phase === "break"}>{timerLeft}</span>
      {:else if a.kind === "call"}
        <span class="green">{@render phone()}</span>
        <span class="green num">{duration(now - a.call.start)}</span>
      {:else if a.kind === "download"}
        <span class="blue">{@render arrow()}</span>
        <span class="blue num">{bytes(a.download.speed)}/s</span>
      {/if}
    </div>
  {/each}

  {#if camera || mic}
    <span class="dots">
      {#if camera}<i class="cam"></i>{/if}
      {#if mic}<i class="mic"></i>{/if}
    </span>
  {/if}
</div>

<style>
  .island {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    padding: 0 12px 0 8px;
    gap: 8px;
  }
  .main {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 100%;
  }
  /* Keep the music visualizer anchored at the right edge; lyric length must
     never push it along the row. */
  .main.music .line {
    flex: 1;
    width: auto;
    min-width: 0;
    padding-right: 26px;
    overflow: visible;
    text-overflow: clip;
  }
  .main.music {
    position: relative;
  }
  .main.music :global(.bars) {
    position: absolute;
    right: 0;
    top: 50%;
    flex: none;
    transform: translateY(-50%);
  }
  .main.timer {
    justify-content: center;
    gap: 30px;
  }
  .side {
    flex: none;
    display: flex;
    align-items: center;
    gap: 5px;
  }
  .dots {
    flex: none;
    display: flex;
    gap: 4px;
  }
  .dots i {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }
  .cam {
    background: var(--accent);
    box-shadow: 0 0 6px var(--accent-line);
  }
  .mic {
    background: var(--warning);
    box-shadow: 0 0 6px rgba(240, 182, 110, 0.4);
  }
  .sep {
    flex: none;
    width: 1px;
    height: 16px;
    background: rgba(255, 255, 255, 0.18);
  }
  .line {
    flex: none;
    width: max-content;
    padding: 0 6px;
    text-align: center;
    font-size: 12px;
    font-weight: 600;
    white-space: nowrap;
    overflow: visible;
    text-overflow: clip;
  }
  .clock-line,
  .big {
    font-size: 14px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .label {
    flex: 1;
    min-width: 0;
    font-size: 12.5px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .num {
    font-size: 12px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .side .num {
    font-size: 11.5px;
  }
  .dim {
    color: var(--muted-strong);
    font-weight: 600;
  }
  .green {
    color: var(--accent);
  }
  .red {
    color: var(--danger);
  }
  .blue {
    color: var(--info);
  }
  .icon {
    display: block;
    width: 13px;
    height: 13px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2.2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .badge {
    flex: none;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--accent);
    color: var(--accent-ink);
  }
  .badge .icon {
    width: 11px;
    height: 11px;
  }
  .mute {
    flex: none;
    width: 20px;
    height: 20px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: rgba(239, 250, 243, 0.12);
    color: var(--text);
    cursor: pointer;
  }
  .mute.off {
    background: var(--danger);
  }
  .mute .icon {
    width: 11px;
    height: 11px;
  }

  .ring {
    flex: none;
    width: 18px;
    height: 18px;
    transform: rotate(-90deg);
  }
  .main .ring {
    width: 20px;
    height: 20px;
  }
  .ring circle {
    fill: none;
    stroke: rgba(240, 128, 128, 0.25);
    stroke-width: 2.6;
  }
  .ring .fill {
    stroke: var(--danger);
    stroke-linecap: round;
  }
  .ring.rest circle {
    stroke: var(--accent-soft);
  }
  .ring.rest .fill {
    stroke: var(--accent);
  }
  .big.rest {
    color: var(--text);
  }

  /* The download's right corner: a compact meter, so the row stays balanced
     instead of trailing off after the file name. */
  .main.download {
    gap: 7px;
  }
  .dl-arrow {
    flex: none;
    display: grid;
    place-items: center;
    width: 14px;
    height: 14px;
    color: var(--accent);
  }
  .dl-arrow .icon {
    width: 14px;
    height: 14px;
    stroke-width: 2;
  }
  .dl-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .dl-name {
    font-size: 11.5px;
    font-weight: 600;
    line-height: 1.15;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .dl-meta {
    font-size: 9.5px;
    font-weight: 500;
    line-height: 1.15;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: clip;
  }
  .dl-of {
    padding: 0 1px;
    color: var(--accent-line);
  }
  .dl-speed {
    color: var(--muted-strong);
  }
  .dl-meter {
    flex: none;
    width: 48px;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
  }
  .dl-pct {
    font-size: 10.5px;
    font-weight: 700;
    line-height: 1;
    letter-spacing: -0.01em;
    color: var(--accent);
    font-variant-numeric: tabular-nums;
  }
  .dl-meter.waiting .dl-pct {
    color: var(--muted-strong);
    letter-spacing: 0.06em;
  }
  .dl-bar {
    display: block;
    width: 100%;
    height: 3px;
    border-radius: 999px;
    background: var(--accent-soft);
    overflow: hidden;
  }
  .dl-bar i {
    display: block;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, var(--accent-strong), var(--accent));
    border-radius: inherit;
    transform-origin: left center;
    transition: transform 0.35s cubic-bezier(0.22, 1, 0.36, 1);
  }
  .dl-meter.waiting .dl-bar i {
    transform: none;
    opacity: 0;
  }

  @media (prefers-reduced-motion: reduce) {
    .dl-bar i {
      transition: none;
    }
  }

  /* Cover as a little record: round, with a spindle hole, turning while playing. */
  .disc {
    position: relative;
    flex: none;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--surface-raised) center / cover;
    box-shadow: 0 0 0 0.5px rgba(255, 255, 255, 0.25);
  }
  .disc.small {
    width: 16px;
    height: 16px;
  }
  .disc::after {
    content: "";
    position: absolute;
    inset: 37.5%;
    border-radius: 50%;
    background: var(--surface);
    box-shadow: 0 0 0 1px var(--line);
  }
  /* Stepped in time with the bars (one change every 0.2 s), see Bars.svelte. */
  .disc.spin {
    animation: spin 6s steps(30) infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
