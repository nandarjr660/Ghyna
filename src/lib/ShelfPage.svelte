<script lang="ts">
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";
  import { startDrag } from "@crabnebula/tauri-plugin-drag";
  import type { Shelf, ShelfItem } from "./productivity.svelte";

  // dropTarget: which zone the dragged file is over ("shelf" or "share").
  let { shelf, dropping, dropTarget = null }: { shelf: Shelf; dropping: boolean; dropTarget?: string | null } = $props();

  const IMAGE = /\.(png|jpe?g|gif|webp|bmp)$/i;
  const ext = (name: string) => (name.includes(".") ? name.split(".").pop()!.slice(0, 4).toUpperCase() : "");

  /** A small file badge to show under the cursor while dragging out. */
  function dragIcon(item: ShelfItem): string {
    const c = document.createElement("canvas");
    c.width = c.height = 64;
    const g = c.getContext("2d")!;
    g.fillStyle = "#161616";
    g.beginPath();
    g.roundRect(8, 4, 48, 56, 8);
    g.fill();
    g.strokeStyle = "rgba(255, 255, 255, 0.18)";
    g.lineWidth = 1;
    g.stroke();
    g.fillStyle = "#f2f2f2";
    g.font = "600 11px Segoe UI";
    g.textAlign = "center";
    g.fillText(item.dir ? "DIR" : ext(item.name) || "FILE", 32, 37);
    return c.toDataURL("image/png");
  }

  // Press and move: drag the file out (to a chat, an email, a folder). Press and release: open it.
  let pressed: { item: ShelfItem; x: number; y: number } | null = null;
  function down(e: PointerEvent, item: ShelfItem) {
    if (e.button === 0) pressed = { item, x: e.clientX, y: e.clientY };
  }
  function move(e: PointerEvent) {
    if (!pressed || Math.hypot(e.clientX - pressed.x, e.clientY - pressed.y) < 6) return;
    const { item } = pressed;
    pressed = null;
    startDrag({ item: [item.path], icon: dragIcon(item) });
  }
  function up() {
    if (pressed) invoke("open_path", { path: pressed.item.path });
    pressed = null;
  }

  const size = (bytes: number) =>
    bytes >= 1e6 ? `${(bytes / 1e6).toFixed(1)} MB` : bytes >= 1e3 ? `${Math.round(bytes / 1e3)} KB` : `${bytes} B`;
</script>

<svelte:window onpointermove={move} onpointerup={() => (pressed = null)} />

