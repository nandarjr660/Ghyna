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
  let pageEl = $state<HTMLElement>();
  let dragX = $state(50);
  let dragY = $state(50);
  let clearing = $state(false);
  let clearTimer: ReturnType<typeof setTimeout> | undefined;

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
    // The drag resolves for both "Dropped" and "Cancelled". Only remove the
    // file when it was actually dropped somewhere else; a cancelled drag
    // (released outside any target) keeps it on the Shelf.
    // Keep the file: a chat app may still be uploading it right after the
    // drop. Orphans are swept on the next launch.
    startDrag({ item: [item.path], icon: dragIcon(item) }, ({ result }) => {
      if (result === "Dropped") shelf.remove(item.path, false);
    }).catch((err) => console.error("Failed to start drag:", err));
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

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    const rect = pageEl?.getBoundingClientRect();
    if (!rect) return;
    dragX = Math.max(16, Math.min(rect.width - 16, e.clientX - rect.left));
    dragY = Math.max(16, Math.min(rect.height - 16, e.clientY - rect.top));
  }

  function clearShelf() {
    if (clearing || !shelf.items.length) return;
    clearing = true;
    clearTimer = setTimeout(() => {
      shelf.clear();
      clearing = false;
      clearTimer = undefined;
    }, 740);
  }


  const size = (bytes: number) =>
    bytes >= 1e6 ? `${(bytes / 1e6).toFixed(1)} MB` : bytes >= 1e3 ? `${Math.round(bytes / 1e3)} KB` : `${bytes} B`;
</script>

<svelte:window onpointermove={move} onpointerup={() => (pressed = null)} />

