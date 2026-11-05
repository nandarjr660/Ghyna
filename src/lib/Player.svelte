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

  // Ambient light follows the cover: sample one colourful average per artwork and
  // use it for the lyrics glow. Monochrome art, or remote art the canvas cannot
  // read, falls back to the blue default.
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
      glow = `rgba(${rgb[0]}, ${rgb[1]}, ${rgb[2]}, 0.6)`;
      glowSoft = `rgba(${rgb[0]}, ${rgb[1]}, ${rgb[2]}, 0.34)`;
    });
  });

  function sampleGlow(src: string): Promise<[number, number, number] | null> {
    return new Promise((resolve) => {
      const img = new Image();
      img.crossOrigin = "anonymous";
      img.onload = () => {
        try {
          const size = 32;
          const canvas = document.createElement("canvas");
          canvas.width = size;
          canvas.height = size;
          const ctx = canvas.getContext("2d");
          if (!ctx) return resolve(null);
          ctx.drawImage(img, 0, 0, size, size);
          const data = ctx.getImageData(0, 0, size, size).data;
          let r = 0;
          let g = 0;
          let b = 0;
          let weight = 0;
          for (let i = 0; i < data.length; i += 4) {
            const pr = data[i];
            const pg = data[i + 1];
            const pb = data[i + 2];
            const max = Math.max(pr, pg, pb);
            const min = Math.min(pr, pg, pb);
            const saturation = max ? (max - min) / max : 0;
            const luminance = (max + min) / 510;
            // Greys and near-black/near-white pixels only dull the glow.
            const w =
              Math.max(0, saturation - 0.12) *
              Math.max(0, 1 - Math.abs(luminance - 0.45) * 1.6);
            if (w <= 0) continue;
            r += pr * w;
            g += pg * w;
            b += pb * w;
            weight += w;
          }
          if (!weight) return resolve(null);
          r /= weight;
          g /= weight;
          b /= weight;
          // Dark covers give a glow too faint to see; lift the colour until it reads.
          const peak = Math.max(r, g, b);
          const lift = peak > 0 ? Math.min(2.4, 150 / peak) : 1;
          resolve([
            Math.min(255, Math.round(r * lift)),
            Math.min(255, Math.round(g * lift)),
            Math.min(255, Math.round(b * lift)),
          ]);
        } catch {
          // Tainted canvas (remote art without CORS headers).
          resolve(null);
        }
      };
      img.onerror = () => resolve(null);
      img.src = src;
    });
  }

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
  /* Ambient light on the lyrics page, tinted by the cover art. */
  .player.split::before {
    content: "";
    position: absolute;
    inset: 0;
    background:
      radial-gradient(62% 95% at 10% 0%, var(--lyric-glow, rgba(96, 150, 255, 0.55)), transparent 72%),
      radial-gradient(58% 85% at 94% 100%, var(--lyric-glow-soft, rgba(142, 108, 255, 0.32)), transparent 74%);
    filter: blur(14px);
    pointer-events: none;
  }
  .player.split > * {
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
    gap: 4px;
    height: 20px;
    padding: 0 7px;
    border: 0;
    border-radius: var(--radius-xs);
    background: rgba(255, 255, 255, 0.07);
    box-shadow: inset 0 0 0 0.5px rgba(255, 255, 255, 0.1);
    color: var(--muted);
    font-family: var(--font-mono);
    font-size: 8.5px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    cursor: pointer;
    transition: color 0.18s var(--ease-out), background 0.18s var(--ease-out);
  }
  .lyrics-chip svg {
    width: 11px;
    height: 11px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.9;
    stroke-linecap: round;
    stroke-linejoin: round;
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
