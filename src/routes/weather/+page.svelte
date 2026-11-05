<svelte:head>
  <title>Cuaca · Ghyna</title>
</svelte:head>

<script lang="ts">
  import { onMount } from 'svelte';
  import type { Weather } from '$lib/weather';
  import { fetchWeather } from '$lib/weather';

  let weather: Weather | null = null;
  let error = '';

  // Simple fallback coordinates (Jakarta)
  const DEFAULT_LAT = -6.2088;
  const DEFAULT_LON = 106.8456;

  onMount(async () => {
    try {
      // Try browser geolocation first
      if (navigator.geolocation) {
        navigator.geolocation.getCurrentPosition(
          async (pos) => {
            const { latitude, longitude } = pos.coords;
            weather = await fetchWeather(latitude, longitude);
          },
          async () => {
            // Fallback to default location
            weather = await fetchWeather(DEFAULT_LAT, DEFAULT_LON);
          }
        );
      } else {
        weather = await fetchWeather(DEFAULT_LAT, DEFAULT_LON);
      }
    } catch (e) {
      error = (e as Error).message;
    }
  });
</script>

{#if error}
  <p class="error">Kesalahan: {error}</p>
{:else if !weather}
    <main class="weather-page"><p class="muted">Memuat data cuaca…</p></main>
{:else}
  <main class="weather-page">
    <a class="back" href="/">← Kembali ke jam</a>
    <header><div><p class="eyebrow">CUACA SEKARANG</p><h1>{weather.location}</h1><p class="description">{weather.description}</p></div><div class="icon" aria-hidden="true">{weather.icon}</div></header>
    <section class="current"><p class="temp">{weather.temp}°</p><span>Celsius</span></section>
    <section class="details" aria-label="Detail cuaca">
      <div><span>Kelembapan</span><strong>{weather.humidity}%</strong></div>
      <div><span>Angin</span><strong>{weather.wind} km/j</strong></div>
    </section>
    <section class="forecast"><h2>Beberapa jam ke depan</h2><div class="forecast-row">{#each weather.forecast as item}<div><span>{item.time}</span><b>{item.icon}</b><strong>{item.temp}°</strong></div>{/each}</div></section>
    <p class="updated">Data dari BMKG · diperbarui saat halaman dibuka</p>
  </main>
{/if}

<style>
  :global(body) {
    margin: 0;
    background: var(--surface-solid);
    color: var(--text);
    font-family: var(--font-ui);
    -webkit-font-smoothing: antialiased;
  }
  :global(body::before) {
    content: "";
    position: fixed;
    inset: 0;
    pointer-events: none;
    background-image: var(--grain);
    background-size: 140px 140px;
    opacity: 0.05;
    mix-blend-mode: overlay;
  }
  .weather-page {
    width: min(680px, calc(100% - 40px));
    margin: 40px auto;
    padding: 28px;
    border: 1px solid var(--line);
    border-radius: var(--radius-xl);
    background:
      radial-gradient(90% 70% at 12% -10%, rgba(255, 138, 107, 0.09), transparent 62%),
      linear-gradient(180deg, rgba(255, 255, 255, 0.05), rgba(255, 255, 255, 0.012)),
      var(--surface-raised);
    box-shadow:
      inset 0 1px 0 var(--edge-top),
      0 24px 70px -34px #000,
      0 2px 6px rgba(0, 0, 0, 0.35);
  }
  .back {
    color: var(--accent);
    text-decoration: none;
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  .back:hover {
    color: var(--accent-strong);
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 20px;
    margin-top: 36px;
  }
  h1 {
    margin: 4px 0;
    font-family: var(--font-display);
    font-size: 34px;
    font-weight: 600;
    letter-spacing: 0.01em;
  }
  .eyebrow {
    margin: 0;
    color: var(--muted);
    font-family: var(--font-mono);
    font-size: 8.5px;
    font-weight: 600;
    letter-spacing: 0.14em;
    text-transform: uppercase;
  }
  .description,
  .muted,
  .updated {
    color: var(--muted);
  }
  .icon {
    font-size: 64px;
    line-height: 1;
  }
  .current {
    display: flex;
    align-items: baseline;
    gap: 12px;
    margin: 26px 0;
  }
  .temp {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 72px;
    font-weight: 600;
    letter-spacing: -0.05em;
    font-variant-numeric: tabular-nums;
  }
  .current span {
    color: var(--muted);
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  .details {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }
  .details div,
  .forecast {
    padding: 18px;
    border: 1px solid var(--line);
    border-radius: var(--radius-lg);
    background: rgba(255, 255, 255, 0.035);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
  }
  .details span,
  .forecast span {
    display: block;
    color: var(--muted);
    font-family: var(--font-mono);
    font-size: 8.5px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  .details strong {
    display: block;
    margin-top: 7px;
    font-family: var(--font-mono);
    font-size: 20px;
    font-weight: 600;
    letter-spacing: -0.02em;
    font-variant-numeric: tabular-nums;
  }
  .forecast {
    margin-top: 12px;
  }
  h2 {
    margin: 0 0 16px;
    font-family: var(--font-display);
    font-size: 15px;
    font-weight: 600;
    letter-spacing: 0.015em;
  }
  .forecast-row {
    display: flex;
    gap: 8px;
    overflow-x: auto;
  }
  .forecast-row div {
    min-width: 82px;
    text-align: center;
  }
  .forecast-row b,
  .forecast-row strong {
    display: block;
    margin-top: 9px;
    font-size: 24px;
  }
  .forecast-row strong {
    font-family: var(--font-mono);
    font-size: 17px;
    font-weight: 600;
    letter-spacing: -0.02em;
  }
  .updated {
    margin: 22px 0 0;
    font-family: var(--font-mono);
    font-size: 8.5px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .error {
    color: var(--danger);
  }
  @media (max-width: 520px) {
    .weather-page {
      margin: 20px auto;
      padding: 22px;
    }
    h1 {
      font-size: 28px;
    }
    .icon {
      font-size: 52px;
    }
    .temp {
      font-size: 60px;
    }
  }
</style>
