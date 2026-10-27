<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";


  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { listen } from "@tauri-apps/api/event";


  import { onMount, untrack } from "svelte";
  import { cubicOut } from "svelte/easing";
  import { fade } from "svelte/transition";
  import ClipboardPage, { type ClipEntry } from "$lib/ClipboardPage.svelte";
  import Island from "$lib/Island.svelte";
  import Mirror from "$lib/Mirror.svelte";
  import Peek, { type PeekData } from "$lib/Peek.svelte";
  import Player from "$lib/Player.svelte";
  import ShelfPage from "$lib/ShelfPage.svelte";
  import { arrange, islandWidth, type Activity, type Call, type Download } from "$lib/island";
  import TimerPage from "$lib/TimerPage.svelte";
  import TodoPage from "$lib/TodoPage.svelte";
  import WaterPage from "$lib/WaterPage.svelte";
  import { positionAt, type NowPlaying } from "$lib/media";
  import { cleanTrack, currentLine, findLyrics, type Lyrics } from "$lib/lyrics";
  import { Pomodoro, Shelf, Todo, Water, type Settings } from "$lib/productivity.svelte";
  import { fetchWeather, type Weather } from "$lib/weather";
  import { upcoming, type CalEvent } from "$lib/calendar";

  // Black box for the page: errors go to ghyna.log, and a heartbeat tells Rust the
  // page still runs and draws (sent from an animation frame, which only comes while
  // the page renders). Without beats for a while, Rust reloads the page.
  const logJs = (msg: string) => invoke("log_js", { msg }).catch(() => {});
  window.addEventListener("error", (e) => logJs(`${e.message} (${String(e.filename).split("/").pop()}:${e.lineno})`));
  window.addEventListener("unhandledrejection", (e) => logJs(`unhandled: ${e.reason?.stack ?? e.reason}`));
  setInterval(() => requestAnimationFrame(() => invoke("heartbeat").catch(() => {})), 5000);

  // Sizes from ghyna.md §4 (logical px).
  const SIZES = {
    // Tucked keeps the normal closed notch width; only its height collapses.
    tucked: { w: 150, h: 6, r: 4, e: 4 },
    rest: { w: 150, h: 30, r: 14, e: 10 },
    // The island's width depends on what it shows (lib/island.ts).
    island: { w: 200, h: 30, r: 14, e: 10 },
    charging: { w: 200, h: 30, r: 14, e: 10 },
    peek: { w: 310, h: 60, r: 19, e: 13 },
    shot: { w: 360, h: 96, r: 24, e: 16 },
    notify: { w: 380, h: 72, r: 22, e: 14 },
    clock: { w: 400, h: 132, r: 30, e: 21 },
    player: { w: 400, h: 132, r: 30, e: 21 },
    lyrics: { w: 500, h: 140, r: 30, e: 21 },
    pomodoro: { w: 400, h: 132, r: 30, e: 21 },
    water: { w: 400, h: 132, r: 30, e: 21 },
    todo: { w: 400, h: 132, r: 30, e: 21 },
    shelf: { w: 400, h: 132, r: 30, e: 21 },
    clipboard: { w: 400, h: 132, r: 30, e: 21 },
    mirror: { w: 400, h: 132, r: 30, e: 21 },
  };

  // Pages of the open notch, switched with the dots or the mouse wheel.
  const ALL_PAGES = ["now", "pomodoro", "water", "todo", "shelf", "clipboard"] as const;
  type PageName = (typeof ALL_PAGES)[number];
  // The water reminder can be paused; the Air page stays available for logging.
  let waterEnabled = $state(true);
  const pages = ALL_PAGES;

  async function toggleWater() {
    const next = !waterEnabled;
    waterEnabled = next;
    try {
      const s = await invoke<Record<string, unknown>>("get_settings");
      s.waterEnabled = next;
      await invoke("save_settings", { settings: s });
    } catch (e) {
      waterEnabled = !next;
      logJs(`water toggle: ${e}`);
    }
  }

  type Power = { charging: boolean; percent: number; battery: boolean };
  type InUse = { camera: string[]; mic: string[] };

  let open = $state(false);
  // The active window sits under the notch (e.g. a maximized browser): shrink to a thin line.
  let tucked = $state(false);
  let blink = $state(false);
  let wink = $state<"left" | "right" | null>(null);
  // A quiet cursor steps the notch through: clock -> awake eyes -> sleeping eyes.
  let idleStage = $state<"clock" | "awake" | "sleep">("clock");
  let waving = $state(false);
  // Where the eyes look (-1..1 per axis) and how close the cursor is (0..1), from Rust.
  let look = $state<[number, number, number]>([0, 0, 0]);
  // A small autonomous glance so the eyes feel alive when the cursor is far away.
  let gaze = $state(0);
  let gazeY = $state(0);
  let gazeTilt = $state(0);
  const eyeX = $derived(look[2] > 0.35 ? look[0] : gaze);
  const eyeY = $derived(look[2] > 0.35 ? look[1] : gazeY);
  let now = $state(new Date());
  let media = $state<NowPlaying | null>(null);
  let lyricsOn = $state(false);
  let peek = $state<PeekData | null>(null);
  let power = $state<Power>({ charging: false, percent: 100, battery: false });
