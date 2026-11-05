<script lang="ts">
  import Bars from "./Bars.svelte";
  import { sampleGlow } from "./cover-glow";
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

  // Ambient light follows the cover: one hue sampled from the artwork, kept
  // faint so the panel still reads as smoked glass. Monochrome art, or remote
  // art the canvas cannot read, falls back to the green default.
  let glow = $state<string | null>(null);
  let glowSoft = $state<string | null>(null);
  let sampled = "";
  $effect(() => {
    const art = media.art;
    if (!art) {
      if (sampled) {
        sampled = "";
        glow = null;
        glowSoft = null;
      }
      return;
    }
    if (art === sampled) return;
    sampled = art;
    glow = null;
    glowSoft = null;
    void sampleGlow(art).then((rgb) => {
      if (sampled !== art || !rgb) return;
      glow = `rgba(${rgb[0]}, ${rgb[1]}, ${rgb[2]}, 0.3)`;
      glowSoft = `rgba(${rgb[0]}, ${rgb[1]}, ${rgb[2]}, 0.14)`;
    });
  });

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
<div
  class="player"
  class:split={withLyrics}
  style:--lyric-glow={glow}
  style:--lyric-glow-soft={glowSoft}
>
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
      <button class="lyrics-chip" class:on={withLyrics} onclick={onLyrics} title={withLyrics ? "Sembunyikan lirik" : "Tampilkan lirik"} aria-label="Lirik" aria-pressed={withLyrics}>
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <g fill="none">
            <path fill="currentColor" fill-rule="evenodd" opacity="0.2" d="M18.32 17.43c.9 0 1.36 0 1.73-.1a3 3 0 0 0 2.15-2.14c.1-.37.1-.82.1-1.73V9.72c0-2.83 0-4.24-.88-5.12s-2.3-.88-5.12-.88H7.7c-2.82 0-4.24 0-5.12.88S1.7 6.9 1.7 9.72v3.74c0 .9 0 1.36.1 1.73a3 3 0 0 0 2.15 2.15c.37.1.82.1 1.73.1h.56a3 3 0 0 1 1.12.34c.12.06.24.14.48.3l.9.58c1.58 1.02 2.37 1.53 3.26 1.53s1.68-.5 3.26-1.53l.9-.59l.48-.3a3 3 0 0 1 1.12-.32c.13-.02.28-.02.56-.02M15.34 7.28a2.14 2.14 0 1 1 0 4.28a2.14 2.14 0 0 1 0-4.28M10.6 9.42a2.14 2.14 0 1 0-4.28 0a2.14 2.14 0 0 0 4.28 0" clip-rule="evenodd" />
            <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M17.88 17.43c1.32 0 1.98 0 2.5-.2a3 3 0 0 0 1.71-1.71c.2-.53.2-1.18.2-2.5v-3.3c0-2.83 0-4.24-.87-5.12c-.88-.88-2.3-.88-5.12-.88H7.7c-2.82 0-4.24 0-5.12.88S1.7 6.9 1.7 9.72v3.25c0 1.36 0 2.05.22 2.59a3 3 0 0 0 1.66 1.66c.54.21 1.22.21 2.59.21c.63 0 .94 0 1.24.06a3 3 0 0 1 1.16.51c.24.18.46.41.88.87l.15.16c.88.96 1.32 1.44 1.85 1.6a2 2 0 0 0 1.12 0c.53-.14.97-.62 1.87-1.56l.24-.26c.41-.43.62-.65.85-.82a3 3 0 0 1 1.18-.5c.28-.06.58-.06 1.17-.06" />
            <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M10.6 9.42a2.13 2.13 0 0 0-2.14-2.14a2.14 2.14 0 1 0 2.14 2.14m6.87 0a2.13 2.13 0 0 0-2.14-2.14a2.14 2.14 0 1 0 2.14 2.14" />
            <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M10.6 9.45c.04.55.15 1.97-.35 2.88c-.59 1.09-1.4 1.69-2.47 1.7m9.69-4.58c.05.55.15 1.97-.34 2.87c-.6 1.08-1.4 1.68-2.48 1.7" />
          </g>
        </svg>
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
  /* Ambient light on the player page (plain and lyrics), tinted by the cover. */
  .player::before {
    content: "";
    position: absolute;
    inset: 0;
    /* Ambient light rises from below the page and is tinted by the cover. */
    background:
      radial-gradient(72% 94% at 50% 106%, var(--lyric-glow, rgba(143, 230, 180, 0.34)), transparent 72%),
      radial-gradient(56% 78% at 92% 2%, var(--lyric-glow-soft, rgba(143, 208, 232, 0.18)), transparent 74%);
    filter: blur(14px);
    pointer-events: none;
  }
  .player > * {
    position: relative;
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
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--surface-raised) center / cover;
    display: grid;
    place-items: center;
    color: rgba(255, 255, 255, 0.4);
    box-shadow:
      0 0 0 0.5px rgba(255, 255, 255, 0.16),
      0 10px 20px -10px rgba(0, 0, 0, 0.95);
  }
  .split .art {
    width: 40px;
    height: 40px;
    border-radius: var(--radius-sm);
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
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    color: var(--muted);
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
    font-family: var(--font-display);
    font-size: 13.5px;
    font-weight: 600;
    letter-spacing: 0.01em;
    line-height: 1.2;
  }
  .artist {
    margin-top: 2px;
    font-size: 10.5px;
    color: var(--muted);
  }

  .progress {
    display: flex;
    align-items: center;
    gap: 8px;
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: -0.02em;
    color: var(--muted-strong);
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
    height: 2px;
    border-radius: 2px;
    background: rgba(255, 255, 255, 0.2);
  }
  .fill {
    position: relative;
    z-index: 1;
    height: 2px;
    border-radius: 2px;
    background: var(--accent);
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
    color: var(--text);
    cursor: pointer;
    display: grid;
    place-items: center;
    border-radius: var(--radius-xs);
    transition:
      transform 0.15s var(--ease-out),
      opacity 0.15s var(--ease-out);
  }
  .icon:hover {
    transform: scale(1.08);
  }
  .icon:active {
    transform: scale(0.94);
  }
  .icon:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
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
    justify-content: center;
    width: 20px;
    height: 20px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-xs);
    background: rgba(255, 255, 255, 0.07);
    box-shadow: inset 0 0 0 0.5px rgba(255, 255, 255, 0.1);
    color: var(--muted);
    cursor: pointer;
    transition: color 0.18s var(--ease-out), background 0.18s var(--ease-out);
  }
  .lyrics-chip svg {
    display: block;
    width: 13px;
    height: 13px;
  }
  .lyrics-chip:hover {
    color: var(--text);
    background: rgba(255, 255, 255, 0.12);
  }
  .lyrics-chip.on {
    color: var(--accent);
    background: var(--accent-soft);
    box-shadow: inset 0 0 0 0.5px var(--accent-line);
  }
  .lyrics-chip:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .lyrics {
    flex: 1;
    position: relative;
    overflow: hidden;
    margin-left: 12px;
    padding-left: 12px;
    border-left: 0.5px solid rgba(255, 255, 255, 0.1);
    mask: linear-gradient(transparent, #000 22%, #000 78%, transparent);
  }
  ol {
    list-style: none;
    margin: 0;
    padding: 0;
    transition: transform 0.7s cubic-bezier(0.22, 1, 0.36, 1);
  }
  li button {
    display: block;
    width: 100%;
    padding: 4px 0;
    border: 0;
    background: none;
    text-align: left;
    font: inherit;
    font-family: var(--font-display);
    font-size: 14px;
    font-weight: 500;
    letter-spacing: 0.005em;
    line-height: 1.22;
    color: rgba(238, 246, 248, 0.3);
    cursor: pointer;
    transform: scale(0.94);
    transform-origin: left center;
    transition:
      color 0.5s,
      transform 0.5s;
  }
  li.past button {
    color: rgba(238, 246, 248, 0.18);
  }
  li.now button {
    color: var(--text);
    font-weight: 600;
    transform: none;
  }
  li button:hover {
    color: rgba(238, 246, 248, 0.72);
  }
  .plain {
    height: 100%;
    overflow-y: auto;
    white-space: pre-line;
    font-size: 13px;
    line-height: 1.5;
    color: var(--muted-strong);
  }
  .note {
    margin: 0;
    height: 100%;
    display: grid;
    place-items: center;
    text-align: center;
    font-size: 12.5px;
    color: var(--muted);
  }
</style>
