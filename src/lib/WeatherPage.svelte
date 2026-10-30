<script lang="ts">
  import type { Weather } from "$lib/weather";

  let { weather }: { weather: Weather | null } = $props();
</script>

{#if weather}
  <section class="weather-page" aria-label="Cuaca sekarang">
    <div class="weather-head">
      <div><span class="eyebrow">CUACA SEKARANG</span><strong>{weather.location}</strong><small>{weather.description}</small></div>
      <span class="icon" aria-hidden="true">{weather.icon}</span>
    </div>
    <div class="weather-main"><b>{weather.temp}°</b><span>kelembapan {weather.humidity}% · angin {weather.wind} km/j</span></div>
    <div class="forecast" aria-label="Prakiraan cuaca">
      {#each weather.forecast.slice(0, 4) as item}<div><small>{item.time}</small><span>{item.icon}</span><b>{item.temp}°</b></div>{/each}
    </div>
    <small class="source">Data dari BMKG</small>
  </section>
{:else}
  <section class="weather-page empty" aria-label="Cuaca"><span class="eyebrow">CUACA</span><strong>Memuat data cuaca…</strong></section>
{/if}

<style>
  .weather-page { width: 100%; box-sizing: border-box; padding: 18px 22px 14px; color: #f7f5f1; }
  .weather-head, .weather-main, .forecast { display: flex; align-items: center; }
  .weather-head { justify-content: space-between; gap: 16px; }
  .weather-head div { display: grid; gap: 3px; min-width: 0; }
  .eyebrow, .source, .weather-head small, .weather-main span, .forecast small { color: #aaa6ae; font-size: 10px; letter-spacing: .08em; }
  .weather-head strong { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 15px; }
  .weather-head small { letter-spacing: 0; font-size: 11px; }
  .icon { font-size: 34px; line-height: 1; }
  .weather-main { gap: 12px; margin: 10px 0 8px; }
  .weather-main b { font-size: 34px; line-height: 1; }
  .weather-main span { letter-spacing: 0; font-size: 10px; }
  .forecast { gap: 8px; border-top: 1px solid #ffffff18; padding-top: 8px; }
  .forecast div { display: grid; justify-items: center; gap: 2px; flex: 1; }
  .forecast span { font-size: 17px; }.forecast b { font-size: 11px; }.forecast small { letter-spacing: 0; font-size: 9px; }
  .source { display: block; margin-top: 7px; letter-spacing: 0; font-size: 9px; }.empty { display: grid; gap: 8px; align-content: center; height: 100%; }
</style>
