// Current weather from Open-Meteo (free, no API key).

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

const descriptions: Record<number, string> = {
  0: "Cerah", 1: "Cerah berawan", 2: "Berawan sebagian", 3: "Mendung",
  45: "Berkabut", 48: "Berkabut tebal", 51: "Gerimis ringan", 53: "Gerimis",
  55: "Gerimis lebat", 61: "Hujan ringan", 63: "Hujan", 65: "Hujan lebat",
  71: "Salju ringan", 73: "Salju", 75: "Salju lebat", 80: "Hujan lokal",
  81: "Hujan lokal", 82: "Hujan deras", 95: "Badai petir", 96: "Badai petir",
  99: "Badai petir",
};

/** WMO weather code → emoji. */
function icon(code: number, day: boolean): string {
  if (code === 0) return day ? "☀️" : "🌙";
  if (code <= 2) return day ? "🌤️" : "☁️";
  if (code === 3) return "☁️";
  if (code <= 48) return "🌫️";
  if (code <= 67) return "🌧️";
  if (code <= 77) return "❄️";
  if (code <= 82) return "🌦️";
  return "⛈️";
}

export async function fetchWeather(latitude: number, longitude: number): Promise<Weather | null> {
  try {
    const q = new URLSearchParams({
      latitude: String(latitude),
      longitude: String(longitude),
      current: "temperature_2m,relative_humidity_2m,weather_code,wind_speed_10m,is_day",
      hourly: "temperature_2m,weather_code",
      forecast_days: "1",
      timezone: "auto",
    });
    const res = await fetch(`https://api.open-meteo.com/v1/forecast?${q}`);
    if (!res.ok) return null;
    const data = await res.json();
    const { current, hourly } = data;
    const currentCode = Number(current.weather_code);
    const forecast: WeatherHour[] = [];
    for (let i = 0; i < hourly.time.length && forecast.length < 5; i += 1) {
      if (hourly.time[i] <= current.time) continue;
      forecast.push({
        time: new Date(hourly.time[i]).toLocaleTimeString("id-ID", { hour: "2-digit", minute: "2-digit" }),
        temp: Math.round(hourly.temperature_2m[i]),
        icon: icon(Number(hourly.weather_code[i]), true),
      });
    }
    const timezone = String(data.timezone ?? "").split("/").pop()?.replaceAll("_", " ");
    return {
      temp: Math.round(current.temperature_2m),
      icon: icon(currentCode, current.is_day === 1),
      description: descriptions[currentCode] ?? "Kondisi tidak diketahui",
      location: timezone || "Lokasi saat ini",
      humidity: Math.round(current.relative_humidity_2m),
      wind: Math.round(current.wind_speed_10m),
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
