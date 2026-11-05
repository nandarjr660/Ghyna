<script lang="ts">
  // Camera check before a call: a small live preview in the notch. The camera
  // is opened only while this is shown and released (light off) when it closes.
  import { onDestroy, onMount } from "svelte";

  let { title = "", link = "", onDone, onJoin }: { title?: string; link?: string; onDone: () => void; onJoin: () => void } = $props();

  let video: HTMLVideoElement;
  let stream: MediaStream | null = null;
  let error = $state("");

  onMount(async () => {
    try {
      stream = await navigator.mediaDevices.getUserMedia({ video: { width: 640, height: 360 }, audio: false });
      video.srcObject = stream;
    } catch (e) {
      error = (e as DOMException)?.name === "NotAllowedError" ? "Akses kamera ditolak" : "Kamera tidak bisa dibuka";
    }
  });
  onDestroy(() => stream?.getTracks().forEach((t) => t.stop()));
</script>

<div class="mirror">
  <div class="cam">
    <video bind:this={video} autoplay muted playsinline></video>
    {#if error}<span class="err">{error}</span>{/if}
  </div>
  <div class="side">
    <div class="title"><span class="live"></span>Cek kamera</div>
    <div class="detail">{title || "Pastikan wajah terlihat jelas"}</div>
    <div class="actions">
      <button class="primary" onclick={onDone}>Selesai</button>
      {#if link}<button onclick={onJoin}>Gabung</button>{/if}
    </div>
  </div>
</div>

<style>
  .mirror {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 12px 18px 14px;
  }
  .cam {
    position: relative;
    flex: none;
    width: 176px;
    height: 99px;
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--surface-solid);
    box-shadow: inset 0 0 0 0.5px rgba(255, 255, 255, 0.12);
  }
  video {
    width: 100%;
    height: 100%;
    object-fit: cover;
    /* Like a mirror. */
    transform: scaleX(-1);
  }
  .err {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    font-size: 11.5px;
    color: rgba(255, 255, 255, 0.6);
  }
  .side {
    flex: 1;
    min-width: 0;
  }
  .title {
    font-family: var(--font-display);
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.015em;
  }
  .live {
    display: inline-block;
    width: 6px;
    height: 6px;
    margin-right: 6px;
    border-radius: 50%;
    background: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
  .detail {
    margin-top: 4px;
    font-family: var(--font-mono);
    font-size: 9.5px;
    letter-spacing: 0.02em;
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .actions {
    display: flex;
    gap: 6px;
    margin-top: 12px;
  }
  button {
    padding: 5px 11px;
    border: 0;
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.08);
    box-shadow: inset 0 0 0 0.5px var(--line);
    color: var(--text);
    font: inherit;
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    cursor: pointer;
    transition: background var(--motion-fast) var(--ease-out);
  }
  button:hover {
    background: rgba(255, 255, 255, 0.15);
  }
  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  .err {
    font-family: var(--font-mono);
    font-size: 9.5px;
    letter-spacing: 0.04em;
    color: var(--muted-strong);
  }
  button.primary {
    background: var(--accent);
    color: var(--accent-ink);
    font-weight: 700;
    box-shadow: inset 0 0 0 0.5px rgba(255, 255, 255, 0.3);
  }
  button.primary:hover {
    background: var(--accent-strong);
  }
</style>
