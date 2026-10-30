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
  <p class="error">Error: {error}</p>
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
    <p class="updated">Data dari Open-Meteo · diperbarui saat halaman dibuka</p>
  </main>
{/if}

<style>
  :global(body) { margin: 0; background: #17131a; color: #f7f1e8; font-family: system-ui, sans-serif; }
  .weather-page { width: min(680px, calc(100% - 40px)); margin: 40px auto; padding: 28px; border: 1px solid #ffffff1c; border-radius: 28px; background: #242029; box-shadow: 0 24px 70px #0006; }
  .back { color: #b7e8c0; text-decoration: none; font-size: 14px; } header { display:flex; justify-content:space-between; align-items:center; margin-top: 42px; } h1 { margin: 4px 0; font-size: 34px; } .eyebrow { color:#8fd69e; font-size:11px; letter-spacing:.16em; } .description,.muted,.updated { color:#bcb4bd; } .icon { font-size: 76px; } .current { display:flex; align-items:baseline; gap:12px; margin: 28px 0; } .temp { margin:0; font-size:76px; font-weight:700; } .current span { color:#aaa0aa; } .details { display:grid; grid-template-columns:1fr 1fr; gap:12px; } .details div,.forecast { padding:18px; border:1px solid #ffffff12; border-radius:18px; background:#ffffff08; } .details span,.forecast span { display:block; color:#aaa0aa; font-size:13px; } .details strong { display:block; margin-top:7px; font-size:20px; } .forecast { margin-top:12px; } h2 { margin:0 0 16px; font-size:16px; } .forecast-row { display:flex; gap:8px; overflow-x:auto; } .forecast-row div { min-width:82px; text-align:center; } .forecast-row b,.forecast-row strong { display:block; margin-top:9px; font-size:24px; } .forecast-row strong { font-size:17px; } .updated { margin:22px 0 0; font-size:12px; } .error { color:#ff9d9d; }
  @media (max-width: 520px) { .weather-page { margin:20px auto; padding:22px; } h1 { font-size:28px; } .icon { font-size:58px; } .temp { font-size:64px; } }
</style>