<div class="page" class:dropping>
  <div class="header">
    <span class="title">Shelf</span>
    <span class="count">{shelf.items.length}</span>
    <span class="hint">tarik keluar · klik buka</span>
    {#if shelf.items.length}
      <button class="clear" onclick={() => shelf.clear()} aria-label="Bersihkan shelf">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6 6 18" /></svg>
      </button>
    {/if}
  </div>

  {#if dropping}
    <!-- Two places to drop: keep it on the shelf, or hand it to Windows' Share window. -->
    <div class="zones">
      <div class="zone" class:hot={dropTarget !== "share"} data-drop="shelf">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 8h16v11H4zM4 8l2-3h5l2 3M12 12v4M10 14h4" /></svg>
        <span><strong>Simpan</strong><small>di Shelf</small></span>
      </div>
      <div class="zone" class:hot={dropTarget === "share"} data-drop="share">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 12v7h16v-7M12 3v12M8 7l4-4 4 4" /></svg>
        <span><strong>Bagikan</strong><small>via Windows</small></span>
      </div>
    </div>
  {:else if shelf.items.length === 0}
    <div class="empty-zone" data-drop="shelf">
      <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 8h16v11H4zM4 8l2-3h5l2 3" /><path d="M9 13h6" /></svg>
      <span class="empty-copy">Tarik file ke sini<small>Simpan sementara, lalu tarik keluar kapan saja.</small></span>
    </div>
  {:else}
    <div class="items scroll">
      {#each shelf.items as item (item.path)}
        <div class="item" role="button" tabindex="-1" title={item.name}
          onpointerdown={(e) => down(e, item)} onpointerup={up}>
          <span class="thumb">
            {#if IMAGE.test(item.name)}
              <img src={convertFileSrc(item.path)} alt="" draggable="false" />
            {:else if item.dir}
              <svg viewBox="0 0 24 24" class="folder" aria-hidden="true"><path d="M3 7h7l2 2h9v11H3z" /></svg>
            {:else}
              <span class="ext">{ext(item.name) || "FILE"}</span>
            {/if}
          </span>
          <span class="name">{item.name}</span>
          <span class="size">{item.dir ? "Folder" : size(item.size)}</span>
          <button class="remove" onpointerdown={(e) => e.stopPropagation()} onclick={() => shelf.remove(item.path)} aria-label="Hapus dari shelf">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6 6 18" /></svg>
          </button>
        </div>
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
    height: 22px;
  }
  .title {
    font-size: 14px;
    font-weight: 600;
    color: #a855f7;
  }
  .count {
    font-size: 11.5px;
    color: rgba(255, 255, 255, 0.38);
    font-variant-numeric: tabular-nums;
  }
  .hint {
    flex: 1;
    font-size: 10.5px;
    color: rgba(255, 255, 255, 0.28);
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
    color: rgba(255, 255, 255, 0.3);
    transition:
      background 0.15s,
      color 0.15s;
  }
  .clear svg {
    width: 8px;
    height: 8px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
  }
  .clear:hover {
    background: rgba(255, 255, 255, 0.07);
    color: #fff;
  }

  .empty-zone {
    flex: 1;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    margin-top: 7px;
    border: 1px dashed rgba(255, 255, 255, 0.16);
    border-radius: 10px;
    color: rgba(255, 255, 255, 0.5);
  }
  .empty-zone > svg {
    width: 22px;
    height: 22px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .empty-copy {
    font-size: 13px;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.66);
  }
  .empty-copy small {
    display: block;
    margin-top: 2px;
    font-size: 11px;
    color: rgba(255, 255, 255, 0.3);
  }

  .zones {
    flex: 1;
    min-height: 0;
    display: flex;
    gap: 8px;
    margin-top: 7px;
  }
  .zone {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    border: 1px dashed rgba(255, 255, 255, 0.14);
    border-radius: 10px;
    color: rgba(255, 255, 255, 0.45);
    transition:
      border-color 0.18s,
      background 0.18s,
      color 0.18s;
  }
  .zone.hot {
    border-color: rgba(255, 255, 255, 0.55);
    background: rgba(255, 255, 255, 0.05);
    color: #fff;
  }
  .zone > svg {
    width: 20px;
    height: 20px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.6;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .zone strong {
    display: block;
    font-size: 12px;
    font-weight: 500;
  }
  .zone small {
    display: block;
    font-size: 10px;
    color: rgba(255, 255, 255, 0.32);
  }

  .items {
    flex: 1;
    min-height: 0;
    display: flex;
    gap: 4px;
    margin-top: 7px;
    padding-bottom: 2px;
    overflow-x: auto;
    scrollbar-width: none;
  }
  .item {
    position: relative;
    flex: none;
    width: 66px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3px;
    padding: 5px 5px 4px;
    border: 1px solid transparent;
    border-radius: 8px;
    cursor: grab;
    transition:
      background 0.15s,
      border-color 0.15s;
  }
  .item:hover {
    border-color: rgba(255, 255, 255, 0.15);
    background: rgba(255, 255, 255, 0.05);
  }
  .item:active {
    cursor: grabbing;
  }
  .thumb {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    overflow: hidden;
    border-radius: 6px;
    background: #121212;
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.08);
  }
  .thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .folder {
    width: 20px;
    height: 20px;
    fill: none;
    stroke: rgba(255, 255, 255, 0.6);
    stroke-width: 1.6;
    stroke-linejoin: round;
  }
  .ext {
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.03em;
    color: rgba(255, 255, 255, 0.68);
  }
  .name {
    width: 100%;
    margin-top: 2px;
    text-align: center;
    font-size: 10px;
    color: rgba(255, 255, 255, 0.9);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .size {
    font-size: 9px;
    color: rgba(255, 255, 255, 0.38);
    font-variant-numeric: tabular-nums;
  }
  .remove {
    position: absolute;
    top: 2px;
    right: 2px;
    display: grid;
    place-items: center;
    width: 16px;
    height: 16px;
    padding: 0;
    border-radius: 5px;
    background: rgba(0, 0, 0, 0.6);
    color: rgba(255, 255, 255, 0.6);
    opacity: 0;
    transition:
      opacity 0.15s,
      color 0.15s;
  }
  .remove svg {
    width: 9px;
    height: 9px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
  }
  .item:hover .remove,
  .remove:focus-visible {
    opacity: 1;
  }
  .remove:hover {
    color: #fff;
  }

  button:focus-visible,
  .item:focus-visible {
    outline: 1.5px solid rgba(255, 255, 255, 0.8);
    outline-offset: 2px;
  }
  @media (prefers-reduced-motion: reduce) {
    .zone,
    .item,
    .remove,
    .clear {
      transition: none;
    }
  }
</style>
