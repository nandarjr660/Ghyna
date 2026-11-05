<script lang="ts">
  // Notification history: the mirrored toasts that arrived in the last minute.
  // System toasts (Windows battery, network, defender...) never reach this list.
  import { invoke } from "@tauri-apps/api/core";

  export type NotifEntry = {
    key: string;
    id: number | null;
    app: string;
    title: string;
    body: string;
    icon: string | null;
    photo: string | null;
    aumid: string;
    at: number;
  };

  let { items, ttl }: { items: NotifEntry[]; ttl: number } = $props();

  // A local clock drives the thin lifetime bars without asking the parent to tick.
  let now = $state(Date.now());
  $effect(() => {
    const id = setInterval(() => (now = Date.now()), 250);
    return () => clearInterval(id);
  });

  const left = (at: number) => Math.max(0, 1 - (now - at) / ttl);

  function open(item: NotifEntry) {
    if (item.aumid) invoke("open_app", { aumid: item.aumid, id: item.id ?? undefined });
  }
</script>

<div class="page">
  <div class="header">
    <span class="title">Notifikasi</span>
    <span class="count">{items.length ? `${items.length} aktif` : "kosong"}</span>
    <span class="hint">hilang dalam {Math.round(ttl / 1000)} dtk</span>
  </div>

  {#if items.length === 0}
    <div class="empty">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M6 9a6 6 0 1 1 12 0c0 4 1.5 5.5 2 6H4c.5-.5 2-2 2-6Z" />
        <path d="M10 19a2 2 0 0 0 4 0" />
      </svg>
      <span>Belum ada notifikasi<small>Notifikasi aplikasi tersimpan {Math.round(ttl / 1000)} detik. Notifikasi sistem dilewati.</small></span>
    </div>
  {:else}
    <div class="list scroll">
      {#each items as item (item.key)}
        <button class="item" onclick={() => open(item)} title={item.app}>
          <span class="face" aria-hidden="true">
            {#if item.photo || item.icon}
              <img src={item.photo ?? item.icon ?? ""} alt="" />
            {:else}
              <svg viewBox="0 0 24 24"><path d="M6 9a6 6 0 1 1 12 0c0 4 1.5 5.5 2 6H4c.5-.5 2-2 2-6Z" /></svg>
            {/if}
          </span>
          <span class="body">
            <span class="line">
              <b>{item.title}</b>
              <small>{item.app}</small>
            </span>
            {#if item.body}<span class="detail">{item.body}</span>{/if}
          </span>
          <span class="life" aria-hidden="true">
            <i style:transform={`scaleX(${left(item.at)})`}></i>
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
    padding: 9px 40px 14px;
    overflow: hidden;
    font-family: var(--font-ui);
  }
  .header {
    flex: none;
    display: flex;
    align-items: center;
    gap: 7px;
    height: 22px;
  }
  .title {
    font-family: var(--font-display);
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.015em;
    color: var(--accent);
  }
  .count {
    padding: 2px 6px;
    border-radius: var(--radius-xs);
    background: rgba(255, 255, 255, 0.07);
    box-shadow: inset 0 0 0 0.5px var(--line);
    font-family: var(--font-mono);
    font-size: 8.5px;
    letter-spacing: 0.04em;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
  .hint {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 8px;
    letter-spacing: 0.05em;
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

  .empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 9px;
    margin-top: 6px;
    border-top: 1px solid var(--line);
    font-family: var(--font-mono);
    font-size: 9.5px;
    letter-spacing: 0.03em;
    color: var(--muted-strong);
  }
  .empty > svg {
    width: 18px;
    height: 18px;
    fill: none;
    stroke: var(--muted);
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .empty small {
    display: block;
    margin-top: 2px;
    font-family: var(--font-ui);
    font-size: 9px;
    color: var(--muted);
    max-width: 250px;
  }

  .list {
    flex: 1;
    min-height: 0;
    margin-top: 4px;
    display: flex;
    flex-direction: column;
    gap: 3px;
    overflow-y: auto;
    scrollbar-width: none;
  }
  .item {
    position: relative;
    flex: none;
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 3px 6px 4px 4px;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.03);
    text-align: left;
    overflow: hidden;
    transition:
      background 160ms var(--ease-out),
      border-color 160ms var(--ease-out),
      transform 120ms var(--ease-out);
    animation: notif-in 240ms var(--ease-out) both;
  }
  .item:hover {
    border-color: rgba(255, 255, 255, 0.2);
    background: rgba(255, 255, 255, 0.07);
  }
  .item:active {
    transform: scale(0.98);
  }
  .face {
    flex: none;
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 7px;
    background: rgba(255, 255, 255, 0.08);
    color: var(--muted);
    overflow: hidden;
  }
  .face img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .face svg {
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
    flex-direction: column;
    gap: 1px;
  }
  .line {
    display: flex;
    align-items: baseline;
    gap: 6px;
    min-width: 0;
  }
  .line b {
    flex: 0 1 auto;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.005em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .line small {
    flex: none;
    font-family: var(--font-mono);
    font-size: 8px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .detail {
    font-size: 9.5px;
    line-height: 1.25;
    color: var(--muted-strong);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  /* Lifetime bar: full when it arrives, gone when the entry is dropped. */
  .life {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 1.5px;
    background: rgba(255, 255, 255, 0.05);
  }
  .life i {
    display: block;
    height: 100%;
    transform-origin: left center;
    background: var(--accent);
    opacity: 0.55;
    transition: transform 240ms linear;
  }

  @keyframes notif-in {
    from {
      opacity: 0;
      transform: translateY(-4px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  @media (prefers-reduced-motion: reduce) {
    .item {
      transition: none;
      animation: none;
    }
    .life i {
      transition: none;
    }
  }
</style>
