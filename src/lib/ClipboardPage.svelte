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
    <span class="title">Clipboard</span>
    <span class="count">{items.length ? `${items.length} terakhir` : "kosong"}</span>
    <span class="hint">klik untuk salin</span>
    {#if items.length}
      <button class="clear" onclick={() => invoke("clipboard_clear")} aria-label="Hapus semua dari Clipboard" title="Hapus semua">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 7h16M10 4h4l1 3H9l1-3Z" /><path d="m7 7 1 13h8l1-13M10 10v7M14 10v7" /></svg>
      </button>
    {/if}
  </div>

  {#if items.length === 0}
    <div class="empty">
      <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M8 5H6v15h12V5h-2" /><rect x="9" y="3" width="6" height="4" rx="1.5" /><path d="M9 12h6M9 15h4" /></svg>
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
              <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m5 12 4 4L19 6" /></svg>
              Tersalin
            {:else}
              <svg viewBox="0 0 24 24" aria-hidden="true"><rect x="8" y="8" width="11" height="11" rx="2" /><path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2" /></svg>
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
    gap: 8px;
    height: 18px;
  }
  .title {
    font-size: 12px;
    font-weight: 600;
    color: var(--accent);
  }
  .count {
    font-size: 9.5px;
    color: rgba(255, 255, 255, 0.38);
    font-variant-numeric: tabular-nums;
  }
  .hint {
    flex: 1;
    font-size: 8.5px;
    color: rgba(255, 255, 255, 0.28);
    text-align: right;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  button {
    border: 0;
    background: none;
    color: var(--text);
    font: inherit;
    cursor: pointer;
  }
  .clear {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border-radius: 6px;
    color: rgba(255, 255, 255, 0.3);
    transition:
      background 0.15s,
      color 0.15s;
  }
  .clear svg {
    width: 15px;
    height: 15px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.7;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .clear:hover {
    background: rgba(255, 255, 255, 0.07);
    color: #fff;
  }
  .clear:active,
  .item:active,
  .action:active {
    transform: scale(0.97);
  }

  .empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 9px;
    margin-top: 7px;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
    font-size: 10.5px;
    color: rgba(255, 255, 255, 0.66);
  }
  .empty > svg {
    width: 18px;
    height: 18px;
    fill: none;
    stroke: rgba(255, 255, 255, 0.38);
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .empty small {
    display: block;
    margin-top: 1px;
    font-size: 8.5px;
    color: rgba(255, 255, 255, 0.3);
  }

  .list {
    flex: 1;
    min-height: 0;
    margin-top: 6px;
    padding-bottom: 2px;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    scrollbar-width: none;
  }
  .item {
    flex: none;
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    min-height: 24px;
    padding: 0 2px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.055);
    text-align: left;
    transition: background 0.15s;
  }
  .item:hover {
    background: rgba(255, 255, 255, 0.03);
  }
  .kind {
    flex: none;
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    border-radius: 5px;
    background: rgba(255, 255, 255, 0.06);
    color: rgba(255, 255, 255, 0.4);
  }
  .kind svg {
    width: 14px;
    height: 14px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.6;
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
    width: 22px;
    height: 22px;
    border-radius: 3px;
    object-fit: cover;
  }
  .text {
    flex: 1;
    min-width: 0;
    font-size: 10px;
    color: rgba(255, 255, 255, 0.74);
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
    color: rgba(255, 255, 255, 0.28);
    opacity: 0.6;
    transition:
      color 0.15s,
      opacity 0.15s;
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
    color: rgba(255, 255, 255, 0.7);
  }
  .action.ok {
    color: var(--accent);
    opacity: 1;
  }

  button:focus-visible {
    outline: 1.5px solid var(--accent);
    outline-offset: 2px;
  }
  @media (prefers-reduced-motion: reduce) {
    .clear,
    .item,
    .action {
      transition: none;
      transform: none !important;
    }
  }
</style>
