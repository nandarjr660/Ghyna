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
  <p>Memuat data cuaca…</p>
{:else}
  <div class="weather-card">
    <div class="icon" aria-hidden="true">{weather.icon}</div>
    <p class="temp">{weather.temp}°C</p>
  </div>
{/if}

<style>
  .weather-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 2rem;
    background: var(--bg, #f2e3cf);
    border-radius: 1rem;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
    margin-top: 2rem;
  }
  .icon { font-size: 4rem; margin: 0.5rem; }
  .temp { font-size: 2rem; font-weight: bold; }
  .error { color: #c00; }
</style>