// Determine eye color / expression based on battery level
// low15: <=15% (angry red), low20: 16-20% (annoyed orange), normal: >20% (warm green)
const eyeState = $derived(
  power.charging ? "charging" :
  power.percent <= 15 ? "low15" :
  power.percent <= 20 ? "low20" :
  "normal"
);
  let inUse = $state<InUse>({ camera: [], mic: [] });
  let micMuted = $state(false);
  let weather = $state<Weather | null>(null);
  let page = $state(0);
  // -1 = moving left, 1 = moving right; drives the page slide direction.
  let pageDir = $state(1);
  // Typing a task: the notch holds keyboard focus and stays open.
  let typing = $state(false);
  let hovered = false;
  // A file is being dragged over the notch, and which drop zone it's over.
  let dropping = $state(false);
  let dropTarget = $state<string | null>(null);
  // Things running at once, shown side by side in the closed notch.
  let call = $state<Call | null>(null);
  let download = $state<Download | null>(null);
  // Equalizer bar heights from the audio meter, while it runs.
  let levels = $state<number[] | null>(null);
  // Horizontal swipes cycle which activity leads.
  let turn = $state(0);
  let clipItems = $state<ClipEntry[]>([]);
  // Long-pressed open: stays open until long-pressed again or a click elsewhere.
  let pinned = $state(false);
  // Camera check in the notch, optionally for a meeting.
  let mirror = $state<{ title: string; link: string } | null>(null);
  // The wheel is changing the volume right now.
  let scrollingVolume = $state(false);
  let scrollVolumeTimer: ReturnType<typeof setTimeout>;

  // Status peeks: a few seconds each; volume/brightness drags keep extending it.
  // Notifications queue up instead of replacing each other, and a peek you can
  // click (notification, screenshot) stays put while the cursor is on it.
  let peekTimer: ReturnType<typeof setTimeout>;
  let peekQueue: PeekData[] = [];
  const clickable = (p: PeekData | null) =>
    ["notification", "screenshot", "update", "download-done", "meeting"].includes(p?.kind ?? "");
  const peekLength = (p: PeekData) =>
    p.kind === "meeting"
      ? 30_000
      : ["screenshot", "update", "download-done"].includes(p.kind)
        ? 6000
        : p.kind === "notification"
          ? peekQueue.length
            ? 3200
            : 4500
          : ["todo", "calendar", "update-info", "earbuds-low"].includes(p.kind)
            ? 5000
            : 2200;
  function showPeek(data: PeekData) {
    if (data.kind === "notification" && peek?.kind === "notification") {
      peekQueue.push(data);
      return;
    }
    peek = data;
    clearTimeout(peekTimer);
    if (!(clickable(data) && hovered)) peekTimer = setTimeout(endPeek, peekLength(data));
  }
  function endPeek() {
    peek = null;
    const next = peekQueue.shift();
    if (next) setTimeout(() => showPeek(next), 120);
  }
  function openFromPeek(p: PeekData) {
    if (p.kind === "notification" && p.aumid) invoke("open_app", { aumid: p.aumid, id: p.id });
    if (p.kind === "update") invoke("install_update");
    if (p.kind === "download-done") invoke("open_download", { path: p.path });
    clearTimeout(peekTimer);
    endPeek();
  }
  // Buttons inside a peek (the meeting reminder's "Cek kamera" and "Join").
  function peekAction(p: PeekData, action: string) {
    if (action === "camera") openMirror(p.title, p.link ?? "");
    if (action === "join" && p.link) invoke("run_link", { url: p.link });
    clearTimeout(peekTimer);
    endPeek();
  }

  function setPinned(on: boolean) {
    pinned = on;
    invoke("set_pinned", { on });
    if (!on && !hovered && !typing) closeTimer = setTimeout(() => (open = false), 250);
  }
  function openMirror(title = "", link = "") {
    mirror = { title, link };
    invoke("set_mirror", { on: true });
    setPinned(true);
    open = true;
  }
  function closeMirror() {
    mirror = null;
    setPinned(false);
    // The camera's in-use entry clears a moment after the stream stops.
    setTimeout(() => invoke("set_mirror", { on: false }), 3000);
  }
  function joinMeeting() {
    const link = mirror?.link;
    closeMirror();
    if (link) invoke("run_link", { url: link });
  }

  // Calendar: re-read every 15 minutes; remind 10 minutes before and at the start.
  let events = $state<CalEvent[]>([]);
  let calendarTimers: ReturnType<typeof setTimeout>[] = [];
  async function loadCalendar() {
    const ics = await invoke<string>("fetch_calendar").catch(() => "");
    events = ics ? upcoming(ics) : [];
    calendarTimers.forEach(clearTimeout);
    // Reminders 10 minutes before and at the start; for a video meeting also an
    // offer 2 minutes before to check the camera or join.
    calendarTimers = events
      .filter((e) => !e.allDay)
      .flatMap((e) =>
        (e.link ? [10, 2, 0] : [10, 0]).map((before) => {
          const at = e.start - before * 60_000;
          const peekData: PeekData =
            before === 2
              ? { kind: "meeting", title: e.title, start: e.start, link: e.link }
              : { kind: "calendar", title: e.title, start: e.start, before };
          return at > Date.now() && at - Date.now() < 2 * 24 * 3600_000
            ? setTimeout(() => showPeek(peekData), at - Date.now())
            : undefined;
        }),
      )
      .filter((t) => t !== undefined);
  }
  const nextEvent = $derived(events.find((e) => !e.allDay && e.end > now.getTime()));
  const clockOf = (t: number) => new Date(t).toLocaleTimeString("id-ID", { hour: "2-digit", minute: "2-digit" }).replace(".", ":");

  // Output device for the switch button in the open notch.
  type Output = { id: string; name: string; headphones: boolean; current: boolean };
  let output = $state<Output | null>(null);
  const refreshOutput = async () => (output = (await invoke<Output[]>("list_outputs")).find((o) => o.current) ?? null);
  $effect(() => {
    if (open) refreshOutput();
  });
  async function switchOutput() {
    await invoke("next_output");
    await refreshOutput();
  }

  const pomo = new Pomodoro(showPeek);
  const water = new Water(showPeek);
  const todo = new Todo(showPeek);
  const shelf = new Shelf();

  const charging = $derived(power.charging);
  let chargingHappy = $state(false);
  $effect(() => {
    if (!charging) {
      chargingHappy = false;
      return;
    }
    let stopped = false;
    let timer: ReturnType<typeof setTimeout>;
    const mood = () => {
      if (stopped) return;
      chargingHappy = Math.random() > 0.35;
      timer = setTimeout(mood, 1600 + Math.random() * 4200);
    };
    mood();
    return () => {
      stopped = true;
      clearTimeout(timer);
    };
  });
  const timerActive = $derived(pomo.running || pomo.left < pomo.length());
  // The notch is showing its pages (not a held notification or screenshot peek).
  const paged = $derived(open && !clickable(peek));
  // What runs right now, for the closed notch (the lyric flag is filled in below).
  const running = $derived.by((): Activity[] => [
    ...(call ? [{ kind: "call", call } as const] : []),
    ...(timerActive ? [{ kind: "timer" } as const] : []),
    ...(download ? [{ kind: "download", download } as const] : []),
    ...(media?.playing ? [{ kind: "music", lyric: false } as const] : []),
  ]);
  const view = $derived(
    mirror
      ? "mirror"
      : // Scrolling the volume on an open notch: show the volume, not the page.
        scrollingVolume && peek?.kind === "volume"
        ? "peek"
        : paged
        ? pages[page] === "now"
          ? media
            ? lyricsOn
              ? "lyrics"
              : "player"
            : "clock"
          : (pages[page] as Exclude<PageName, "now">)
        : peek
          ? peek.kind === "screenshot"
            ? "shot"
            : peek.kind === "notification" || peek.kind === "meeting" || peek.kind === "download-done"
              ? "notify"
              : "peek"
          : tucked && !download
            ? "tucked"
            : running.length
              ? "island"
              : charging
                ? "charging"
                : "rest",
  );
  // Lyrics: fetched for the lyrics panel, and (if enabled) for a single line in the small notch.
  let lyrics = $state<Lyrics | null>(null);
  let lyricsStatus = $state<"loading" | "done" | "offline">("loading");
  let smallLyrics = $state(true);
  const track = $derived(media && cleanTrack(media.title, media.artist, media.source));
  $effect(() => {
    if (!track || !(lyricsOn || smallLyrics)) return;
    const { title, artist } = track;
    let cancelled = false;
    lyrics = null;
    lyricsStatus = "loading";
    const { album, duration } = untrack(() => media!);
    findLyrics(title, artist, album, duration).then((found) => {
      if (cancelled) return;
      lyrics = found;
      lyricsStatus = found ? "done" : "offline";
    });
    return () => (cancelled = true);
  });

  // Browser players often publish only their own icon through GSMTC (or no
  // picture at all). Look the real cover up once per track, and keep it for
  // the player and the small island.
  let cover = $state<string | null>(null);
  let coverKey = "";
  $effect(() => {
    const m = media;
    const t = track;
    if (!m || !t) {
      cover = null;
      coverKey = "";
      return;
    }
    const browser = /chrome|msedge|edge|firefox|brave|opera|vivaldi|arc/i.test(m.source);
    if (m.art && !browser) {
      cover = null;
      coverKey = "";
      return;
    }
    const key = `${t.artist}|${t.title}|${m.album}`;
    if (key === coverKey) return;
    coverKey = key;
    cover = null;
    invoke<string | null>("cover_lookup", { title: t.title, artist: t.artist, album: m.album })
      .then((url) => {
        if (coverKey === key && url) cover = url;
      })
      .catch(() => {});
  });
  const art = $derived(cover ?? media?.art ?? null);
  const shownMedia = $derived(media ? { ...media, art } : null);

  // The line being sung, for the small notch. Instead of ticking, wake up
  // exactly when the next line starts.
  let lineNow = $state(Date.now());
  const smallLine = $derived.by(() => {
    if (!smallLyrics || !media?.playing || !lyrics?.synced) return "";
    const i = currentLine(lyrics.synced, positionAt(media, lineNow));
    // Instrumental gaps ("♪") count as no line, so the clock shows instead.
    return i >= 0 && lyrics.synced[i].text !== "♪" ? lyrics.synced[i].text : "";
  });
  // The island with the sung line filled in: music with words leads (lib/island.ts).
  const island = $derived(
    arrange(
      running.map((a) => (a.kind === "music" ? { kind: "music", lyric: !!smallLine } : a)),
      turn,
    ),
  );
  // Keeps the sung line current whenever music is in the island, also while
  // another activity leads: during an instrumental gap the line is empty, the
  // timer (say) takes the lead, and the next line must still bring music back.
  $effect(() => {
    if ((view !== "island" && view !== "clock") || !smallLyrics || !media?.playing || !lyrics?.synced) return;
    // A new song, a seek, a pause or coming back from another view re-runs this with
    // lineNow left behind, which would put the song near its start and leave the line
    // empty until the next wake-up. Catch up to the real time first (this re-runs it).
    if (Date.now() - lineNow > 250) {
      lineNow = Date.now();
      return;
    }
    const pos = positionAt(media, lineNow);
    const next = lyrics.synced.find((l) => l.time > pos + 0.25);
    if (!next) return;
    const id = setTimeout(() => (lineNow = Date.now()), (next.time - 0.25 - pos) * 1000 + 30);
    return () => clearTimeout(id);
  });

  // In the island the camera/mic dots sit at the end, so they get room of their own.
  const islandDots = $derived((inUse.camera.length > 0 ? 10 : 0) + (inUse.mic.length > 0 && !call ? 10 : 0));
  const cameraTitle = $derived(
    inUse.camera.length === 1 && inUse.camera[0] === "Kamera"
      ? "Kamera sedang digunakan"
      : `Kamera: ${inUse.camera.join(", ")}`
  );
  const islandSize = $derived(
    island.main
      ? {
          ...SIZES.island,
          w: Math.min(
            760,
            Math.max(
              islandWidth(island.main, island.side) + islandDots,
              // 7px is closer to the actual average width of the 12px lyric font;
              // the previous estimate left a visible empty tail after the visualizer.
              island.main.kind === "music" && smallLine ? smallLine.length * 7 + 82 + islandDots : 0,
            ),
          ),
        }
      : SIZES.island,
  );
  // The closed notch's normal footprint, including a running island. Tucking only
  // removes height, so the line stays exactly as wide as the notch it replaces.
  const normalClosedSize = $derived(running.length ? islandSize : charging ? SIZES.charging : SIZES.rest);
  const size = $derived(
    view === "tucked"
      ? { ...SIZES.tucked, w: normalClosedSize.w }
      : view === "island"
        ? islandSize
        : view === "clock" && smallLine
          ? { ...SIZES.clock, w: Math.min(760, Math.max(SIZES.clock.w, smallLine.length * 8 + 100)) }
          : SIZES[view],
  );

  // The audio meter runs only while the equalizer is on screen.
  const metering = $derived(view === "island" && !!media?.playing && island.main !== null && [island.main, ...island.side].some((a) => a.kind === "music"));
  $effect(() => {
    invoke("set_metering", { on: metering });
    if (!metering) levels = null;
  });

  // Tell Rust the target size right away, so the growing notch already counts as "inside".
  // Tucked, the notch is a thin line, so whatever is underneath (like browser tabs) stays clickable.
  $effect(() => {
    invoke("set_notch_size", { w: size.w, h: size.h });
  });

  // A clock only while something on screen moves: playback position or the
  // running timer (smooth when open, once a second in the small notch).
  let nowMs = $state(Date.now());
  $effect(() => {
    const every =
      open && (media?.playing || pomo.running) ? 250 : view === "island" && (pomo.running || call) ? 1000 : 0;
    if (!every) return;
    nowMs = Date.now();
    const id = setInterval(() => (nowMs = Date.now()), every);
    return () => clearInterval(id);
  });

  let closeTimer: ReturnType<typeof setTimeout>;
  function setTyping(on: boolean) {
    typing = on;
    invoke(on ? "begin_typing" : "end_typing");
    if (!on && !hovered) closeTimer = setTimeout(() => (open = false), 250);
  }

  function turnPage(step: number) {
    pageDir = step > 0 ? 1 : -1;
    page = (page + step + pages.length) % pages.length;
  }
  function jumpPage(index: number) {
    if (index === page) return;
    pageDir = index > page ? 1 : -1;
    page = index;
  }

  // Pages cross-fade while the old one slips away and the new one settles in
  // from the direction of travel. The small lift and defocus keep the swap
  // legible without the hard snap of a pure slide.
  function pageSlide(
    _node: HTMLElement,
    { dir, leaving }: { dir: number; leaving: boolean },
  ) {
    return {
      duration: leaving ? 220 : 360,
      delay: leaving ? 0 : 20,
      easing: cubicOut,
      css: (t: number, u: number) => {
        const shift = dir * 18 * u;
        const lift = (leaving ? -3 : 5) * u;
        const scale = 1 - 0.04 * u;
        const pointer = leaving ? "pointer-events:none;" : "";
        return `${pointer}opacity:${t};transform:translate3d(${shift}px, ${lift}px, 0) scale(${scale});filter:blur(${(1.8 * u).toFixed(3)}px);`;
      },
    };
  }

  let lastWheel = 0;
  let lastVolume = 0;
  function wheel(e: WheelEvent) {
    // Sideways (touchpad swipe): another activity leads the closed notch.
    if (Math.abs(e.deltaX) > Math.abs(e.deltaY)) {
      if (Date.now() - lastWheel < 450 || Math.abs(e.deltaX) < 8) return;
      lastWheel = Date.now();
      turn += e.deltaX > 0 ? 1 : -1;
      return;
    }
    // The band where the closed notch sits: volume, up = louder.
    if (e.offsetY >= 0 && (e.target as HTMLElement).closest(".notch") && e.clientY < 30 && !typing) {
      if (Date.now() - lastVolume < 40 || Math.abs(e.deltaY) < 1) return;
      lastVolume = Date.now();
      invoke("volume_step", { up: e.deltaY < 0 });
      scrollingVolume = true;
      clearTimeout(scrollVolumeTimer);
      scrollVolumeTimer = setTimeout(() => (scrollingVolume = false), 1200);
      return;
    }
    // Lists scroll themselves; elsewhere the wheel flips pages.
    if (!open || typing || (e.target as HTMLElement).closest(".scroll, .lyrics")) return;
    if (Date.now() - lastWheel < 350 || Math.abs(e.deltaY) < 4) return;
    lastWheel = Date.now();
    turnPage(e.deltaY > 0 ? 1 : -1);
  }

  // Long press (still, on a non-button spot) pins the notch open, or lets it go.
  let pressTimer: ReturnType<typeof setTimeout> | undefined;
  let pressAt: [number, number] = [0, 0];
  function pressDown(e: PointerEvent) {
    if (e.button !== 0 || (e.target as HTMLElement).closest("button, input, textarea, .item, video")) return;
    pressAt = [e.clientX, e.clientY];
    clearTimeout(pressTimer);
    pressTimer = setTimeout(() => {
      pressTimer = undefined;
      if (mirror) return;
      setPinned(!pinned);
      open = true;
    }, 600);
  }
  function pressMove(e: PointerEvent) {
    if (pressTimer && Math.hypot(e.clientX - pressAt[0], e.clientY - pressAt[1]) > 6) {
      clearTimeout(pressTimer);
      pressTimer = undefined;
    }
  }
  function pressUp() {
    clearTimeout(pressTimer);
    pressTimer = undefined;
  }

  onMount(() => {
    const unlisten = listen<boolean>("notch-hover", (e) => {
      hovered = e.payload;
      clearTimeout(closeTimer);
      if (e.payload) {
        open = true;
        wake();
      } else if (!typing && !pinned) closeTimer = setTimeout(() => (open = false), 250);
      // Hold a clickable peek while the cursor is on it; let it go shortly after.
      if (clickable(peek)) {
        clearTimeout(peekTimer);
        if (!e.payload) peekTimer = setTimeout(endPeek, 1500);
      }
    });

    // A click outside a pinned notch lets it close.
    const unlistenOutside = listen("notch-outside-click", () => {
      if (pinned && !mirror) setPinned(false);
    });

    // Files dragged onto the notch go to the shelf, or to Windows' Share window
    // when dropped on "Bagikan".
    const zoneAt = (p: { x: number; y: number }) => {
      const el = document.elementFromPoint(p.x / devicePixelRatio, p.y / devicePixelRatio);
      return (el?.closest("[data-drop]") as HTMLElement | null)?.dataset.drop ?? null;
    };
    const unlistenDrop = getCurrentWebviewWindow().onDragDropEvent((e) => {
      if (e.payload.type === "enter" || e.payload.type === "over") {
        dropping = true;
        page = pages.indexOf("shelf");
        dropTarget = zoneAt(e.payload.position);
      } else if (e.payload.type === "drop") {
        dropping = false;
        dropTarget = null;
        shelf.add(e.payload.paths);
      } else {
        dropping = false;
        dropTarget = null;
      }
    });

    // Activities and pages fed from Rust.
    const unlistenCall = listen<Call | null>("call", (e) => (call = e.payload));
    invoke<Call | null>("call_state").then((c) => (call = c));
    const unlistenDownload = listen<Download | null>("download", (e) => (download = e.payload));
    const unlistenLevel = listen<number[]>("audio-level", (e) => (levels = metering ? e.payload : null));
    const unlistenClip = listen<ClipEntry[]>("clipboard", (e) => (clipItems = e.payload));
    invoke<ClipEntry[]>("clipboard_list").then((c) => (clipItems = c));

    water.load();
    todo.load();
    shelf.load();
    // A quiet cursor steps the notch through: clock, awake eyes, then sleeping eyes.
    const IDLE_CLOCK_MS = 6_000;
    const IDLE_SLEEP_MS = 30_000;
    let idleTimer: ReturnType<typeof setTimeout>;
    let sleepTimer: ReturnType<typeof setTimeout>;
    let waveTimer: ReturnType<typeof setTimeout>;
    const scheduleSleep = () => {
      clearTimeout(sleepTimer);
      sleepTimer = setTimeout(() => {
        // Music or video keeps the eyes awake; sleep is only for a truly quiet machine.
        if (media?.playing) {
          scheduleSleep();
          return;
        }
        idleStage = "sleep";
        gaze = 0;
      }, IDLE_SLEEP_MS);
    };
    const scheduleClock = () => {
      clearTimeout(idleTimer);
      clearTimeout(sleepTimer);
      idleStage = "clock";
      gaze = 0;
      idleTimer = setTimeout(() => {
        idleStage = "awake";
        scheduleSleep();
      }, IDLE_CLOCK_MS);
    };
    const waveHello = () => {
      waving = true;
      clearTimeout(waveTimer);
      waveTimer = setTimeout(() => {
        waving = false;
        scheduleClock();
      }, 2200);
    };
    function wake() {
      if (idleStage === "sleep") {
        clearTimeout(idleTimer);
        clearTimeout(sleepTimer);
        idleStage = "awake";
        waveHello();
      } else if (idleStage === "awake") {
        scheduleSleep();
      } else {
        scheduleClock();
      }
    }
    scheduleClock();
    const unlistenLook = listen<[number, number, number]>("eyes-look", (e) => {
      look = e.payload;
      wake();
    });
    let gazeTimer: ReturnType<typeof setTimeout>;
    const glance = () => {
      if (idleStage === "awake") {
        const directions = [
          [-1, -0.55], [-0.65, 0.65], [0, -0.7], [0, 0],
          [0.65, 0.65], [1, -0.45], [-0.55, 0], [0.55, 0],
        ];
        const [x, y] = directions[Math.floor(Math.random() * directions.length)];
        gaze = x;
        gazeY = y;
        gazeTilt = Math.abs(y) > 0.5 && Math.random() > 0.55 ? (x < 0 ? -1 : 1) : 0;
      }
      gazeTimer = setTimeout(glance, 1_700 + Math.random() * 1_800);
    };
    glance();
    // Keyboard input and activity in other apps do not reach the mouse hook, so ask
    // Windows for the last input time. Recent input keeps the eyes awake.
    let idlePoll: ReturnType<typeof setInterval>;
    let lastIdle = 0;
    const watchInput = async () => {
      try {
        const idle = await invoke<number>("idle_seconds");
        // A drop means new keyboard or mouse input happened since the last poll.
        if (idle < 1 || idle + 0.35 < lastIdle) wake();
        lastIdle = idle;
      } catch {
        // Older builds do not expose this; the timer-based idle still works.
      }
    };
    idlePoll = setInterval(watchInput, 1_000);
    const unlistenTuck = listen<boolean>("notch-tucked", (e) => (tucked = e.payload));
    invoke<boolean>("is_tucked").then((t) => (tucked = t));

    const setMedia = (m: Omit<NowPlaying, "receivedAt"> | null) => {
      const wasPlaying = media?.playing ?? false;
      media = m && { ...m, receivedAt: Date.now() };
      lineNow = Date.now();
      if (!m) lyricsOn = false;
      if (media?.playing) {
        // A new song or video is activity: open the eyes and keep them from sleeping.
        if (idleStage === "sleep") wake();
        else if (idleStage === "awake") scheduleSleep();
      } else if (wasPlaying && idleStage === "awake") {
        scheduleSleep();
      }
    };
    const unlistenMedia = listen<Omit<NowPlaying, "receivedAt"> | null>("media", (e) => setMedia(e.payload));
    invoke<Omit<NowPlaying, "receivedAt"> | null>("media_now").then(setMedia);

    const unlistenPeek = listen<PeekData>("peek", (e) => {
      showPeek(e.payload);
      // New screenshots also land on the shelf.
      if (e.payload.kind === "screenshot") shelf.add([e.payload.path]);
    });

    const unlistenPower = listen<Power>("power", (e) => { power = e.payload; console.log('power event', e.payload); });
    invoke<Power>("power_state").then((p) => { power = p; console.log('initial power', p); });
    // Fallback polling in case Windows power notifications are not delivered reliably.
    const powerPoll = setInterval(() => {
        invoke<Power>("power_state").then((p) => (power = p));
    }, 30000); // every 30 seconds
    // Cleanup interval on unload
    const cleanupPowerPoll = () => clearInterval(powerPoll);

    const unlistenInUse = listen<InUse>("in-use", (e) => (inUse = e.payload));
    invoke<InUse>("in_use_state").then((u) => (inUse = u));
    const unlistenMic = listen<boolean>("mic-muted", (e) => (micMuted = e.payload));
    invoke<boolean>("mic_state").then((m) => (micMuted = m));

    // Settings, and weather every 30 minutes for the chosen city or else the Windows location.
    type AllSettings = Settings & {
      weatherCity: { latitude: number; longitude: number } | null;
      smallLyrics: boolean;
    };
    let weatherTimer: ReturnType<typeof setTimeout>;
    const loadWeather = async (settings?: AllSettings) => {
      clearTimeout(weatherTimer);
      settings ??= await invoke<AllSettings>("get_settings");
      const city = settings.weatherCity;
      const at = city ? [city.latitude, city.longitude] : await invoke<[number, number] | null>("locate");
      weather = at ? await fetchWeather(at[0], at[1]) : null;
      weatherTimer = setTimeout(loadWeather, 30 * 60_000);
    };
    const applySettings = (s: AllSettings) => {
      smallLyrics = s.smallLyrics;
      waterEnabled = s.waterEnabled !== false;
      pomo.configure(s);
      water.configure(s);
      loadWeather(s);
      loadCalendar();
    };
    const calendarPoll = setInterval(loadCalendar, 15 * 60_000);
    invoke<AllSettings>("get_settings").then(applySettings);
    const unlistenSettings = listen<AllSettings>("settings", (e) => applySettings(e.payload));

    // Tick once per minute, aligned to the minute, so idle stays idle.
    let tick: ReturnType<typeof setTimeout>;
    const schedule = () => {
      now = new Date();
      tick = setTimeout(schedule, 60_000 - (Date.now() % 60_000));
    };
    schedule();

    // Blink every few seconds. A one-shot class instead of an infinite CSS
    // animation, so nothing renders between blinks.
    let blinkTimer: ReturnType<typeof setTimeout>;
    let winkTimer: ReturnType<typeof setTimeout>;
    const scheduleBlink = () => {
      blinkTimer = setTimeout(() => {
        blink = true;
        setTimeout(() => (blink = false), 180);
        scheduleBlink();
      }, 3500 + Math.random() * 3000);
    };
    scheduleBlink();
    const scheduleWink = () => {
      winkTimer = setTimeout(() => {
        if (idleStage === "awake" && !blink) {
          wink = Math.random() > 0.5 ? "left" : "right";
          setTimeout(() => (wink = null), 150);
        }
        scheduleWink();
      }, 9000 + Math.random() * 7000);
    };
    scheduleWink();

    return () => {
      unlisten.then((f) => f());
      unlistenLook.then((f) => f());
      unlistenTuck.then((f) => f());
      unlistenMedia.then((f) => f());
      for (const u of [
        unlistenPeek,
        unlistenPower,
        unlistenInUse,
        unlistenMic,
        unlistenSettings,
        unlistenDrop,
        unlistenOutside,
        unlistenCall,
        unlistenDownload,
        unlistenLevel,
        unlistenClip,
      ])
        u.then((f) => f());
      clearTimeout(pressTimer);
      clearTimeout(peekTimer);
      clearTimeout(weatherTimer);
      clearInterval(calendarPoll);
      calendarTimers.forEach(clearTimeout);
      clearTimeout(closeTimer);
      clearTimeout(tick);
      clearTimeout(blinkTimer);
      clearTimeout(winkTimer);
      clearTimeout(sleepTimer);
      clearTimeout(idleTimer);
      clearTimeout(waveTimer);
      clearTimeout(gazeTimer);
      clearInterval(idlePoll);
    };
  });

  const time = $derived(now.toLocaleTimeString("id-ID", { hour: "2-digit", minute: "2-digit" }).replace(".", ":"));
  const date = $derived(now.toLocaleDateString("id-ID", { weekday: "long", day: "numeric", month: "long" }));
