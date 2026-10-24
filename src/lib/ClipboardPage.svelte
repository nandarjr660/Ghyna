<script lang="ts">
  // Recent copies (kept in memory by Rust, gone after a restart). Click to copy again.
  import { invoke } from "@tauri-apps/api/core";

  export type ClipEntry = { id: number; kind: "text" | "image"; preview: string; width?: number; height?: number };

  let { items }: { items: ClipEntry[] } = $props();
  let copied = $state<number | null>(null);

  async function copy(item: ClipEntry) {
    if (await invoke<boolean>("clipboard_copy", { id: item.id })) {
      copied = item.id;
      setTimeout(() => (copied = null), 1200);
    }
  }
</script>

<div class="page">
  <div class="header">
    <span class="mark" aria-hidden="true">
      <svg viewBox="0 0 24 24"><path d="M8 5H6v15h12V5h-2" /><rect x="9" y="3" width="6" height="4" rx="1.5" /></svg>
    </span>
    <span class="title">Clipboard</span>
    <span class="count">{items.length ? `${items.length} terakhir` : "kosong"}</span>
    <span class="hint">klik untuk salin</span>
    {#if items.length}
      <button class="clear" onclick={() => invoke("clipboard_clear")} aria-label="Bersihkan clipboard">
        <svg viewBox="0 0 24 24"><path d="M6 6l12 12M18 6 6 18" /></svg>
      </button>
    {/if}
  </div>

  {#if items.length === 0}
    <div class="empty">
      <span class="empty-icon" aria-hidden="true">
        <svg viewBox="0 0 24 24"><path d="M8 5H6v15h12V5h-2" /><rect x="9" y="3" width="6" height="4" rx="1.5" /><path d="M9 12h6M9 15h4" /></svg>
      </span>
      <span>Belum ada salinan<small>Teks dan gambar yang kamu copy muncul di sini.</small></span>
    </div>
  {:else}
    <div class="list scroll">
      {#each items as item (item.id)}
        <button class="item" class:done={copied === item.id} onclick={() => copy(item)} title={item.kind === "text" ? item.preview : "Gambar"}>
          <span class="kind" aria-hidden="true">
            {#if item.kind === "image"}
              <svg viewBox="0 0 24 24"><rect x="3" y="4" width="18" height="16" rx="2" /><circle cx="8.5" cy="9" r="1.5" /><path d="m4 18 5-5 4 4 3-3 4 4" /></svg>
            {:else}
              <svg viewBox="0 0 24 24"><path d="M5 7h14M5 12h10M5 17h7" /></svg>
            {/if}
          </span>
          <span class="body">
            {#if item.kind === "image"}
              <img src={item.preview} alt="" />
              <span class="text">Gambar · {item.width} × {item.height}</span>
            {:else}
              <span class="text">{item.preview}</span>
            {/if}
          </span>
          <span class="action" class:ok={copied === item.id}>
            {#if copied === item.id}
              <svg viewBox="0 0 24 24"><path d="m5 12 4 4L19 6" /></svg>
              Disalin
            {:else}
              <svg viewBox="0 0 24 24"><rect x="8" y="8" width="11" height="11" rx="2" /><path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2" /></svg>
            {/if}
          </span>
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
    --accent: #a78bfa;
    --accent-soft: rgba(167, 139, 250, 0.13);
    --accent-line: rgba(167, 139, 250, 0.38);
  }
  .page::before {
    content: "";
    position: absolute;
    inset: 0;
    pointer-events: none;
    background: radial-gradient(150px 90px at 16% 12%, rgba(167, 139, 250, 0.14), transparent 70%);
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
    stroke-width: 1.8;
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
  button {
    border: 0;
    background: none;
    color: #fff;
    font: inherit;
    cursor: pointer;
  }
  .clear {
    display: grid;
    place-items: center;
    width: 16px;
    height: 16px;
    padding: 0;
    border-radius: 4px;
    color: rgba(255, 255, 255, 0.35);
    transition:
      background 0.15s,
      color 0.15s;
  }
  .clear svg {
    width: 9px;
    height: 9px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
  }
  .clear:hover {
    background: rgba(248, 113, 113, 0.14);
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
    border: 0.5px dashed rgba(167, 139, 250, 0.25);
    border-radius: 13px;
    background:
      radial-gradient(180px 70px at 50% 120%, rgba(167, 139, 250, 0.1), transparent 70%),
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

  .list {
    position: relative;
    z-index: 1;
    flex: 1;
    min-height: 0;
    margin-top: 5px;
    padding-bottom: 2px;
    display: flex;
    flex-direction: column;
    gap: 3px;
    overflow-y: auto;
    scrollbar-width: none;
  }
  .item {
    flex: none;
    display: flex;
    align-items: center;
    gap: 7px;
    min-height: 23px;
    padding: 3px 7px 3px 4px;
    border: 0.5px solid transparent;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.04);
    text-align: left;
    transition:
      background 0.18s,
      border-color 0.18s,
      transform 0.18s;
  }
  .item:hover {
    border-color: rgba(167, 139, 250, 0.2);
    background: rgba(167, 139, 250, 0.08);
    transform: translateX(1px);
  }
  .item.done {
    border-color: rgba(52, 211, 153, 0.35);
    background: rgba(52, 211, 153, 0.1);
  }
  .kind {
    flex: none;
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    border-radius: 6px;
    background: var(--accent-soft);
    color: var(--accent);
  }
  .kind svg {
    width: 11px;
    height: 11px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.7;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .body {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 6px;
    overflow: hidden;
  }
  .body img {
    flex: none;
    width: 20px;
    height: 20px;
    border-radius: 4px;
    object-fit: cover;
  }
  .text {
    flex: 1;
    min-width: 0;
    font-size: 10px;
    color: rgba(255, 255, 255, 0.78);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .action {
    flex: none;
    display: flex;
    align-items: center;
    gap: 3px;
    font-size: 8.5px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.32);
    opacity: 0.65;
    transition:
      color 0.18s,
      opacity 0.18s;
  }
  .action svg {
    width: 10px;
    height: 10px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.9;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .item:hover .action {
    opacity: 1;
    color: var(--accent);
  }
  .action.ok {
    color: #34d399;
    opacity: 1;
  }

  button:focus-visible {
    outline: 1.5px solid var(--accent);
    outline-offset: 2px;
  }
  @media (prefers-reduced-motion: reduce) {
    .item,
    .action {
      transition: none;
    }
  }
</style>
