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
    <div class="weather-layout">
      <div class="weather-copy">
        <span class="eyebrow">CUACA SEKARANG</span>
        <strong>{weather.description}</strong>
        <small>{weather.location}</small>
        <div class="weather-main"><b>{weather.temp}°</b><span>Celsius</span></div>
      </div>
      <div class="weather-art" aria-hidden="true">
        <span class="weather-marks"></span>
        <span class="icon">{weather.icon}</span>
      </div>
    </div>
  </section>
{:else}
  <section class="weather-page empty" aria-label="Cuaca"><span class="eyebrow">CUACA</span><strong>Memuat data cuaca…</strong></section>
{/if}

<style>
  .weather-page { width: 100%; height: 100%; box-sizing: border-box; overflow: hidden; padding: 12px 20px 8px; color: #f7f5f1; font-family: var(--font-ui); }
  .weather-layout { height: 100%; display: grid; grid-template-columns: minmax(0, 1fr) 96px; align-items: center; gap: 12px; }
  .weather-copy { display: grid; align-content: center; min-width: 0; }
  .eyebrow, .weather-main span { color: #aaa6ae; font-size: 10px; letter-spacing: .08em; }
  .weather-copy strong { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 15px; line-height: 1.2; }
  .weather-copy small { color: #aaa6ae; letter-spacing: 0; font-size: 10px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .weather-art { position: relative; display: grid; place-items: center; width: 96px; height: 88px; }
  .icon { position: relative; z-index: 1; font-size: 38px; line-height: 1; animation: weather-float 3.5s ease-in-out infinite; }
  .weather-main { display: flex; align-items: baseline; gap: 7px; margin-top: 9px; }
  .weather-main b { font-size: 37px; line-height: .95; letter-spacing: -.04em; }
  .weather-main span { letter-spacing: 0; font-size: 9px; }
  .weather-rain .icon { animation: weather-rain 1.8s ease-in-out infinite; }
  .weather-storm .icon { animation: weather-storm 8s ease-in-out infinite; }
  .weather-cloudy .icon, .weather-fog .icon { animation-duration: 4.5s; }
  .weather-marks { position: absolute; inset: 14px 8px 10px; border-radius: 50%; background: rgba(255, 198, 93, .08); opacity: 0; }
  .weather-rain .weather-marks { opacity: .8; background: repeating-linear-gradient(110deg, transparent 0 13px, rgba(140, 190, 255, .45) 14px 15px, transparent 16px 25px); animation: rain-marks 1.2s linear infinite; }
  .weather-storm .weather-marks { background: rgba(255, 241, 172, .28); animation: storm-flash 8s ease-in-out infinite; }
  .weather-cloudy .weather-marks, .weather-fog .weather-marks { opacity: .3; background: linear-gradient(90deg, transparent, rgba(255,255,255,.14), transparent); animation: cloud-drift 7s ease-in-out infinite; }
  .empty { display: grid; gap: 8px; align-content: center; height: 100%; }
  @keyframes weather-float { 0%, 100% { transform: translateY(0); } 50% { transform: translateY(-3px); } }
  @keyframes weather-rain { 0%, 100% { transform: translateY(0) rotate(0); } 50% { transform: translateY(2px) rotate(-4deg); } }
  @keyframes weather-storm { 0%, 100% { transform: translateX(0); } 45% { transform: translateX(-2px); } 55% { transform: translateX(2px); } }
  @keyframes rain-marks { from { transform: translateY(-8px); } to { transform: translateY(8px); } }
  @keyframes storm-flash { 0%, 92%, 100% { opacity: 0; } 94% { opacity: 1; } 96% { opacity: .1; } 98% { opacity: .7; } }
  @keyframes cloud-drift { 0%, 100% { transform: translateX(-8px); } 50% { transform: translateX(8px); } }
  @media (prefers-reduced-motion: reduce) { .icon, .weather-marks { animation: none; } }
</style>