</script>

<div
  class="notch"
  role="group"
  aria-label="Ghyna"
  data-idle={idleStage}
  class:open={paged}
  class:tucked={view === "tucked"}
  style:--w="{size.w}px"
  style:--h="{size.h}px"
  style:--r="{size.r}px"
  style:--e="{size.e}px"
  onwheel={wheel}
  onpointerdown={pressDown}
  onpointermove={pressMove}
  onpointerup={pressUp}
  onpointerleave={pressUp}
>
  {#if pinned && paged && !mirror}<i class="pin" title="Terkunci terbuka · tekan lama untuk melepas"></i>{/if}

  <div class="layer rest" class:on={view === "rest"}>
    {#if weather && idleStage === "clock"}<span class="weather">{weather.icon}<b>{weather.temp}°</b></span>{/if}
    {#if idleStage !== "clock"}
      <div
        class="eyes"
        class:sleep={idleStage === "sleep"}
        class:blink={blink && idleStage === "awake"}
        class:wink-left={wink === "left"}
        class:wink-right={wink === "right"}
        class:tilt-left={gazeTilt < 0}
        class:tilt-right={gazeTilt > 0}
        class:charge={eyeState === "charging"}
        class:happy={eyeState === "charging" && chargingHappy}
        class:angry={eyeState === "low15"}
        style:--lx={eyeX}
        style:--ly={eyeY}
        style:--near={look[2]}
        style:--eye-grad={eyeState === "charging"
          ? "linear-gradient(to bottom, #ffffff, #38bdf8)"
          : eyeState === "low15"
          ? "linear-gradient(to bottom, #ffffff, #ff4d4d)"
          : eyeState === "low20"
          ? "linear-gradient(to bottom, #ffffff, #ff9f00)"
          : "linear-gradient(to bottom, #e3ffe6, #42c763)"}
        in:fade={{ duration: 400 }}
      >
        <i></i><i></i>
      </div>
      {#if waving}
        <span class="wave" in:fade={{ duration: 200 }} out:fade={{ duration: 160 }} aria-hidden="true">
          <svg viewBox="0 0 24 24">
            <path d="M8 12.6V6.5a1.4 1.4 0 0 1 2.8 0v4.4m0-5.3v-1a1.4 1.4 0 0 1 2.8 0v6m0-4.8a1.4 1.4 0 0 1 2.8 0v4.8m0-3.2a1.4 1.4 0 0 1 2.8 0v5.7c0 3.6-2.3 6-5.9 6-2.4 0-4-1.2-5.2-3.2l-1.7-2.9a1.4 1.4 0 0 1 2.4-1.4L8 13.3" />
          </svg>
        </span>
      {/if}
    {:else}
      <span class="rest-time" in:fade={{ duration: 400 }}>{time}</span>
    {/if}
  </div>

  <!-- Things running (music, timer, a call, a download), side by side. -->
  {#if island.main}
    <div class="layer" class:on={view === "island"}>
      <Island
        main={island.main}
        side={island.side}
        media={shownMedia ?? media}
        line={smallLine}
        {time}
        {pomo}
        now={nowMs}
        {levels}
        {micMuted}
        active={view === "island"}
        camera={inUse.camera.length > 0}
        mic={inUse.mic.length > 0 && !call}
      />
    </div>
  {/if}

  <!-- While charging: percentage left, expressive eyes centered, charging cue right. -->
  <div class="layer charge" class:on={view === "charging"}>
    <span class="charge-percent">{power.percent}%</span>
    {#if idleStage !== "clock"}
      <div
        class="eyes charge-eyes"
        class:happy={eyeState === "charging" && chargingHappy}
        class:angry={eyeState === "low15"}
          class:sleep={idleStage === "sleep"}
        class:blink={blink && idleStage === "awake"}
        class:wink-left={wink === "left"}
        class:wink-right={wink === "right"}
        class:tilt-left={gazeTilt < 0}
        class:tilt-right={gazeTilt > 0}
        style:--lx={eyeX}
        style:--ly={eyeY}
        style:--near={look[2]}
        style:--eye-grad={eyeState === "charging"
          ? "linear-gradient(to bottom, #ffffff, #38bdf8)"
          : eyeState === "low15"
          ? "linear-gradient(to bottom, #ffffff, #ff4d4d)"
          : eyeState === "low20"
          ? "linear-gradient(to bottom, #ffffff, #ff9f00)"
          : "linear-gradient(to bottom, #e3ffe6, #42c763)"}
        in:fade={{ duration: 300 }}
      >
        <i></i><i></i>
      </div>
      {#if waving}
        <span class="wave charge-wave" in:fade={{ duration: 200 }} out:fade={{ duration: 160 }} aria-hidden="true">
          <svg viewBox="0 0 24 24">
            <path d="M8 12.6V6.5a1.4 1.4 0 0 1 2.8 0v4.4m0-5.3v-1a1.4 1.4 0 0 1 2.8 0v6m0-4.8a1.4 1.4 0 0 1 2.8 0v4.8m0-3.2a1.4 1.4 0 0 1 2.8 0v5.7c0 3.6-2.3 6-5.9 6-2.4 0-4-1.2-5.2-3.2l-1.7-2.9a1.4 1.4 0 0 1 2.4-1.4L8 13.3" />
          </svg>
        </span>
      {/if}
    {:else}
      <span class="charge-time">{time}</span>
    {/if}
    <span class="charge-status" aria-label="Sedang mengisi daya">⚡︎</span>
  </div>

  <div class="layer clock" class:on={view === "clock"}>
    <div class="time">{time}</div>
    <div class="date">{date}{#if weather} · {weather.icon} {weather.temp}°{/if}</div>
    {#if smallLyrics && media?.playing && lyrics?.synced && smallLine}
      <div class="clock-lyric" in:fade={{ duration: 300 }}>{smallLine}</div>
    {/if}
    {#if nextEvent}
      <div class="event" title={nextEvent.location}>
        <span class="event-time">{clockOf(nextEvent.start)}</span>{nextEvent.title}
      </div>
    {/if}
    {#if waving}
      <span class="wave clock-wave" in:fade={{ duration: 200 }} out:fade={{ duration: 160 }} aria-hidden="true">
        <svg viewBox="0 0 24 24">
          <path d="M8 12.6V6.5a1.4 1.4 0 0 1 2.8 0v4.4m0-5.3v-1a1.4 1.4 0 0 1 2.8 0v6m0-4.8a1.4 1.4 0 0 1 2.8 0v4.8m0-3.2a1.4 1.4 0 0 1 2.8 0v5.7c0 3.6-2.3 6-5.9 6-2.4 0-4-1.2-5.2-3.2l-1.7-2.9a1.4 1.4 0 0 1 2.4-1.4L8 13.3" />
        </svg>
      </span>
    {/if}
  </div>

  {#if peek && (view === "peek" || view === "shot" || view === "notify")}
    <div class="layer on" in:fade={{ duration: 220, delay: 120 }} out:fade={{ duration: 100 }}>
      <Peek {peek} onOpen={() => peek && openFromPeek(peek)} onAction={(a) => peek && peekAction(peek, a)} />
    </div>
  {/if}

  {#if mirror}
    <div class="layer on" in:fade={{ duration: 260, delay: 160 }} out:fade={{ duration: 100 }}>
      <Mirror title={mirror.title} link={mirror.link} onDone={closeMirror} onJoin={joinMeeting} />
    </div>
  {/if}

  <!-- Camera/mic in use, and a muted mic: visible on every view that doesn't
       already carry its own indicator (the island and the clock do). -->
  {#if view !== "island" && view !== "clock" && view !== "peek" && view !== "shot" && view !== "notify" && !mirror && (inUse.camera.length || inUse.mic.length || micMuted)}
    <div
      class="dots"
      class:wide={view === "charging"}
      class:open={paged && !mirror}
      class:tucked={view === "tucked"}
      class:pinspace={pinned && paged && !mirror}
    >
      {#if inUse.camera.length}<i class="cam" title={cameraTitle}></i>{/if}
      {#if micMuted}
        <svg class="muted" viewBox="0 0 24 24"><path d="M9 9v3a3 3 0 0 0 5.1 2.1M15 9.3V6a3 3 0 0 0-5.7-1.3M19 11a7 7 0 0 1-1.2 3.9M5 11a7 7 0 0 0 10.6 6M12 18v3M3 3l18 18" /></svg>
      {:else if inUse.mic.length}<i class="mic" title="Mikrofon: {inUse.mic.join(', ')}"></i>{/if}
    </div>
  {/if}

  {#if view === "clock"}
    <button
      class="mic-toggle"
      class:muted={micMuted}
      class:live={inUse.mic.length > 0}
      onclick={() => invoke("toggle_mic")}
      title={micMuted ? "Nyalakan mikrofon" : "Matikan mikrofon"}
      aria-label={micMuted ? "Nyalakan mikrofon" : "Matikan mikrofon"}
      in:fade={{ duration: 200, delay: 200 }}
    >
      <svg viewBox="0 0 24 24">
        {#if micMuted}
          <path d="M9 9v3a3 3 0 0 0 5.1 2.1M15 9.3V6a3 3 0 0 0-5.7-1.3M19 11a7 7 0 0 1-1.2 3.9M5 11a7 7 0 0 0 10.6 6M12 18v3M3 3l18 18" />
        {:else}
          <path d="M12 3a3 3 0 0 0-3 3v6a3 3 0 0 0 6 0V6a3 3 0 0 0-3-3zM19 11a7 7 0 0 1-14 0M12 18v3" />
        {/if}
      </svg>
    </button>
    {#if output}
      <button
        class="mic-toggle output"
        onclick={switchOutput}
        title="Audio: {output.name} (klik untuk ganti)"
        aria-label="Ganti output audio, sekarang {output.name}"
        in:fade={{ duration: 200, delay: 200 }}
      >
        <svg viewBox="0 0 24 24">
          {#if output.headphones}
            <path d="M3 14v-2a9 9 0 0 1 18 0v2M3 14h4v6H3zM17 14h4v6h-4z" />
          {:else}
            <path d="M11 5 6 9H3v6h3l5 4zM15.5 8.5a5 5 0 0 1 0 7M18.5 5.5a9 9 0 0 1 0 13" />
          {/if}
        </svg>
      </button>
    {/if}
    <button
      class="mic-toggle camera"
      onclick={() => openMirror(nextEvent?.link ? nextEvent.title : "", nextEvent?.link ?? "")}
      title="Cek kamera"
      aria-label="Cek kamera"
      in:fade={{ duration: 200, delay: 200 }}
    >
      <svg viewBox="0 0 24 24"><path d="M15 10l5-3v10l-5-3zM3 7h12v10H3z" /></svg>
      {#if inUse.camera.length}<i class="cam" title={cameraTitle}></i>{/if}
    </button>
  {/if}

  {#if media && (view === "player" || view === "lyrics")}
    <div class="layer on" in:fade={{ duration: 300, delay: 180 }} out:fade={{ duration: 120 }}>
      <Player
        media={shownMedia ?? media}
        now={nowMs}
        clock="{date} · {time}"
        withLyrics={lyricsOn}
        {lyrics}
        status={lyricsStatus}
        onLyrics={() => (lyricsOn = !lyricsOn)}
      />
    </div>
  {/if}

  {#if paged && !mirror && view !== "peek" && pages[page] !== "now"}
    {#key page}
      <div
        class="layer on page-layer"
        in:pageSlide={{ dir: pageDir, leaving: false }}
        out:pageSlide={{ dir: -pageDir, leaving: true }}
      >
        {#if pages[page] === "pomodoro"}
          <TimerPage {pomo} now={nowMs} />
        {:else if pages[page] === "water"}
          <WaterPage {water} enabled={waterEnabled} onToggle={toggleWater} />
        {:else if pages[page] === "todo"}
          <TodoPage {todo} onTyping={setTyping} />
        {:else if pages[page] === "clipboard"}
          <ClipboardPage items={clipItems} />
        {:else}
          <ShelfPage {shelf} {dropping} />
        {/if}
      </div>
    {/key}
  {/if}

  {#if paged && !mirror && view !== "lyrics" && view !== "peek"}
    {#each [-1, 1] as step}
      <button
        class="arrow"
        class:next={step > 0}
        onclick={() => turnPage(step)}
        aria-label={step > 0 ? "Halaman berikutnya" : "Halaman sebelumnya"}
        in:fade={{ duration: 200, delay: 200 }}
      >
        <svg viewBox="0 0 24 24"><path d={step > 0 ? "M9 6l6 6-6 6" : "M15 6l-6 6 6 6"} /></svg>
      </button>
    {/each}
    <div class="pager" role="tablist" in:fade={{ duration: 200, delay: 200 }}>
      {#each pages as name, i}
        <button role="tab" aria-selected={i === page} aria-label={name} class:on={i === page} onclick={() => jumpPage(i)}></button>
      {/each}
    </div>
  {/if}
</div>

<style>
  /* Register the ear radius so it can animate in step with the notch body.
     Without this, --e snaps to its next value while the ear boxes are still
     mid-transition, leaving detached black tabs at the top corners. */
  @property --e {
    syntax: "<length>";
    inherits: true;
    initial-value: 10px;
  }

  :global(html, body) {
    margin: 0;
    background: transparent;
    overflow: hidden;
    user-select: none;
    cursor: default;
    font-family: "Segoe UI Variable Text", "Segoe UI", sans-serif;
    color: var(--text);
  }

  .notch {
    --spring: cubic-bezier(0.22, 1, 0.36, 1);
    /* A green-black surface: green is the material, not a decoration. A
       hairline edge keeps it visible on a dark wallpaper. */
    --edge: var(--line);
    position: absolute;
    top: 0;
    left: 50%;
    width: var(--w);
    height: var(--h);
    transform: translateX(-50%);
    background: var(--surface);
    border-radius: 0 0 var(--r) var(--r);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.07),
      0 0 0 0.5px var(--edge),
      0 12px 30px -18px rgba(0, 0, 0, 0.95);
  }
  .notch.open {
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.1),
      0 0 0 0.5px var(--edge),
      0 16px 36px -18px rgba(0, 0, 0, 0.98);
  }

  /* "Ears": concave fillets that blend the notch into the screen's top edge. */
  .notch::before,
  .notch::after {
    content: "";
    position: absolute;
    top: 0;
    width: var(--e);
    height: var(--e);
  }
  /* The ears carry the hairline along their curve, covering the straight edge beside them. */
  .notch::before {
    right: 100%;
    background: radial-gradient(
      circle at 0 100%,
      transparent calc(var(--e) - 0.5px),
      var(--edge) calc(var(--e) - 0.5px),
      var(--edge) var(--e),
      var(--surface) calc(var(--e) + 0.3px)
    );
  }
  .notch::after {
    left: 100%;
    background: radial-gradient(
      circle at 100% 100%,
      transparent calc(var(--e) - 0.5px),
      var(--edge) calc(var(--e) - 0.5px),
      var(--edge) var(--e),
      var(--surface) calc(var(--e) + 0.3px)
    );
  }

  .notch {
    transition:
      width 0.62s var(--spring),
      height 0.62s var(--spring),
      --e 0.45s ease,
      border-radius 0.45s ease,
      box-shadow 0.45s ease;
  }

  /* One layer per view; only the current one is visible. */
  .layer {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.12s;
  }
  .layer.on {
    opacity: 1;
    pointer-events: auto;
    transition: opacity 0.3s 0.15s;
  }

  .rest-time {
    font-size: 14px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .weather {
    position: absolute;
    left: 12px;
    display: flex;
    align-items: center;
    gap: 3px;
    font-size: 10px;
  }
  .weather b {
    font-size: 11px;
    font-weight: 600;
  }
  /* The page content settles in from the direction you are travelling. */
  .page-layer {
    will-change: transform, opacity, filter;
  }

  .charge {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    gap: 0;
    padding: 0 18px;
  }
  .charge-time {
    font-size: 15px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  /* Previous / next page, on the notch's sides. */
  .arrow {
    position: absolute;
    top: 50%;
    left: 10px;
    width: 28px;
    height: 28px;
    margin-top: -14px;
    padding: 0;
    border: 0;
    border-radius: 8px;
    display: grid;
    place-items: center;
    background: transparent;
    cursor: pointer;
    opacity: 0.52;
    transition:
      opacity 0.18s,
      background 0.18s,
      transform 0.18s;
  }
  .arrow.next {
    left: auto;
    right: 10px;
  }
  .arrow:hover {
    opacity: 1;
    background: rgba(255, 255, 255, 0.07);
  }
  .arrow:active {
    transform: scale(0.94);
  }
  .arrow svg {
    width: 16px;
    height: 16px;
    fill: none;
    stroke: var(--text);
    stroke-width: 1.9;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  /* Page dots at the bottom of the open notch, the current one stretched. */
  .pager {
    position: absolute;
    z-index: 6;
    bottom: 4px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 1px;
  }
  .pager button {
    position: relative;
    width: 12px;
    height: 12px;
    padding: 0;
    border: 0;
    background: transparent;
    cursor: pointer;
  }
  .pager button::after {
    content: "";
    position: absolute;
    top: 50%;
    left: 50%;
    width: 4px;
    height: 4px;
    border-radius: 2px;
    background: var(--muted);
    transform: translate(-50%, -50%);
    transition:
      width 0.24s ease,
      background 0.2s;
  }
  .pager button:hover::after {
    background: var(--muted-strong);
  }
  .pager button.on::after {
    width: 12px;
    background: var(--accent);
  }

  /* Camera / mic indicators on the notch. */
  .dots {
    position: absolute;
    top: 0;
    right: 12px;
    height: 30px;
    display: flex;
    align-items: center;
    gap: 4px;
    z-index: 3;
  }
  .dots.wide {
    right: 38px;
  }
  /* The open notch: tuck the dots into the top-right padding, clear of the
     pin and of the page controls. */
  .dots.open {
    top: 7px;
    right: 24px;
    height: 18px;
  }
  .dots.open.pinspace {
    right: 34px;
  }
  /* Tucked, the notch is only 6px tall: a 4px dot is the whole indicator. */
  .dots.tucked {
    top: 1px;
    right: 6px;
    height: 6px;
  }
  .cam,
  .mic {
    width: 7px;
    height: 7px;
    border-radius: 50%;
  }
  .cam {
    background: var(--accent);
    box-shadow: 0 0 6px var(--accent);
    animation: cam-pulse 2.2s ease-in-out infinite;
  }
  .mic {
    background: var(--warning);
    box-shadow: 0 0 6px var(--warning);
  }
  .dots.tucked .cam,
  .dots.tucked .mic {
    width: 4px;
    height: 4px;
  }
  .dots.tucked .muted {
    width: 6px;
    height: 6px;
  }
  @keyframes cam-pulse {
    0%,
    100% {
      box-shadow:
        0 0 6px var(--accent),
        0 0 0 0 var(--accent-line);
    }
    50% {
      box-shadow:
        0 0 6px var(--accent),
        0 0 0 3px transparent;
    }
  }
  .dots .muted {
    width: 13px;
    height: 13px;
  }
  .muted,
  .mic-toggle svg {
    fill: none;
    stroke: var(--danger);
    stroke-width: 2.2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .mic-toggle {
    position: absolute;
    top: 10px;
    left: 18px;
    width: 26px;
    height: 26px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: rgba(239, 250, 243, 0.08);
    cursor: pointer;
  }
  .mic-toggle:hover {
    background: rgba(239, 250, 243, 0.14);
  }
  .mic-toggle svg {
    width: 14px;
    height: 14px;
    stroke: var(--text);
  }
  .mic-toggle.live svg {
    stroke: var(--warning);
  }
  .mic-toggle.muted {
    background: rgba(240, 128, 128, 0.18);
  }
  .mic-toggle.muted svg {
    stroke: var(--danger);
  }
  .mic-toggle.output {
    left: 50px;
  }
  .mic-toggle.camera {
    left: auto;
    right: 18px;
  }

  .event {
    max-width: 300px;
    margin-top: 6px;
    padding: 3px 10px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.06);
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .event-time {
    margin-right: 6px;
    color: var(--text);
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .mic-toggle .cam {
    position: absolute;
    top: 0;
    right: -2px;
    width: 6px;
    height: 6px;
  }

  /* Pinned open (long press): a small mark in the corner. */
  .pin {
    position: absolute;
    top: 9px;
    right: 14px;
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--muted-strong);
    z-index: 2;
  }

  .eyes {
    display: flex;
    gap: 15px;
    /* Glance toward the cursor (vertical travel is smaller because the notch is short),
       and open up a little, curious, as the cursor comes close. */
    transform: translate(calc(var(--lx, 0) * 9px), calc(var(--ly, 0) * 4px)) scale(calc(1 + var(--near, 0) * 0.25));
    transition: transform 0.22s var(--spring);
  }
  .eyes i {
    width: 8.4px;
    height: 12.4px;
    border-radius: 50%;
    background: var(--eye-grad, #fff);
    box-shadow:
      0 0 6px 1px rgba(255, 255, 255, 0.45),
      0 0 14px 4px rgba(255, 255, 255, 0.12);
    transition: transform 0.09s ease;
  }
  .eyes.charge i {
    animation: pulse-blue 1.25s infinite ease-in-out;
  }
  .charge-percent {
    min-width: 31px;
    color: #38bdf8;
    font-size: 12px;
    font-weight: 800;
    font-variant-numeric: tabular-nums;
    text-align: left;
    justify-self: start;
  }
  .charge-status {
    margin-left: 0;
    color: #38bdf8;
    font-size: 13px;
    line-height: 1;
    filter: drop-shadow(0 0 5px rgba(56, 189, 248, 0.65));
    animation: charging-spark 1.4s ease-in-out infinite;
    justify-self: end;
  }
  @keyframes charging-spark {
    0%, 100% { opacity: 0.5; transform: scale(0.9); }
    50% { opacity: 1; transform: scale(1.12); }
  }
  .eyes.happy i {
    height: 8px;
    border-radius: 10px 10px 3px 3px;
    transform: translateY(2px) rotate(0deg);
  }
  .eyes.happy i:first-child {
    transform: translateY(2px) rotate(-8deg);
  }
  .eyes.happy i:last-child {
    transform: translateY(2px) rotate(8deg);
  }
  /* Critical battery: narrowed eyes with an inward brow angle. */
  .eyes.angry i {
    height: 9.5px;
    border-radius: 3px 3px 50% 50%;
    transform: translateY(2px) rotate(12deg);
  }
  .eyes.angry i:first-child {
    transform: translateY(2px) rotate(20deg);
  }
  .eyes.angry i:last-child {
    transform: translateY(2px) rotate(-20deg);
  }
  @keyframes pulse-blue {
    0%, 100% {
      box-shadow:
        0 0 8px 2px rgba(56, 189, 248, 0.6),
        0 0 16px 5px rgba(56, 189, 248, 0.25);
      filter: brightness(1);
    }
    50% {
      box-shadow:
        0 0 14px 4px rgba(56, 189, 248, 0.95),
        0 0 24px 8px rgba(56, 189, 248, 0.45);
      filter: brightness(1.35);
    }
  }
  .eyes.blink i {
    transform: scaleY(0.1);
  }
  .eyes.wink-left i:first-child,
  .eyes.wink-right i:last-child {
    transform: scaleY(0.1);
  }
  .eyes.tilt-left {
    transform: translate(calc(var(--lx, 0) * 9px), calc(var(--ly, 0) * 4px)) rotate(-6deg) scale(calc(1 + var(--near, 0) * 0.25));
  }
  .eyes.tilt-right {
    transform: translate(calc(var(--lx, 0) * 9px), calc(var(--ly, 0) * 4px)) rotate(6deg) scale(calc(1 + var(--near, 0) * 0.25));
  }
  .eyes.sleep {
    transform: none;
    animation: sleep-breathe 4.5s ease-in-out infinite;
  }
  .eyes.sleep i {
    height: 2.8px;
    border-radius: 2px;
    opacity: 0.58;
    transform: translateY(4px);
    box-shadow: 0 0 4px rgba(255, 255, 255, 0.22);
    transition:
      height 0.35s var(--spring),
      opacity 0.35s ease,
      transform 0.35s var(--spring);
  }

  .wave {
    position: absolute;
    right: 8px;
    bottom: 4px;
    z-index: 4;
    transform-origin: 64% 86%;
    filter: drop-shadow(0 0 6px rgba(255, 255, 255, 0.25));
    animation: wave 0.7s ease-in-out 3;
  }
  .wave svg {
    display: block;
    width: 18px;
    height: 18px;
    fill: none;
    stroke: #fff;
    stroke-width: 1.55;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .charge-wave {
    right: 9px;
    bottom: 5px;
  }
  .clock-wave {
    right: 20px;
    bottom: 18px;
    opacity: 0.9;
  }
  @keyframes wave {
    0%,
    100% {
      transform: rotate(-10deg) translateY(2px);
    }
    22% {
      transform: rotate(17deg) translateY(0);
    }
    46% {
      transform: rotate(-14deg) translateY(1px);
    }
    70% {
      transform: rotate(12deg) translateY(0);
    }
  }
  @keyframes sleep-breathe {
    0%,
    100% {
      opacity: 0.72;
      transform: translateY(0.5px) scale(0.98);
    }
    50% {
      opacity: 1;
      transform: translateY(0) scale(1.02);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .wave,
    .eyes.sleep,
    .cam {
      animation: none;
    }
    .eyes,
    .eyes i {
      transition: none;
    }
  }

  .clock {
    transform: translateY(-6px) scale(0.96);
  }
  .clock.on {
    transform: none;
    transition:
      opacity 0.32s 0.2s,
      transform 0.5s 0.15s var(--spring);
  }

  .time {
    font-family: "Segoe UI Variable Display", "Segoe UI", sans-serif;
    font-size: 42px;
    font-weight: 700;
    letter-spacing: -0.03em;
    font-variant-numeric: tabular-nums;
  }
  .date {
    margin-top: 2px;
    font-size: 13px;
    color: rgba(255, 255, 255, 0.6);
  }
  .clock-lyric {
    max-width: none;
    margin-top: 5px;
    padding: 2px 9px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.07);
    font-size: 11.5px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.8);
    white-space: nowrap;
    overflow: visible;
    text-overflow: clip;
  }
</style>
