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
    <span class="title">Pintasan</span>
    <span class="count">{shortcuts.length} aksi</span>
    <span class="hint">atur di Settings</span>
  </div>

  {#if shortcuts.length === 0}
    <div class="empty">
      <svg viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="6" width="18" height="12" rx="2" /><path d="M7 10h2M11 10h2M15 10h2M7 14h10" /></svg>
      <span>Belum ada pintasan<small>Tambahkan aksi cepat di Settings.</small></span>
    </div>
  {:else}
    <!-- Icons only, no labels - simpler monochrome -->
    <div class="grid" style:--cols={shortcuts.length <= 5 ? Math.max(shortcuts.length, 1) : 4}>
      {#each shortcuts.slice(0, 8) as s, i}
        <button class="sc" class:failed={failed === i} onclick={() => run(i)} title={s.target || s.name}>
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d={ICON[s.action] ?? ICON.open} /></svg>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .page {
    position: absolute;
    inset: 0;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    padding: 10px 42px 19px;
    overflow: hidden;
  }
  .header {
    flex: none;
    display: flex;
    align-items: center;
    gap: 10px;
    height: 24px;
  }
  .title {
    font-size: 15px;
    font-weight: 600;
  }
  .count {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.42);
    font-variant-numeric: tabular-nums;
  }
  .hint {
    flex: 1;
    font-size: 11px;
    color: rgba(255, 255, 255, 0.32);
    text-align: right;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .grid {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: repeat(var(--cols, 4), minmax(0, 1fr));
    grid-auto-rows: minmax(0, 1fr);
    gap: 14px;
    margin-top: 15px;
    padding-bottom: 6px;
  }
  .sc {
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 14px;
    padding: 20px 12px;
    border: 1.5px solid rgba(255, 255, 255, 0.14);
    border-radius: 14px;
    background: rgba(255, 255, 255, 0.015);
    color: rgba(255, 255, 255, 0.85);
    font: inherit;
    cursor: pointer;
    transition:
      border-color 0.18s,
      background 0.18s,
      color 0.18s;
  }
  .sc:hover {
    border-color: rgba(255, 255, 255, 0.4);
    background: rgba(255, 255, 255, 0.08);
    color: #fff;
  }
  .sc:active {
    background: rgba(255, 255, 255, 0.09);
  }
  .sc svg {
    flex: none;
    width: 28px;
    height: 28px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
    opacity: 0.8;
  }
  .sc:hover svg {
    opacity: 1;
  }
  .sc.failed {
    border-color: rgba(248, 113, 113, 0.5);
    color: #fca5a5;
  }
  .sc.failed svg {
    opacity: 1;
  }

  .empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    margin-top: 10px;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
    font-size: 12.5px;
    color: rgba(255, 255, 255, 0.68);
  }
  .empty > svg {
    width: 22px;
    height: 22px;
    fill: none;
    stroke: rgba(255, 255, 255, 0.38);
    stroke-width: 1.6;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .empty small {
    display: block;
    margin-top: 2px;
    font-size: 10.5px;
    color: rgba(255, 255, 255, 0.34);
  }

  .sc:focus-visible {
    outline: 1.5px solid rgba(255, 255, 255, 0.8);
    outline-offset: 2px;
  }
  @media (prefers-reduced-motion: reduce) {
    .sc {
      transition: none;
    }
  }
</style>
