<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { searchCity, type Place } from "$lib/weather";

  type City = { name: string; latitude: number; longitude: number };
  type Settings = {
    weatherCity: City | null;
    focusMinutes: number;
    breakMinutes: number;
    glassMl: number;
    goalMl: number;
    waterFrom: number;
    waterTo: number;
    remindEvery: number;
    waterEnabled: boolean;
    mirrorNotifications: boolean;
    notificationFilter: boolean;
    notificationApps: string[];
    calendarUrl: string;
    smallLyrics: boolean;
  };
  type NotificationApp = { aumid: string; name: string };

  let settings = $state<Settings | null>(null);
  let query = $state("");
  let results = $state<Place[]>([]);
  let searching = $state(false);
  let saved = $state(false);
  let apps = $state<NotificationApp[]>([]);
  let appsLoading = $state(false);
  let appsError = $state("");

  // The settings window is a Tauri webview; opening the page in a plain browser
  // (for design previews) has no backend, so fall back to representative data.
  const preview = typeof window !== "undefined" && new URLSearchParams(window.location.search).has("preview");
  const demo: Settings = {
    weatherCity: { name: "Bandung", latitude: -6.9175, longitude: 107.6191 },
    focusMinutes: 25,
    breakMinutes: 5,
    glassMl: 250,
    goalMl: 2000,
    waterFrom: 8,
    waterTo: 21,
    remindEvery: 60,
    waterEnabled: true,
    mirrorNotifications: true,
    notificationFilter: true,
    notificationApps: ["5319275A.WhatsAppDesktop_cv1g1gvanyjgm!App", "SpotifyAB.SpotifyMusic_zpdnekdrzrea0!Spotify", "com.squirrel.Discord.Discord"],
    calendarUrl: "",
    smallLyrics: true,
  };
  const demoApps: NotificationApp[] = [
    { aumid: "5319275A.WhatsAppDesktop_cv1g1gvanyjgm!App", name: "WhatsApp" },
    { aumid: "SpotifyAB.SpotifyMusic_zpdnekdrzrea0!Spotify", name: "Spotify" },
    { aumid: "com.squirrel.Discord.Discord", name: "Discord" },
    { aumid: "com.microsoft.teams.Teams", name: "Microsoft Teams" },
    { aumid: "MSTeams_8wekyb3d8bbwe!MSTeams", name: "Microsoft Teams (new)" },
    { aumid: "com.slack.Slack", name: "Slack" },
  ];

  onMount(async () => {
    try {
      settings = await invoke<Settings>("get_settings");
    } catch (e) {
      if (!preview) throw e;
      settings = demo;
      apps = demoApps;
      return;
    }
    await refreshApps();
  });

  async function refreshApps() {
    appsLoading = true;
    appsError = "";
    try {
      apps = await invoke<NotificationApp[]>("notification_apps");
    } catch (e) {
      appsError = String(e);
    } finally {
      appsLoading = false;
    }
  }

  async function toggleFilter(on: boolean) {
    if (!settings) return;
    const patch: Partial<Settings> = { notificationFilter: on };
    // Start from "everything allowed" so turning the filter on doesn't hide
    // every notification until the user picks something.
    if (on && settings.notificationApps.length === 0 && apps.length > 0) {
      patch.notificationApps = apps.map((app) => app.aumid);
    }
    await save(patch);
  }

  function toggleApp(aumid: string, on: boolean) {
    if (!settings) return;
    const selected = new Set(settings.notificationApps);
    if (on) selected.add(aumid);
    else selected.delete(aumid);
    save({ notificationApps: [...selected] });
  }

  function selectAllApps() {
    if (!settings) return;
    save({ notificationApps: apps.map((app) => app.aumid) });
  }

  function clearApps() {
    save({ notificationApps: [] });
  }

  async function search(e: SubmitEvent) {
    e.preventDefault();
    if (!query.trim()) return;
    searching = true;
    results = await searchCity(query.trim()).catch(() => []);
    searching = false;
  }

  async function save(patch: Partial<Settings> = {}) {
    if (!settings) return;
    settings = { ...settings, ...patch };
    results = [];
    query = "";
    saved = true;
    setTimeout(() => (saved = false), 1500);
    if (preview) return;
    await invoke("save_settings", { settings });
  }

  // Number fields save when they lose focus.
  const numbers: [keyof Settings, string, string][] = [
    ["focusMinutes", "Fokus", "menit"],
    ["breakMinutes", "Istirahat", "menit"],
  ];
  const water: [keyof Settings, string, string][] = [
    ["glassMl", "Sekali minum", "ml"],
    ["goalMl", "Target harian", "ml"],
    ["waterFrom", "Ingatkan mulai", "jam"],
    ["waterTo", "Sampai", "jam"],
    ["remindEvery", "Jeda pengingat", "menit"],
  ];
  const selectedCount = $derived(settings?.notificationApps.length ?? 0);
