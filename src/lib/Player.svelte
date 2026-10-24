<script lang="ts">
  import Bars from "./Bars.svelte";
  import { cleanTrack, currentLine, type Lyrics } from "./lyrics";
  import { formatTime, next, positionAt, prev, seek, toggle, type NowPlaying } from "./media";

  // Lyrics are fetched by the page, which also shows them in the small notch.
  let {
    media,
    now,
    clock,
    withLyrics,
    onLyrics,
    lyrics,
    status,
  }: {
    media: NowPlaying;
    now: number;
    clock: string;
    withLyrics: boolean;
    onLyrics: () => void;
    lyrics: Lyrics | null;
    status: "loading" | "done" | "offline";
  } = $props();

  const track = $derived(cleanTrack(media.title, media.artist, media.source));
  const pos = $derived(positionAt(media, now));
  const progress = $derived(media.duration > 0 ? pos / media.duration : 0);

  const lineIndex = $derived(lyrics?.synced ? currentLine(lyrics.synced, pos) : -1);

  // Keep the current line in the middle of the panel.
  let panel = $state<HTMLElement>();
  let list = $state<HTMLElement>();
  let offset = $state(0);
  $effect(() => {
    const line = list?.children[Math.max(lineIndex, 0)] as HTMLElement | undefined;
    if (panel && line) offset = panel.clientHeight / 2 - (line.offsetTop + line.offsetHeight / 2);
  });

  function seekTo(e: MouseEvent) {
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    seek(((e.clientX - r.left) / r.width) * media.duration);
  }
</script>

<!-- Layout: cover beside the header and title, progress underneath,
     then the lyrics button under the cover and the transport centered. -->
