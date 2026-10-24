<script lang="ts">
  // Buttons for quick actions; the list is set in Settings.
  import { invoke } from "@tauri-apps/api/core";

  export type Shortcut = { name: string; action: string; target: string };

  // onCamera: the "camera" action opens the notch's own camera check.
  let { shortcuts, onCamera }: { shortcuts: Shortcut[]; onCamera: () => void } = $props();
  let failed = $state<number | null>(null);

  const ICON: Record<string, string> = {
    lock: "M6 11h12v10H6zM8 11V7a4 4 0 0 1 8 0v4",
    sleep: "M20 14A8 8 0 1 1 10 4a6 6 0 0 0 10 10z",
    screenshot: "M4 8V5h3M17 5h3v3M20 16v3h-3M7 19H4v-3",
    downloads: "M3 7h6l2 2h10v10H3z",
    camera: "M15 10l5-3v10l-5-3zM3 7h12v10H3z",
    open: "M14 4h6v6M20 4l-9 9M18 14v6H4V6h6",
  };

  const ACCENT: Record<string, string> = {
    lock: "#60a5fa",
    sleep: "#a78bfa",
    screenshot: "#22d3ee",
    downloads: "#fbbf24",
    camera: "#fb7185",
    open: "#34d399",
  };

  async function run(i: number) {
    if (shortcuts[i]?.action === "camera") return onCamera();
    try {
      await invoke("run_shortcut", { index: i });
    } catch {
      failed = i;
      setTimeout(() => (failed = null), 1500);
    }
  }
</script>

<div class="page">
  <div class="header">
    <span class="mark" aria-hidden="true">
      <svg viewBox="0 0 24 24"><rect x="3" y="6" width="18" height="12" rx="2" /><path d="M7 10h2M11 10h2M15 10h2M7 14h10" /></svg>
    </span>
    <span class="title">Pintasan</span>
    <span class="count">{shortcuts.length} aksi</span>
    <span class="hint">atur di Settings</span>
  </div>

  {#if shortcuts.length === 0}
    <div class="empty">
      <span class="empty-icon" aria-hidden="true">
        <svg viewBox="0 0 24 24"><rect x="3" y="6" width="18" height="12" rx="2" /><path d="M7 10h2M11 10h2M15 10h2M7 14h10" /></svg>
      </span>
      <span>Belum ada pintasan<small>Tambahkan aksi cepat di Settings.</small></span>
    </div>
  {:else}
    <!-- One row for up to five, two rows of four beyond that. -->
    <div class="grid" style:--cols={shortcuts.length <= 5 ? Math.max(shortcuts.length, 1) : 4}>
      {#each shortcuts.slice(0, 8) as s, i}
        <button class="sc" class:failed={failed === i} style:--a={ACCENT[s.action] ?? ACCENT.open} onclick={() => run(i)} title={s.target || s.name}>
          <span class="icon" aria-hidden="true">
            <svg viewBox="0 0 24 24"><path d={ICON[s.action] ?? ICON.open} /></svg>
          </span>
          <span class="label">{failed === i ? "Gagal" : s.name}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .page {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    padding: 8px 42px 18px;
    overflow: hidden;
    --accent: #f472b6;
    --accent-soft: rgba(244, 114, 182, 0.13);
    --accent-line: rgba(244, 114, 182, 0.38);
  }
  .page::before {
    content: "";
    position: absolute;
    inset: 0;
    pointer-events: none;
    background: radial-gradient(150px 90px at 16% 12%, rgba(244, 114, 182, 0.13), transparent 70%);
  }

  .header {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: 6px;
    height: 20px;
  }
  .mark {
    display: grid;
    place-items: center;
    width: 18px;
    height: 18px;
    border: 0.5px solid var(--accent-line);
    border-radius: 6px;
    background: var(--accent-soft);
    color: var(--accent);
  }
  .mark svg {
    width: 11px;
    height: 11px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.7;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .title {
    font-size: 12.5px;
    font-weight: 650;
  }
  .count {
    font-size: 8.5px;
    color: rgba(255, 255, 255, 0.4);
    font-variant-numeric: tabular-nums;
  }
  .hint {
    flex: 1;
    font-size: 8.5px;
    color: rgba(255, 255, 255, 0.3);
    text-align: right;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .grid {
    position: relative;
    z-index: 1;
    flex: 1;
    min-height: 0;
    margin-top: 5px;
    padding-bottom: 2px;
    display: grid;
    grid-template-columns: repeat(var(--cols, 4), minmax(0, 1fr));
    grid-auto-rows: minmax(0, 1fr);
    gap: 5px;
  }
  .sc {
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 3px;
    padding: 3px;
    border: 0.5px solid rgba(255, 255, 255, 0.1);
    border-radius: 10px;
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.07), rgba(255, 255, 255, 0.025));
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.08),
      0 2px 6px rgba(0, 0, 0, 0.22);
    color: #fff;
    font: inherit;
    cursor: pointer;
    transition:
      background 0.18s,
      border-color 0.18s,
      transform 0.14s,
      box-shadow 0.18s;
  }
  .sc:hover {
    border-color: color-mix(in srgb, var(--a) 40%, transparent);
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--a) 13%, transparent), rgba(255, 255, 255, 0.02));
    transform: translateY(-1px);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.1),
      0 3px 9px rgba(0, 0, 0, 0.28);
  }
  .sc:active {
    transform: translateY(1px) scale(0.97);
  }
  .icon {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border: 0.5px solid color-mix(in srgb, var(--a) 30%, transparent);
    border-radius: 7px;
    background: color-mix(in srgb, var(--a) 14%, transparent);
    color: var(--a);
  }
  .icon svg {
    width: 12px;
    height: 12px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .label {
    max-width: 100%;
    font-size: 9px;
    font-weight: 550;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sc.failed {
    border-color: rgba(248, 113, 113, 0.55);
    background: rgba(248, 113, 113, 0.13);
    color: #fca5a5;
    animation: shake 0.32s ease;
  }
  .sc.failed .icon {
    border-color: rgba(248, 113, 113, 0.4);
    background: rgba(248, 113, 113, 0.15);
    color: #f87171;
  }

  .empty {
    position: relative;
    z-index: 1;
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 9px;
    margin-top: 5px;
    border: 0.5px dashed rgba(244, 114, 182, 0.25);
    border-radius: 13px;
    background:
      radial-gradient(180px 70px at 50% 120%, rgba(244, 114, 182, 0.1), transparent 70%),
      rgba(255, 255, 255, 0.018);
    color: rgba(255, 255, 255, 0.72);
    font-size: 10.5px;
  }
  .empty-icon {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    border: 0.5px solid var(--accent-line);
    border-radius: 9px;
    background: var(--accent-soft);
    color: var(--accent);
  }
  .empty-icon svg {
    width: 16px;
    height: 16px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.7;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .empty small {
    display: block;
    margin-top: 1px;
    font-size: 8.5px;
    color: rgba(255, 255, 255, 0.38);
  }

  .sc:focus-visible {
    outline: 1.5px solid var(--a);
    outline-offset: 2px;
  }
  @keyframes shake {
    0%,
    100% {
      transform: translateX(0);
    }
    25% {
      transform: translateX(-2px);
    }
    50% {
      transform: translateX(2px);
    }
    75% {
      transform: translateX(-1px);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .sc {
      transition: none;
    }
    .sc.failed {
      animation: none;
    }
  }
</style>