</script>

{#snippet field(key: keyof Settings, label: string, unit: string)}
  {#if settings}
    <label class="row">
      <span class="label">{label}</span>
      <span class="num">
        <input
          type="number"
          min="0"
          value={settings[key] as number}
          onchange={(e) => save({ [key]: Math.max(0, Math.round(Number(e.currentTarget.value))) })}
        />
        {#if unit}<em>{unit}</em>{/if}
      </span>
    </label>
  {/if}
{/snippet}

{#snippet icon(path: string)}
  <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
    <path d={path} />
  </svg>
{/snippet}

<div class="page">
  <header>
    <div class="brand">
      <h1>Ghyna</h1>
      <span>Pengaturan</span>
    </div>
    <span class="saved" class:on={saved}>Tersimpan</span>
  </header>

  <main>
    <section class="card">
      <div class="head">{@render icon("M18 8a6 6 0 1 0-12 0c0 7-3 8-3 8h18s-3-1-3-8M13.7 21a2 2 0 0 1-3.4 0")}<h2>Notifikasi</h2></div>
      <div class="body">
        {#if settings}
          <label class="row">
            <span class="label">
              Tampilkan di notch
              <small>WhatsApp, Teams, email, dan aplikasi lain</small>
            </span>
            <input
              class="switch"
              type="checkbox"
              checked={settings.mirrorNotifications}
              onchange={(e) => save({ mirrorNotifications: e.currentTarget.checked })}
            />
          </label>
          {#if settings.mirrorNotifications}
            <label class="row">
              <span class="label">
                Hanya aplikasi terpilih
                <small>Abaikan aplikasi lain</small>
              </span>
              <input
                class="switch"
                type="checkbox"
                checked={settings.notificationFilter}
                onchange={(e) => toggleFilter(e.currentTarget.checked)}
              />
            </label>
            {#if settings.notificationFilter}
              <div class="picker">
                {#if appsLoading}
                  <p class="note">Memuat daftar aplikasi…</p>
                {:else if appsError}
                  <p class="note">Gagal memuat daftar aplikasi: {appsError}</p>
                {:else if apps.length === 0}
                  <p class="note">
                    Belum ada aplikasi tercatat. Munculkan satu notifikasi dari aplikasi yang ingin
                    dipilih, lalu tekan Muat ulang.
                  </p>
                {:else}
                  <div class="picker-bar">
                    <span class="count">{selectedCount}/{apps.length} dipilih</span>
                    <div class="links">
                      <button class="link" onclick={selectAllApps}>Semua</button>
                      <button class="link" onclick={clearApps}>Kosongkan</button>
                      <button class="link" onclick={refreshApps}>Muat ulang</button>
                    </div>
                  </div>
                  <div class="app-list">
                    {#each apps as app (app.aumid)}
                      <label class="app">
                        <input
                          type="checkbox"
                          checked={settings.notificationApps.includes(app.aumid)}
                          onchange={(e) => toggleApp(app.aumid, e.currentTarget.checked)}
                        />
                        <span class="app-name">{app.name}</span>
                        {#if app.name !== app.aumid}<span class="app-id">{app.aumid}</span>{/if}
                      </label>
                    {/each}
                  </div>
                  {#if selectedCount === 0}
                    <p class="note warn">Tidak ada aplikasi dipilih — notifikasi tidak akan tampil.</p>
                  {/if}
                {/if}
              </div>
            {/if}
          {/if}
          <div class="tip">
            <p>
              Nyalakan <strong>Do not disturb</strong> di Windows supaya notifikasi tidak muncul dua
              kali. Ghyna tetap menampilkannya, dan klik notifikasi untuk membuka aplikasinya.
            </p>
            <button class="quiet" onclick={() => invoke("open_dnd_settings")}>
              Buka pengaturan Do not disturb
            </button>
          </div>
        {/if}
      </div>
    </section>

    <section class="card">
      <div class="head">{@render icon("M9 18V6l10-2v12M9 18a2.5 2.5 0 1 1-5 0 2.5 2.5 0 0 1 5 0m10-2a2.5 2.5 0 1 1-5 0 2.5 2.5 0 0 1 5 0")}<h2>Musik</h2></div>
      <div class="body">
        {#if settings}
          <label class="row">
            <span class="label">
              Lirik di notch kecil
              <small>Judul lagu dikirim ke LRCLIB untuk tiap lagu</small>
            </span>
            <input
              class="switch"
              type="checkbox"
              checked={settings.smallLyrics}
              onchange={(e) => save({ smallLyrics: e.currentTarget.checked })}
            />
          </label>
        {/if}
      </div>
    </section>

    <section class="card">
      <div class="head">{@render icon("M8 2v4m8-4v4M3 10h18M5 4h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2")}<h2>Kalender</h2></div>
      <div class="body">
        <p class="note">Tempel alamat ICS dari Google Calendar atau Outlook. Disimpan hanya di laptop ini.</p>
        {#if settings}
          <input
            class="wide"
            type="password"
            value={settings.calendarUrl}
            placeholder="https://calendar.google.com/calendar/ical/…/basic.ics"
            onchange={(e) => save({ calendarUrl: e.currentTarget.value.trim() })}
          />
        {/if}
      </div>
    </section>

    <section class="card">
      <div class="head">{@render icon("M12 8v4l3 2M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0")}<h2>Pomodoro</h2></div>
      <div class="body">
        {#each numbers as [key, label, unit]}{@render field(key, label, unit)}{/each}
        <p class="note">Saat fokus berjalan, notifikasi aplikasi dibisukan. Info penting tetap tampil.</p>
      </div>
    </section>

    <section class="card">
      <div class="head">{@render icon("M12 2.7s6 6.3 6 10.3a6 6 0 0 1-12 0c0-4 6-10.3 6-10.3")}<h2>Minum air</h2></div>
      <div class="body">
        {#if settings}
          <label class="row">
            <span class="label">
              Pengingat minum
              <small>Halaman Air tetap ada di notch</small>
            </span>
            <input
              class="switch"
              type="checkbox"
              checked={settings.waterEnabled}
              onchange={(e) => save({ waterEnabled: e.currentTarget.checked })}
            />
          </label>
          {#if settings.waterEnabled}
            {#each water as [key, label, unit]}{@render field(key, label, unit)}{/each}
          {/if}
        {/if}
      </div>
    </section>

    <section class="card">
      <div class="head">{@render icon("M20 10c0 6-8 12-8 12S4 16 4 10a8 8 0 1 1 16 0M12 12.5a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5")}<h2>Lokasi cuaca</h2></div>
      <div class="body">
        <p class="note">
          {#if settings?.weatherCity}
            Sekarang <strong>{settings.weatherCity.name}</strong>
            <button class="link" onclick={() => save({ weatherCity: null })}>Pakai lokasi Windows</button>
          {:else}
            Sekarang <strong>lokasi Windows</strong> (kalau diizinkan)
          {/if}
        </p>
        <form onsubmit={search}>
          <input bind:value={query} placeholder="Cari kota, misalnya Bandung" />
          <button class="primary" type="submit" disabled={searching}>{searching ? "Mencari…" : "Cari"}</button>
        </form>
        {#if results.length}
          <ul>
            {#each results as place}
              <li>
                <button
                  onclick={() => save({ weatherCity: { name: place.name, latitude: place.latitude, longitude: place.longitude } })}
                >
                  <strong>{place.name}</strong>
                  <span>{place.detail}</span>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    </section>

    <p class="foot">Perubahan tersimpan otomatis.</p>
  </main>
</div>

<style>
  :global(body) {
    margin: 0;
    background: var(--surface-solid);
    color: var(--text);
    font-family: var(--font-ui);
    -webkit-font-smoothing: antialiased;
  }
  .page {
    position: relative;
    min-height: 100vh;
    background:
      radial-gradient(120% 52% at 50% 112%, rgba(143, 230, 180, 0.08), transparent 68%),
      linear-gradient(0deg, #0d1315 0%, var(--surface-solid) 46%);
  }
  .page::before {
    content: "";
    position: fixed;
    inset: 0;
    z-index: 0;
    pointer-events: none;
    background-image: var(--grain);
    background-size: 140px 140px;
    opacity: 0.05;
    mix-blend-mode: overlay;
  }
  header,
  main {
    position: relative;
    z-index: 1;
  }
  header {
    position: sticky;
    top: 0;
    z-index: 2;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 18px 12px;
    background: rgba(11, 16, 18, 0.72);
    -webkit-backdrop-filter: blur(20px) saturate(150%);
    backdrop-filter: blur(20px) saturate(150%);
    box-shadow: inset 0 -1px 0 rgba(255, 255, 255, 0.08), 0 1px 0 var(--line);
  }
  .brand {
    display: flex;
    align-items: baseline;
    gap: 9px;
  }
  h1 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 16px;
    font-weight: 600;
    letter-spacing: 0.01em;
  }
  .brand span {
    font-family: var(--font-mono);
    font-size: 8.5px;
    font-weight: 600;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .saved {
    padding: 4px 8px;
    border-radius: var(--radius-xs);
    background: var(--accent-soft);
    box-shadow: inset 0 0 0 0.5px var(--accent-line);
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: 8.5px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    opacity: 0;
    transform: translateY(-2px);
    transition:
      opacity var(--motion-ui) var(--ease-out),
      transform var(--motion-ui) var(--ease-out);
  }
  .saved.on {
    opacity: 1;
    transform: none;
  }
  main {
    display: grid;
    gap: 10px;
    padding: 14px 16px 22px;
  }
  .card {
    background: linear-gradient(0deg, rgba(255, 255, 255, 0.06), rgba(255, 255, 255, 0.012) 46%),
      var(--surface-raised);
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    padding: 0 14px 4px;
    box-shadow: inset 0 -1px 0 rgba(255, 255, 255, 0.08), 0 12px 26px -22px #000;
  }
  .head {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 0 6px;
  }
  .head h2 {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--muted-strong);
  }
  .icon {
    width: 14px;
    height: 14px;
    fill: none;
    stroke: var(--accent);
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
    opacity: 0.9;
  }
  .body > :first-child {
    border-top: 0;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    padding: 9px 0;
    border-top: 1px solid rgba(196, 220, 230, 0.09);
    font-size: 12.5px;
    cursor: default;
  }
  .label {
    display: flex;
    flex-direction: column;
    gap: 2px;
    line-height: 1.25;
  }
  .label small {
    font-size: 10.5px;
    color: var(--muted);
    line-height: 1.3;
  }
  .switch {
    appearance: none;
    flex: none;
    width: 34px;
    height: 20px;
    margin: 0;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.12);
    box-shadow: inset 0 0 0 0.5px var(--line);
    position: relative;
    cursor: pointer;
    transition:
      background var(--motion-ui) var(--ease-out),
      box-shadow var(--motion-ui) var(--ease-out);
  }
  .switch::after {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #e6eef0;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.35);
    transition: transform var(--motion-ui) var(--ease-out), background var(--motion-ui) var(--ease-out);
  }
  .switch:checked {
    background: var(--accent-strong);
    box-shadow: inset 0 0 0 0.5px rgba(255, 255, 255, 0.24);
  }
  .switch:checked::after {
    transform: translateX(14px);
    background: var(--accent-ink);
  }
  .switch:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  .num {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 4px 8px;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: rgba(6, 9, 11, 0.45);
  }
  .num input {
    width: 46px;
    padding: 0;
    border: 0;
    background: none;
    color: inherit;
    font: inherit;
    font-family: var(--font-mono);
    font-size: 12px;
    letter-spacing: -0.02em;
    font-variant-numeric: tabular-nums;
    text-align: right;
  }
  .num input:focus {
    outline: none;
  }
  .num:focus-within {
    border-color: var(--accent-line);
    box-shadow: 0 0 0 2px var(--accent-soft);
  }
  .num em {
    font-style: normal;
    font-family: var(--font-mono);
    font-size: 8.5px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
  }
  input[type="number"]::-webkit-inner-spin-button {
    appearance: none;
  }
  .note {
    margin: 2px 0 9px;
    font-size: 11.5px;
    line-height: 1.5;
    color: var(--muted-strong);
  }
  .note strong {
    color: var(--text);
    font-weight: 600;
  }
  .tip {
    padding: 10px 0 8px;
    border-top: 1px solid rgba(196, 220, 230, 0.09);
  }
  .tip p {
    margin: 0 0 8px;
    font-size: 11px;
    line-height: 1.5;
    color: var(--muted);
  }
  .tip strong {
    color: var(--muted-strong);
    font-weight: 600;
  }
  .quiet,
  .primary {
    padding: 7px 11px;
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    cursor: pointer;
    transition: background var(--motion-fast) var(--ease-out), color var(--motion-fast) var(--ease-out);
  }
  .quiet {
    border: 1px solid var(--line);
    background: rgba(255, 255, 255, 0.04);
    color: var(--muted-strong);
  }
  .quiet:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text);
  }
  .primary {
    border: 0;
    background: var(--accent);
    color: var(--accent-ink);
  }
  .primary:hover {
    background: var(--accent-strong);
  }
  .quiet:focus-visible,
  .primary:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  .primary:disabled {
    opacity: 0.6;
    cursor: default;
  }
  form {
    display: flex;
    gap: 8px;
  }
  input {
    flex: 1;
    min-width: 0;
    padding: 8px 10px;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: rgba(6, 9, 11, 0.45);
    color: inherit;
    font: inherit;
    font-size: 12px;
  }
  input:focus {
    outline: none;
    border-color: var(--accent-line);
    box-shadow: 0 0 0 2px var(--accent-soft);
  }
  input.wide {
    width: 100%;
    box-sizing: border-box;
    margin-bottom: 1px;
  }
  .picker {
    padding: 2px 0 9px;
    border-top: 1px solid rgba(196, 220, 230, 0.09);
  }
  .picker-bar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 8px 0 4px;
  }
  .count {
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.02em;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
  .links {
    display: flex;
    gap: 10px;
  }
  .link {
    padding: 0;
    border: 0;
    background: none;
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: 8.5px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    cursor: pointer;
  }
  .link:hover {
    color: var(--accent-strong);
  }
  .link:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  .app-list {
    max-height: 178px;
    overflow-y: auto;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    padding: 3px;
    background: rgba(6, 9, 11, 0.35);
  }
  .app {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 6px;
    border-radius: var(--radius-xs);
    font-size: 11.5px;
    cursor: pointer;
  }
  .app:hover {
    background: rgba(255, 255, 255, 0.06);
  }
  .app input {
    flex: none;
    width: 13px;
    height: 13px;
    padding: 0;
    accent-color: var(--accent-strong);
  }
  .app-name {
    flex: none;
  }
  .app-id {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: -0.01em;
    color: var(--muted);
  }
  ul {
    list-style: none;
    margin: 8px 0 6px;
    padding: 0;
    border-top: 1px solid rgba(196, 220, 230, 0.09);
  }
  li button {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 1px;
    padding: 7px 6px;
    border: 0;
    border-radius: var(--radius-xs);
    background: none;
    color: inherit;
    font: inherit;
    font-size: 11.5px;
    text-align: left;
    cursor: pointer;
  }
  li button:hover {
    background: rgba(255, 255, 255, 0.06);
  }
  li button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
  li span {
    font-size: 10.5px;
    color: var(--muted);
  }
  .warn {
    color: var(--warning);
  }
  .foot {
    margin: 0;
    padding: 2px 2px 0;
    text-align: center;
    font-family: var(--font-mono);
    font-size: 8.5px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
  }
</style>
