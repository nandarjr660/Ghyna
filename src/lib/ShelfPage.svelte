<script lang="ts">
import { open } from "@tauri-apps/plugin-dialog";

function addFiles() {
  // Open file picker dialog, allow multiple selection
  // @ts-ignore – Tauri dialog returns string | string[] | null
  open({ multiple: true, directory: false }).then((result) => {
    // result can be a string or an array of strings
    const paths = Array.isArray(result) ? result : result ? [result] : [];
    if (paths.length) shelf.add(paths);
  }).catch((e) => console.error('Failed to open file dialog:', e));
}

  import { convertFileSrc, invoke } from "@tauri-apps/api/core";
  import { startDrag } from "@crabnebula/tauri-plugin-drag";
  import type { Shelf, ShelfItem } from "./productivity.svelte";

  let { shelf, dropping }: { shelf: Shelf; dropping: boolean } = $props();

  const IMAGE = /\.(png|jpe?g|gif|webp|bmp)$/i;
  const ext = (name: string) => (name.includes(".") ? name.split(".").pop()!.slice(0, 4).toUpperCase() : "");
  const fileKind = (name: string) => {
    const value = name.split(".").pop()?.toLowerCase() ?? "";
    if (/^(doc|docx|odt|rtf)$/.test(value)) return "document";
    if (/^(pdf)$/.test(value)) return "pdf";
    if (/^(xls|xlsx|csv|ods)$/.test(value)) return "spreadsheet";
    if (/^(ppt|pptx|odp)$/.test(value)) return "presentation";
    if (/^(zip|rar|7z|tar|gz)$/.test(value)) return "archive";
    if (/^(txt|md|json|xml|html?|css|js|ts|py|rs)$/.test(value)) return "text";
    return "file";
  };
  const fileLabel = (name: string) => ext(name) || "FILE";

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
    startDrag({ item: [item.path], icon: dragIcon(item) }).then(() => shelf.remove(item.path)).catch(() => {});
  }
  function up() {
  if (pressed) invoke("open_path", { path: pressed.item.path });
  pressed = null;
}

function handleDrop(e: DragEvent) {
  e.preventDefault();
  const dt = e.dataTransfer;
  if (!dt) return;
  const files = Array.from(dt.files);
  // Tauri's file objects expose a `path` property
  const paths = files.map((f:any) => (f as any).path).filter(Boolean);
  if (paths.length) shelf.add(paths);
}


  const size = (bytes: number) =>
    bytes >= 1e6 ? `${(bytes / 1e6).toFixed(1)} MB` : bytes >= 1e3 ? `${Math.round(bytes / 1e3)} KB` : `${bytes} B`;
</script>

<svelte:window onpointermove={move} onpointerup={() => (pressed = null)} />

<div class="page" class:dropping data-drop="shelf" ondragover={e => e.preventDefault()} ondrop={handleDrop} role="region" aria-label="Shelf drop zone">
  <div class="header">
        <span class="title">Shelf</span>
    <span class="count">{shelf.items.length}</span>
    <span class="hint">tarik keluar · klik buka</span>
    {#if shelf.items.length}
      <button class="clear" onclick={() => shelf.clear()} aria-label="Hapus semua dari Shelf" title="Hapus semua">
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M4 7h16M10 4h4l1 3H9l1-3Z" />
          <path d="m7 7 1 13h8l1-13M10 10v7M14 10v7" />
        </svg>
      </button>
    {/if}
  </div>

  {#if dropping}
    <!-- Files dropped while the shelf is open are always saved here. -->
    <div class="zones">
      <div class="zone hot" data-drop="shelf">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 8h16v11H4zM4 8l2-3h5l2 3M12 12v4M10 14h4" /></svg>
        <span><strong>Simpan</strong><small>di Shelf</small></span>
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
              <span class="file-icon {fileKind(item.name)}" aria-hidden="true">
                <svg viewBox="0 0 24 24"><path d="M6 3h8l4 4v14H6zM14 3v5h5" /><path d="M9 13h6M9 17h5" /></svg>
                <b>{fileLabel(item.name)}</b>
              </span>
            {/if}
          </span>
          <span class="name">{item.name}</span>
          <span class="size">{item.dir ? "Folder" : size(item.size)}</span>
          <button class="copy" onpointerdown={(e) => e.stopPropagation()} onclick={() => shelf.add([item.path])} aria-label="Copy ke Shelf" title="Copy ke Shelf">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M9 9H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><path d="M15 15h4a2 2 0 0 0 2-2v-4"/><path d="M9 15v4a2 2 0 0 0 2 2h4"/><path d="M15 9V5a2 2 0 0 0-2-2h-4"/></svg>
          </button>
          <button class="remove" onpointerdown={(e) => e.stopPropagation()} onclick={() => shelf.remove(item.path)} aria-label="Hapus dari shelf" title="Hapus dari shelf">
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
    padding: 9px 40px 18px;
    overflow: hidden;
  }
  .header {
    flex: none;
    display: flex;
    align-items: center;
    gap: 7px;
    height: 24px;
  }
  .title {
    font-size: 15px;
    font-weight: 600;
    color: var(--accent);
  }
  .count {
    padding: 2px 6px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.07);
    font-size: 10px;
    color: rgba(255, 255, 255, 0.38);
    font-variant-numeric: tabular-nums;
  }
  .hint {
    flex: 1;
    font-size: 9px;
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
    width: 24px;
    height: 24px;
    padding: 0;
    border-radius: 6px;
    margin-left: 2px;
    color: rgba(255, 255, 255, 0.5);
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
    gap: 6px;
    margin-top: 5px;
    padding: 2px 1px 2px 0;
    overflow-x: auto;
    scrollbar-width: none;
  }
  .item {
    position: relative;
    flex: none;
    width: 76px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 5px 5px 4px;
    border: 1px solid rgba(255, 255, 255, 0.07);
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.025);
    cursor: grab;
    transition:
      background 0.15s,
      border-color 0.15s;
  }
  .item:hover {
    border-color: rgba(255, 255, 255, 0.18);
    background: rgba(255, 255, 255, 0.07);
  }
  .item:active {
    cursor: grabbing;
  }
  .thumb {
    display: grid;
    place-items: center;
    width: 46px;
    height: 42px;
    overflow: hidden;
    border-radius: 8px;
    background: var(--surface-raised);
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
  .file-icon {
    position: relative;
    display: grid;
    place-items: center;
    width: 30px;
    height: 34px;
    color: rgba(255, 255, 255, 0.62);
  }
  .file-icon svg {
    width: 30px;
    height: 34px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.4;
    stroke-linejoin: round;
  }
  .file-icon b {
    position: absolute;
    top: 17px;
    max-width: 25px;
    overflow: hidden;
    font-size: 6px;
    font-weight: 700;
    letter-spacing: -0.02em;
    line-height: 1;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .file-icon.document { color: #78b7ff; }
  .file-icon.pdf { color: #ff8278; }
  .file-icon.spreadsheet { color: #79d39a; }
  .file-icon.presentation { color: #ffb36b; }
  .file-icon.archive { color: #c4a7ff; }
  .file-icon.text { color: #aeb8c8; }
  .name {
    width: 100%;
    margin-top: 2px;
    text-align: center;
    font-size: 10.5px;
    color: rgba(255, 255, 255, 0.9);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .size {
    font-size: 9.5px;
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
  .clear:active,
  .remove:active,
  .copy:active,
  .item:active {
    transform: scale(0.97);
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
      transform: none !important;
    }
  }
</style>