<div bind:this={pageEl} class="page" class:dropping class:clearing data-drop="shelf" style={`--drag-x:${dragX}px;--drag-y:${dragY}px`} ondragover={handleDragOver} ondrop={handleDrop} role="region" aria-label="Shelf drop zone">
  <div class="header">
        <span class="title">Shelf</span>
    <span class="count">{shelf.items.length}</span>
    <span class="hint">tarik keluar · klik buka</span>
    {#if shelf.items.length}
      <button class="clear" onclick={clearShelf} aria-label="Hapus semua dari Shelf" title="Hapus semua">
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
      <span class="empty-copy">Tarik file atau gambar ke sini<small>Dari Explorer atau browser. Simpan sementara, lalu tarik keluar kapan saja.</small></span>
    </div>
  {:else}
      <div class="items scroll">
      {#each shelf.items as item, index (item.path)}
        <div class="item" role="button" tabindex="-1" title={item.name}
          style={`--fly-delay:${Math.min(index, 8) * 36}ms`}
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
    padding: 9px 40px 14px;
    overflow: hidden;
    font-family: var(--font-ui);
  }
  .page::after {
    content: "";
    position: absolute;
    z-index: 0;
    width: 84px;
    height: 84px;
    left: calc(var(--drag-x) - 42px);
    top: calc(var(--drag-y) - 42px);
    border-radius: 50%;
    pointer-events: none;
    opacity: 0;
    background: radial-gradient(circle, var(--accent-soft), transparent 68%);
    transform: scale(.7);
    transition: opacity var(--motion-ui) var(--ease-out), transform 220ms var(--ease-out);
  }
  .page.dropping::after {
    opacity: 1;
    transform: scale(1);
    animation: shelf-magnet 1.3s ease-in-out infinite;
  }
  .header {
    position: relative;
    z-index: 1;
    flex: none;
    display: flex;
    align-items: center;
    gap: 7px;
    height: 24px;
  }
  .title {
    font-family: var(--font-display);
    font-size: 15px;
    font-weight: 600;
    letter-spacing: 0.015em;
    color: var(--accent);
  }
  .count {
    padding: 2px 6px;
    border-radius: var(--radius-xs);
    background: rgba(255, 255, 255, 0.07);
    font-family: var(--font-mono);
    font-size: 9px;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
  .hint {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 8.5px;
    letter-spacing: 0.02em;
    color: var(--muted);
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
    border-radius: var(--radius-xs);
    margin-left: 2px;
    color: var(--muted);
    transition:
      background var(--motion-ui) var(--ease-out),
      color var(--motion-ui) var(--ease-out);
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
    background: rgba(255, 255, 255, 0.1);
    color: var(--text);
  }

  .empty-zone {
    flex: 1;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    margin-top: 7px;
    border: 1px dashed var(--line);
    border-radius: var(--radius-md);
    color: var(--muted);
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
    font-size: 12.5px;
    font-weight: 500;
    color: var(--muted-strong);
  }
  .empty-copy small {
    display: block;
    margin-top: 2px;
    font-family: var(--font-mono);
    font-size: 9.5px;
    color: var(--muted);
  }

  .zones {
    position: relative;
    z-index: 1;
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
    border: 1px dashed var(--line);
    border-radius: var(--radius-md);
    color: var(--muted);
    transition:
      border-color var(--motion-ui) var(--ease-out),
      background var(--motion-ui) var(--ease-out),
      color var(--motion-ui) var(--ease-out);
  }
  .zone.hot {
    border-color: var(--accent-line);
    background: var(--accent-soft);
    color: var(--text);
    animation: shelf-zone-pulse 1.4s ease-in-out infinite;
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
    font-family: var(--font-display);
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.01em;
  }
  .zone small {
    display: block;
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.02em;
    color: var(--muted);
  }

  .items {
    position: relative;
    z-index: 1;
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
    gap: 3px;
    padding: 4px 5px 3px;
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: rgba(255, 255, 255, 0.03);
    cursor: grab;
    transition:
      background var(--motion-ui) var(--ease-out),
      border-color var(--motion-ui) var(--ease-out),
      transform var(--motion-fast) var(--ease-out);
    animation: shelf-attach var(--motion-page) var(--ease-out) both;
  }
  .item:hover {
    border-color: var(--accent-line);
    background: rgba(255, 255, 255, 0.08);
  }
  .item:active {
    cursor: grabbing;
  }
  .thumb {
    flex: none;
    display: grid;
    place-items: center;
    width: 46px;
    height: 38px;
    overflow: hidden;
    border-radius: var(--radius-sm);
    background: var(--surface-raised);
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
    stroke: var(--muted-strong);
    stroke-width: 1.6;
    stroke-linejoin: round;
  }
  .ext {
    font-family: var(--font-mono);
    font-size: 8.5px;
    font-weight: 700;
    letter-spacing: 0.04em;
    color: var(--muted-strong);
  }
  .file-icon {
    position: relative;
    display: grid;
    place-items: center;
    width: 28px;
    height: 30px;
    color: rgba(238, 246, 248, 0.58);
  }
  .file-icon svg {
    width: 28px;
    height: 30px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.4;
    stroke-linejoin: round;
  }
  .file-icon b {
    position: absolute;
    top: 15px;
    max-width: 23px;
    overflow: hidden;
    font-family: var(--font-mono);
    font-size: 6px;
    font-weight: 700;
    letter-spacing: -0.02em;
    line-height: 1;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .file-icon.document,
  .file-icon.pdf,
  .file-icon.spreadsheet,
  .file-icon.presentation,
  .file-icon.archive,
  .file-icon.text {
    color: rgba(238, 246, 248, 0.58);
  }
  .name {
    flex: none;
    width: 100%;
    text-align: center;
    font-size: 10.5px;
    line-height: 1.15;
    color: rgba(238, 246, 248, 0.9);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .size {
    flex: none;
    font-family: var(--font-mono);
    font-size: 9px;
    line-height: 1.15;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
  .copy {
    position: absolute;
    top: 2px;
    left: 2px;
    display: grid;
    place-items: center;
    width: 16px;
    height: 16px;
    padding: 0;
    border-radius: var(--radius-xs);
    background: rgba(6, 9, 11, 0.62);
    box-shadow: inset 0 0 0 0.5px var(--line);
    color: var(--muted-strong);
    opacity: 0;
    transition:
      opacity 150ms var(--ease-out),
      color 150ms var(--ease-out),
      transform var(--motion-fast) var(--ease-out);
  }
  .copy svg {
    width: 10px;
    height: 10px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .item:hover .copy,
  .copy:focus-visible {
    opacity: 1;
  }
  .copy:hover {
    color: var(--text);
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
    border-radius: var(--radius-xs);
    background: rgba(6, 9, 11, 0.62);
    box-shadow: inset 0 0 0 0.5px var(--line);
    color: var(--muted-strong);
    opacity: 0;
    transition:
      opacity 150ms var(--ease-out),
      color 150ms var(--ease-out),
      transform var(--motion-fast) var(--ease-out);
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
    color: var(--text);
  }
  .clear:active,
  .remove:active,
  .copy:active,
  .item:active {
    transform: scale(0.97);
  }
  .clearing .item {
    pointer-events: none;
    animation: shelf-fly 420ms var(--ease-out) var(--fly-delay) both;
  }

  @keyframes shelf-magnet {
    0%, 100% { opacity: .7; transform: scale(.92); }
    50% { opacity: 1; transform: scale(1.08); }
  }
  @keyframes shelf-zone-pulse {
    0%, 100% { box-shadow: 0 0 0 rgba(255, 138, 107, 0); }
    50% { box-shadow: 0 0 22px rgba(255, 138, 107, .14); }
  }
  @keyframes shelf-attach {
    from { opacity: 0; transform: translateY(-8px) scale(.94); }
    70% { transform: translateY(1px) scale(1.015); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }
  @keyframes shelf-fly {
    30% { opacity: 1; transform: translate(36px, -16px) scale(.94) rotate(3deg); }
    to { opacity: 0; transform: translate(145px, -42px) scale(.35) rotate(5deg); }
  }

  button:focus-visible,
  .item:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  @media (prefers-reduced-motion: reduce) {
    .zone,
    .item,
    .remove,
    .copy,
    .clear {
      transition: none;
      transform: none !important;
      animation: none !important;
    }
  }
</style>
