// Current weather and forecast from BMKG (Indonesian meteorological agency).

export type WeatherHour = { time: string; temp: number; icon: string };
export type Weather = {
  temp: number;
  icon: string;
  description: string;
  location: string;
  humidity: number;
  wind: number;
  forecast: WeatherHour[];
};

type BmkgLocation = {
  adm4?: string;
  desa?: string;
  kecamatan?: string;
  kotkab?: string;
  provinsi?: string;
};

type BmkgCurrent = {
  weather_desc?: string;
  t?: number;
  hu?: number;
  ws?: number;
  local_datetime?: string;
};

type BmkgPresent = {
  data?: {
    lokasi?: BmkgLocation;
    cuaca?: BmkgCurrent;
  };
};

type BmkgForecastEntry = {
  datetime?: string;
  local_datetime?: string;
  t?: number;
  weather_desc?: string;
};

type BmkgForecast = {
  data?: Array<{ cuaca?: BmkgForecastEntry[][] }>;
};

/** BMKG description → emoji, with a night variant for clear skies. */
function iconFor(description = "", localDatetime?: string): string {
  const d = description.toLowerCase();
  const hour = localDatetime ? Number(localDatetime.slice(11, 13)) : new Date().getHours();
  const night = hour >= 18 || hour < 6;

  if (d.includes("badai") || d.includes("petir")) return "⛈️";
  if (d.includes("hujan lebat") || d.includes("hujan deras")) return "🌧️";
  if (d.includes("hujan")) return "🌦️";
  if (d.includes("gerimis")) return "🌦️";
  if (d.includes("kabut") || d.includes("asap")) return "🌫️";
  if (d.includes("cerah berawan")) return night ? "☁️" : "🌤️";
  if (d.includes("berawan")) return "☁️";
  if (d.includes("cerah")) return night ? "🌙" : "☀️";
  return night ? "☁️" : "🌤️";
}

function displayTime(entry: BmkgForecastEntry): string {
  const local = entry.local_datetime?.slice(11, 16);
  if (local) return local;

  const parsed = Date.parse(entry.datetime ?? "");
  if (Number.isNaN(parsed)) return "--:--";
  return new Date(parsed).toLocaleTimeString("id-ID", { hour: "2-digit", minute: "2-digit" });
}

export async function fetchWeather(latitude: number, longitude: number): Promise<Weather | null> {
  try {
    const presentUrl = new URL("https://cuaca.bmkg.go.id/api/presentwx/coord");
    presentUrl.searchParams.set("lon", String(longitude));
    presentUrl.searchParams.set("lat", String(latitude));

    const presentRes = await fetch(presentUrl);
    if (!presentRes.ok) return null;

    const present = (await presentRes.json()) as BmkgPresent;
    const lokasi = present.data?.lokasi;
    const current = present.data?.cuaca;
    if (!lokasi || !current) return null;

    let forecast: WeatherHour[] = [];
    if (lokasi.adm4) {
      try {
        const forecastRes = await fetch(
          `https://api.bmkg.go.id/publik/prakiraan-cuaca?adm4=${encodeURIComponent(lokasi.adm4)}`
        );
        if (forecastRes.ok) {
          const data = (await forecastRes.json()) as BmkgForecast;
          const days = data.data?.[0]?.cuaca ?? [];
          const now = Date.now();

          for (const entry of days.flat()) {
            const at = Date.parse(entry.datetime ?? "");
            if (Number.isNaN(at) || at < now) continue;

            forecast.push({
              time: displayTime(entry),
              temp: Math.round(Number(entry.t ?? 0)),
              icon: iconFor(entry.weather_desc, entry.local_datetime),
            });
            if (forecast.length >= 5) break;
          }
        }
      } catch {
        // Keep the current weather even when the forecast endpoint is unavailable.
      }
    }

    return {
      temp: Math.round(Number(current.t ?? 0)),
      icon: iconFor(current.weather_desc, current.local_datetime),
      description: current.weather_desc ?? "Kondisi tidak diketahui",
      location:
        [lokasi.desa, lokasi.kecamatan].filter(Boolean).join(", ") ||
        lokasi.kotkab ||
        "Lokasi saat ini",
      humidity: Math.round(Number(current.hu ?? 0)),
      wind: Math.round(Number(current.ws ?? 0)),
      forecast,
    };
  } catch {
    return null;
  }
}

export type Place = { name: string; latitude: number; longitude: number; detail: string };

/** City search for the settings window. */
export async function searchCity(name: string): Promise<Place[]> {
  const q = new URLSearchParams({ name, count: "6", language: "id" });
  const res = await fetch(`https://geocoding-api.open-meteo.com/v1/search?${q}`);
  if (!res.ok) return [];
  const { results = [] } = await res.json();
  return results.map((r: any) => ({
    name: r.name,
    latitude: r.latitude,
    longitude: r.longitude,
    detail: [r.admin1, r.country].filter(Boolean).join(", "),
  }));
}
