<script lang="ts">
  import type { Weather } from "$lib/weather";

  let { weather }: { weather: Weather | null } = $props();

  function weatherKind(description: string) {
    const text = description.toLowerCase();
    if (text.includes("petir") || text.includes("badai")) return "storm";
    if (text.includes("hujan") || text.includes("gerimis")) return "rain";
    if (text.includes("kabut")) return "fog";
    if (text.includes("berawan") || text.includes("mendung")) return "cloudy";
    return "clear";
  }
</script>

{#if weather}
  <section class="weather-page" class:weather-rain={weatherKind(weather.description) === "rain"} class:weather-storm={weatherKind(weather.description) === "storm"} class:weather-cloudy={weatherKind(weather.description) === "cloudy"} class:weather-fog={weatherKind(weather.description) === "fog"} aria-label="Cuaca sekarang">
    <div class="weather-head">
      <div><span class="eyebrow">CUACA SEKARANG</span><strong>{weather.description}</strong><small>{weather.location}</small></div>
      <span class="icon" aria-hidden="true">{weather.icon}</span>
    </div>
    <div class="weather-main"><b>{weather.temp}°</b><span>Celsius</span></div>
  </section>
{:else}
  <section class="weather-page empty" aria-label="Cuaca"><span class="eyebrow">CUACA</span><strong>Memuat data cuaca…</strong></section>
{/if}

<style>
  .weather-page { width: 100%; height: 100%; box-sizing: border-box; overflow: hidden; padding: 16px 20px 8px; color: #f7f5f1; }
  .weather-head, .weather-main { display: flex; align-items: center; }
  .weather-head { justify-content: space-between; gap: 12px; }
  .weather-head div { display: grid; gap: 1px; min-width: 0; }
  .eyebrow, .weather-head small, .weather-main span { color: #aaa6ae; font-size: 10px; letter-spacing: .08em; }
  .weather-head strong { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 14px; }
  .weather-head small { letter-spacing: 0; font-size: 10px; }
  .icon { font-size: 32px; line-height: 1; animation: weather-float 3s ease-in-out infinite; }
  .weather-main { gap: 8px; margin-top: 13px; }
  .weather-main b { font-size: 36px; line-height: 1; }
  .weather-main span { letter-spacing: 0; font-size: 9px; }
  .weather-rain .icon { animation: weather-rain 1.8s ease-in-out infinite; }
  .weather-storm .icon { animation: weather-storm 1.2s ease-in-out infinite; }
  .weather-cloudy .icon, .weather-fog .icon { animation-duration: 4.5s; }
  .empty { display: grid; gap: 8px; align-content: center; height: 100%; }
  @keyframes weather-float { 0%, 100% { transform: translateY(0); } 50% { transform: translateY(-3px); } }
  @keyframes weather-rain { 0%, 100% { transform: translateY(0) rotate(0); } 50% { transform: translateY(2px) rotate(-4deg); } }
  @keyframes weather-storm { 0%, 100% { transform: translateX(0); } 45% { transform: translateX(-2px); } 55% { transform: translateX(2px); } }
  @media (prefers-reduced-motion: reduce) { .icon { animation: none; } }
</style>
