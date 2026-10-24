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
    g.fillStyle = "#23201a";
    g.beginPath();
    g.roundRect(8, 4, 48, 56, 10);
    g.fill();
    g.fillStyle = "#fbbf24";
    g.beginPath();
    g.roundRect(15, 15, 34, 34, 7);
    g.fill();
    g.fillStyle = "#241a04";
    g.font = "bold 11px Segoe UI";
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
    <span class="mark" aria-hidden="true">
      <svg viewBox="0 0 24 24"><path d="M4 8h16v11H4zM4 8l2-3h5l2 3" /></svg>
    </span>
    <span class="title">Shelf</span>
    <span class="count">{shelf.items.length}</span>
    <span class="hint">tarik keluar · klik buka</span>
    {#if shelf.items.length}
      <button class="clear" onclick={() => shelf.clear()} aria-label="Bersihkan shelf">
        <svg viewBox="0 0 24 24"><path d="M6 6l12 12M18 6 6 18" /></svg>
      </button>
    {/if}
  </div>

  {#if dropping}
    <!-- Two places to drop: keep it on the shelf, or hand it to Windows' Share window. -->
    <div class="zones">
      <div class="zone" class:hot={dropTarget !== "share"} data-drop="shelf">
        <span class="zone-icon" aria-hidden="true">
          <svg viewBox="0 0 24 24"><path d="M4 8h16v11H4zM4 8l2-3h5l2 3M12 12v4M10 14h4" /></svg>
        </span>
        <span><strong>Simpan</strong><small>di Shelf</small></span>
      </div>
      <div class="zone" class:hot={dropTarget === "share"} data-drop="share">
        <span class="zone-icon" aria-hidden="true">
          <svg viewBox="0 0 24 24"><path d="M4 12v7h16v-7M12 3v12M8 7l4-4 4 4" /></svg>
        </span>
        <span><strong>Bagikan</strong><small>via Windows</small></span>
      </div>
    </div>
  {:else if shelf.items.length === 0}
    <div class="empty-zone" data-drop="shelf">
      <span class="empty-icon" aria-hidden="true">
        <svg viewBox="0 0 24 24"><path d="M4 8h16v11H4zM4 8l2-3h5l2 3" /><path d="M9 13h6" /></svg>
      </span>
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
              <svg viewBox="0 0 24 24" class="folder"><path d="M3 7h7l2 2h9v11H3z" /></svg>
            {:else}
              <span class="ext">{ext(item.name) || "FILE"}</span>
            {/if}
          </span>
          <span class="name">{item.name}</span>
          <span class="size">{item.dir ? "Folder" : size(item.size)}</span>
          <button class="remove" onpointerdown={(e) => e.stopPropagation()} onclick={() => shelf.remove(item.path)} aria-label="Hapus dari shelf">
            <svg viewBox="0 0 24 24"><path d="M6 6l12 12M18 6 6 18" /></svg>
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
    display: flex;
    flex-direction: column;
    padding: 8px 42px 18px;
    overflow: hidden;
    --accent: #fbbf24;
    --accent-soft: rgba(251, 191, 36, 0.13);
    --accent-line: rgba(251, 191, 36, 0.38);
  }
  .page::before {
    content: "";
    position: absolute;
    inset: 0;
    pointer-events: none;
    background: radial-gradient(150px 90px at 16% 12%, rgba(251, 191, 36, 0.12), transparent 70%);
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
    display: grid;
    place-items: center;
    min-width: 15px;
    height: 15px;
    padding: 0 3px;
    border-radius: 5px;
    background: var(--accent-soft);
    color: var(--accent);
    font-size: 8.5px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .hint {
    flex: 1;
    font-size: 8.5px;
    color: rgba(255, 255, 255, 0.35);
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

  .empty-zone {
    position: relative;
    z-index: 1;
    flex: 1;
    min-height: 0;
    margin-top: 5px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 9px;
    border: 1px dashed rgba(251, 191, 36, 0.3);
    border-radius: 13px;
    background:
      radial-gradient(180px 70px at 50% 120%, rgba(251, 191, 36, 0.1), transparent 70%),
      rgba(255, 255, 255, 0.018);
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
  .empty-copy {
    font-size: 11px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.8);
  }
  .empty-copy small {
    display: block;
    margin-top: 1px;
    font-size: 8.5px;
    font-weight: 400;
    color: rgba(255, 255, 255, 0.4);
  }

  .zones {
    position: relative;
    z-index: 1;
    flex: 1;
    min-height: 0;
    display: flex;
    gap: 6px;
    margin-top: 5px;
  }
  .zone {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    border: 1px dashed rgba(255, 255, 255, 0.16);
    border-radius: 12px;
    color: rgba(255, 255, 255, 0.55);
    transition:
      border-color 0.2s,
      background 0.2s,
      color 0.2s;
  }
  .zone.hot {
    border-color: var(--accent);
    background: var(--accent-soft);
    color: #fef3c7;
  }
  .zone-icon {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 7px;
    background: rgba(255, 255, 255, 0.06);
    color: var(--accent);
  }
  .zone.hot .zone-icon {
    background: rgba(251, 191, 36, 0.2);
  }
  .zone-icon svg {
    width: 14px;
    height: 14px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.7;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .zone strong {
    display: block;
    font-size: 10.5px;
    font-weight: 600;
  }
  .zone small {
    display: block;
    font-size: 8px;
    color: rgba(255, 255, 255, 0.38);
  }

  .items {
    position: relative;
    z-index: 1;
    flex: 1;
    min-height: 0;
    display: flex;
    gap: 6px;
    margin-top: 5px;
    padding-bottom: 2px;
    overflow-x: auto;
    scrollbar-width: none;
  }
  .item {
    position: relative;
    flex: none;
    width: 64px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 4px 4px 3px;
    border: 0.5px solid transparent;
    border-radius: 10px;
    cursor: grab;
    transition:
      background 0.18s,
      border-color 0.18s,
      transform 0.18s;
  }
  .item:hover {
    border-color: rgba(251, 191, 36, 0.2);
    background: rgba(251, 191, 36, 0.07);
    transform: translateY(-1px);
  }
  .item:active {
    cursor: grabbing;
    transform: scale(0.97);
  }
  .thumb {
    display: grid;
    place-items: center;
    width: 38px;
    height: 38px;
    overflow: hidden;
    border-radius: 8px;
    background: #211d15;
    box-shadow: inset 0 0 0 0.5px rgba(255, 255, 255, 0.1);
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
    stroke: var(--accent);
    stroke-width: 1.6;
    stroke-linejoin: round;
  }
  .ext {
    font-size: 8px;
    font-weight: 750;
    letter-spacing: 0.03em;
    color: var(--accent);
  }
  .name {
    width: 100%;
    margin-top: 1px;
    text-align: center;
    font-size: 8.5px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .size {
    font-size: 7.5px;
    color: rgba(255, 255, 255, 0.35);
    font-variant-numeric: tabular-nums;
  }
  .remove {
    position: absolute;
    top: 2px;
    right: 2px;
    display: grid;
    place-items: center;
    width: 14px;
    height: 14px;
    padding: 0;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.55);
    color: rgba(255, 255, 255, 0.55);
    opacity: 0;
    transition:
      opacity 0.15s,
      color 0.15s;
  }
  .remove svg {
    width: 8px;
    height: 8px;
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
    color: #f87171;
  }

  button:focus-visible,
  .item:focus-visible {
    outline: 1.5px solid var(--accent);
    outline-offset: 2px;
  }
  @media (prefers-reduced-motion: reduce) {
    .item,
    .remove {
      transition: none;
    }
  }
</style>