<div class="player" class:split={withLyrics}>
  <div class="main">
    <div class="top">
      <div class="art" style:background-image={media.art ? `url(${media.art})` : null}>
        {#if !media.art}♪{/if}
      </div>
      <div class="meta">
        {#if !withLyrics}
          <div class="header">
            <span>{clock}</span>
            <Bars live={media.playing} />
          </div>
        {/if}
        <div class="title">{track.title}</div>
        <div class="artist">{[track.artist, media.album].filter(Boolean).join(" - ")}</div>
      </div>
    </div>

    <div class="progress">
      <span>{formatTime(pos)}</span>
      <button class="bar" onclick={seekTo} disabled={!media.duration} aria-label="Geser posisi lagu">
        <span class="fill" style:width="{progress * 100}%"></span>
      </button>
      <span>{media.duration ? `-${formatTime(media.duration - pos)}` : "--:--"}</span>
    </div>

    <div class="controls">
      <button class="lyrics-chip" class:on={withLyrics} onclick={onLyrics} title={withLyrics ? "Sembunyikan lirik" : "Tampilkan lirik"} aria-label="Lirik">
        <svg viewBox="0 0 24 24"><path d="M4 5h16v11H9l-5 4z" /><path d="M8 9h8M8 12h5" /></svg>
        <span>Lirik</span>
      </button>
      <button class="icon" onclick={prev} aria-label="Sebelumnya">
        <svg viewBox="0 0 24 24" class="solid"><path d="M6 6h2v12H6zM9.5 12 18 18V6z" /></svg>
      </button>
      <button class="icon play" onclick={toggle} aria-label={media.playing ? "Jeda" : "Putar"}>
        {#if media.playing}
          <svg viewBox="0 0 24 24" class="solid"><path d="M7 5h3.5v14H7zM13.5 5H17v14h-3.5z" /></svg>
        {:else}
          <svg viewBox="0 0 24 24" class="solid"><path d="M8 5v14l11-7z" /></svg>
        {/if}
      </button>
      <button class="icon" onclick={next} aria-label="Berikutnya">
        <svg viewBox="0 0 24 24" class="solid"><path d="M16 6h2v12h-2zM6 18l8.5-6L6 6z" /></svg>
      </button>
    </div>
  </div>

  {#if withLyrics}
    <div class="lyrics" bind:this={panel}>
      {#if lyrics?.synced}
        <ol bind:this={list} style:transform="translateY({offset}px)">
          {#each lyrics.synced as line, i}
            <li class:now={i === lineIndex} class:past={i < lineIndex}>
              <button onclick={() => seek(line.time)}>{line.text}</button>
            </li>
          {/each}
        </ol>
      {:else if lyrics?.plain}
        <div class="plain">{lyrics.plain}</div>
      {:else}
        <p class="note">
          {status === "loading" ? "Mencari lirik…" : status === "offline" ? "LRCLIB tidak bisa dihubungi" : "Lirik belum tersedia untuk lagu ini"}
        </p>
      {/if}
    </div>
  {/if}
</div>

<style>
  .player {
    position: absolute;
    inset: 0;
    display: flex;
    padding: 6px 48px 24px;
    box-sizing: border-box;
  }
  .main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
  }
  .split .main {
    flex: 0 0 180px;
    justify-content: center;
    gap: 6px;
  }

  .top {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .art {
    flex: none;
    width: 44px;
    height: 44px;
    border-radius: 10px;
    overflow: hidden;
    background: #1b1f2e center / cover;
    display: grid;
    place-items: center;
    color: rgba(255, 255, 255, 0.4);
    box-shadow: 0 6px 16px -6px rgba(0, 0, 0, 0.9);
  }
  .split .art {
    width: 40px;
    height: 40px;
    border-radius: 9px;
  }
  .meta {
    flex: 1;
    min-width: 0;
  }
  .header {
    position: relative;
    display: flex;
    justify-content: center;
    align-items: center;
    margin-bottom: 3px;
    font-size: 10.5px;
    color: rgba(255, 255, 255, 0.68);
    font-variant-numeric: tabular-nums;
  }
  .header :global(.bars) {
    position: absolute;
    right: 0;
  }
  .title,
  .artist {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .title {
    font-size: 13px;
    font-weight: 600;
    line-height: 1.2;
  }
  .artist {
    margin-top: 2px;
    font-size: 10.5px;
    color: rgba(255, 255, 255, 0.5);
  }

  .progress {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 10.5px;
    color: rgba(255, 255, 255, 0.8);
    font-variant-numeric: tabular-nums;
  }
  .progress > span {
    width: 34px;
  }
  .progress > span:last-child {
    text-align: right;
  }
  .bar {
    position: relative;
    flex: 1;
    height: 12px;
    padding: 0;
    border: 0;
    background: none;
    display: flex;
    align-items: center;
    cursor: pointer;
  }
  .bar::after {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    height: 4px;
    border-radius: 2px;
    background: rgba(255, 255, 255, 0.16);
  }
  .fill {
    position: relative;
    z-index: 1;
    height: 4px;
    border-radius: 2px;
    background: #fff;
  }

  /* Transport centered; the lyrics chip sits on the left, under the cover. */
  .controls {
    position: relative;
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 22px;
  }
  .split .controls {
    gap: 12px;
  }

  .icon {
    padding: 0;
    border: 0;
    background: none;
    color: #fff;
    cursor: pointer;
    display: grid;
    place-items: center;
    border-radius: 50%;
    transition:
      transform 0.15s,
      opacity 0.15s;
  }
  .icon:hover {
    transform: scale(1.1);
  }
  .icon:active {
    transform: scale(0.92);
  }
  .icon svg {
    width: 18px;
    height: 18px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .icon svg.solid {
    fill: currentColor;
    stroke: none;
  }
  .play svg {
    width: 24px;
    height: 24px;
  }
  .lyrics-chip {
    position: absolute;
    left: 0;
    display: flex;
    align-items: center;
    gap: 5px;
    height: 24px;
    padding: 0 9px;
    border: 0;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.07);
    color: rgba(255, 255, 255, 0.62);
    font: inherit;
    font-size: 9.5px;
    font-weight: 600;
    letter-spacing: 0.02em;
    cursor: pointer;
    transition:
      background 0.2s,
      color 0.2s,
      transform 0.18s;
  }
  .lyrics-chip svg {
    width: 13px;
    height: 13px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.9;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .lyrics-chip:hover {
    background: rgba(255, 255, 255, 0.14);
    color: #fff;
    transform: translateY(-1px);
  }
  .lyrics-chip.on {
    color: #fff;
    background: rgba(255, 255, 255, 0.16);
  }

  .lyrics {
    flex: 1;
    position: relative;
    overflow: hidden;
    margin-left: 12px;
    padding-left: 12px;
    border-left: 0.5px solid rgba(255, 255, 255, 0.12);
    mask: linear-gradient(transparent, #000 22%, #000 78%, transparent);
  }
  ol {
    list-style: none;
    margin: 0;
    padding: 0;
    transition: transform 0.7s cubic-bezier(0.34, 1.32, 0.5, 1);
  }
  li button {
    display: block;
    width: 100%;
    padding: 4px 0;
    border: 0;
    background: none;
    text-align: left;
    font: inherit;
    font-size: 14px;
    font-weight: 700;
    line-height: 1.22;
    color: rgba(255, 255, 255, 0.3);
    cursor: pointer;
    transform: scale(0.94);
    transform-origin: left center;
    transition:
      color 0.5s,
      transform 0.5s;
  }
  li.past button {
    color: rgba(255, 255, 255, 0.2);
  }
  li.now button {
    color: #fff;
    transform: none;
  }
  li button:hover {
    color: rgba(255, 255, 255, 0.7);
  }
  .plain {
    height: 100%;
    overflow-y: auto;
    white-space: pre-line;
    font-size: 13px;
    line-height: 1.5;
    color: rgba(255, 255, 255, 0.75);
  }
  .note {
    margin: 0;
    height: 100%;
    display: grid;
    place-items: center;
    text-align: center;
    font-size: 12.5px;
    color: rgba(255, 255, 255, 0.5);
  }
</style>
