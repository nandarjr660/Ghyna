//! Now playing, from the Windows media overlay: any player that shows up there
//! (Spotify, YouTube in a browser, ...). Changes arrive as WinRT events.

use std::sync::{Mutex, OnceLock};

use base64::Engine;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use windows::core::HSTRING;
use windows::Foundation::{TypedEventHandler, Uri};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as Manager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
};
use windows::Storage::Streams::{DataReader, IRandomAccessStreamReference};
use windows::Web::Http::HttpClient;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

static MANAGER: OnceLock<Manager> = OnceLock::new();
/// The session we listen to, with its event tokens so they can be removed when it changes.
static CURRENT: Mutex<Option<(Session, [i64; 3])>> = Mutex::new(None);
/// Album art is costly to read, so the last one is kept: (track key, data URL).
static ART: Mutex<Option<(String, String)>> = Mutex::new(None);

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NowPlaying {
    title: String,
    artist: String,
    album: String,
    /// Album art as a data URL.
    art: Option<String>,
    /// App id of the player, e.g. "Spotify.exe" or "MSEdge".
    source: String,
    playing: bool,
    /// Seconds.
    position: f64,
    duration: f64,
    /// Unix ms at which `position` was true.
    updated_at: f64,
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        // This thread stays alive as the MTA that WinRT callbacks and async commands rely on.
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        let Ok(manager) = Manager::RequestAsync().and_then(|op| op.get()) else { return };
        let handle = app.clone();
        let _ = manager.CurrentSessionChanged(&TypedEventHandler::new(move |_, _| {
            watch_current(&handle);
            Ok(())
        }));
        MANAGER.set(manager).ok();
        watch_current(&app);
        loop {
            std::thread::park();
        }
    });
}

/// Moves our event handlers to whichever session Windows now considers current.
fn watch_current(app: &AppHandle) {
    let Some(manager) = MANAGER.get() else { return };
    let mut current = CURRENT.lock().unwrap();
    if let Some((session, [props, playback, timeline])) = current.take() {
        let _ = session.RemoveMediaPropertiesChanged(props);
        let _ = session.RemovePlaybackInfoChanged(playback);
        let _ = session.RemoveTimelinePropertiesChanged(timeline);
    }
    if let Ok(session) = manager.GetCurrentSession() {
        let (a, b, c) = (app.clone(), app.clone(), app.clone());
        let tokens = [
            session
                .MediaPropertiesChanged(&TypedEventHandler::new(move |_, _| Ok(publish(&a))))
                .unwrap_or(0),
            session
                .PlaybackInfoChanged(&TypedEventHandler::new(move |_, _| Ok(publish(&b))))
                .unwrap_or(0),
            session
                .TimelinePropertiesChanged(&TypedEventHandler::new(move |_, _| Ok(publish(&c))))
                .unwrap_or(0),
        ];
        *current = Some((session, tokens));
    }
    drop(current);
    publish(app);
}

fn publish(app: &AppHandle) {
    let _ = app.emit("media", now_playing());
}

fn session() -> Option<Session> {
    MANAGER.get()?.GetCurrentSession().ok()
}

fn now_playing() -> Option<NowPlaying> {
    let session = session()?;
    let props = session.TryGetMediaPropertiesAsync().ok()?.get().ok()?;
    let title = props.Title().ok()?.to_string();
    if title.is_empty() {
        return None;
    }
    let artist = props.Artist().map(|s| s.to_string()).unwrap_or_default();
    let album = props.AlbumTitle().map(|s| s.to_string()).unwrap_or_default();
    let playing = session
        .GetPlaybackInfo()
        .and_then(|info| info.PlaybackStatus())
        .is_ok_and(|status| status == Status::Playing);

    let secs = |ticks: i64| ticks as f64 / 10_000_000.0;
    let (position, duration, updated_at) = match session.GetTimelineProperties() {
        Ok(t) => (
            t.Position().map(|p| secs(p.Duration)).unwrap_or(0.0),
            t.EndTime().map(|p| secs(p.Duration)).unwrap_or(0.0),
            // DateTime counts 100ns ticks since 1601.
            t.LastUpdatedTime()
                .map(|d| (d.UniversalTime - 116_444_736_000_000_000) as f64 / 10_000.0)
                .unwrap_or(0.0),
        ),
        Err(_) => (0.0, 0.0, 0.0),
    };

    let key = format!("{title}\u{1}{artist}");
    let cached = ART.lock().unwrap().as_ref().filter(|(k, _)| *k == key).map(|(_, a)| a.clone());
    let art = cached.or_else(|| {
        let art = read_art(props.Thumbnail().ok()?)?;
        *ART.lock().unwrap() = Some((key, art.clone()));
        Some(art)
    });

    Some(NowPlaying {
        title,
        artist,
        album,
        art,
        source: session.SourceAppUserModelId().map(|s| s.to_string()).unwrap_or_default(),
        playing,
        position,
        duration,
        updated_at,
    })
}

/// Reads an image stream (album art, app logo) into a data URL.
pub(crate) fn read_art(thumbnail: IRandomAccessStreamReference) -> Option<String> {
    let stream = thumbnail.OpenReadAsync().ok()?.get().ok()?;
    let size = stream.Size().ok()? as u32;
    if size == 0 {
        return None;
    }
    let reader = DataReader::CreateDataReader(&stream.GetInputStreamAt(0).ok()?).ok()?;
    reader.LoadAsync(size).ok()?.get().ok()?;
    let mut bytes = vec![0u8; size as usize];
    reader.ReadBytes(&mut bytes).ok()?;
    let mime = stream
        .ContentType()
        .map(|s| s.to_string())
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "image/png".into());
    Some(format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
}

/// Some players (browsers especially) publish only their own logo through GSMTC,
/// or nothing at all. Ask the iTunes catalog for the real cover then.
#[tauri::command]
pub async fn cover_lookup(title: String, artist: String, album: String) -> Option<String> {
    let title = title.trim();
    if title.is_empty() {
        return None;
    }
    lookup_cover(title, artist.trim(), album.trim())
}

fn lookup_cover(title: &str, artist: &str, album: &str) -> Option<String> {
    let term = if artist.is_empty() {
        title.to_string()
    } else {
        format!("{title} {artist}")
    };
    let url = format!(
        "https://itunes.apple.com/search?term={}&entity=song&limit=8",
        url_encode(&term)
    );
    let client = HttpClient::new().ok()?;
    let uri = Uri::CreateUri(&HSTRING::from(url)).ok()?;
    let body = client.GetStringAsync(&uri).ok()?.get().ok()?.to_string();
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;

    let clean = |s: &str| {
        s.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { ' ' })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let want_title = clean(title);
    let want_artist = clean(artist);
    let want_album = clean(album);

    let mut best: Option<(i32, &serde_json::Value)> = None;
    for item in json["results"].as_array()? {
        let track = clean(item["trackName"].as_str().unwrap_or(""));
        let by = clean(item["artistName"].as_str().unwrap_or(""));
        let from = clean(item["collectionName"].as_str().unwrap_or(""));
        if track.is_empty() || (!track.contains(&want_title) && !want_title.contains(&track)) {
            continue;
        }

        let mut score = if track == want_title { 180 } else { 100 };
        if !want_artist.is_empty() {
            let artist_ok = by == want_artist
                || by.contains(&want_artist)
                || want_artist.contains(&by)
                || want_artist
                    .split_whitespace()
                    .any(|word| word.len() > 2 && by.contains(word));
            if !artist_ok {
                continue;
            }
            score += if by == want_artist { 60 } else { 25 };
        }
        if !want_album.is_empty() && from == want_album {
            score += 15;
        }
        if best.as_ref().is_none_or(|(best_score, _)| score > *best_score) {
            best = Some((score, item));
        }
    }

    // A loose title match without a matching artist is usually the wrong song.
    let (score, item) = best?;
    if score < 150 {
        return None;
    }
    let mut artwork = item["artworkUrl100"].as_str()?.to_string();
    artwork = artwork
        .replace("100x100bb", "600x600bb")
        .replace("100x100", "600x600");
    let uri = Uri::CreateUri(&HSTRING::from(artwork)).ok()?;
    let response = client.GetAsync(&uri).ok()?.get().ok()?;
    if !response.IsSuccessStatusCode().unwrap_or(false) {
        return None;
    }
    let buffer = response.Content().ok()?.ReadAsBufferAsync().ok()?.get().ok()?;
    let len = buffer.Length().ok()? as usize;
    if len == 0 || len > 5_000_000 {
        return None;
    }
    let reader = DataReader::FromBuffer(&buffer).ok()?;
    let mut bytes = vec![0u8; len];
    reader.ReadBytes(&mut bytes).ok()?;
    let mime = if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        "image/jpeg"
    } else if bytes.starts_with(b"\x89PNG") {
        "image/png"
    } else if bytes.len() >= 12 && &bytes[8..12] == b"WEBP" {
        "image/webp"
    } else {
        "image/jpeg"
    };
    Some(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

/// WebView fetches can be blocked by a URL policy; this keeps lyrics working
/// anyway. Restricted to LRCLIB so the command cannot be used as an open proxy.
#[tauri::command]
pub async fn lyrics_fetch(url: String) -> Result<String, String> {
    if !url.starts_with("https://lrclib.net/api/") {
        return Err("Hanya LRCLIB yang diizinkan".into());
    }
    let uri = Uri::CreateUri(&HSTRING::from(url)).map_err(|e| e.message().to_string())?;
    let client = HttpClient::new().map_err(|e| e.message().to_string())?;
    client
        .GetStringAsync(&uri)
        .and_then(|op| op.get())
        .map(|s| s.to_string())
        .map_err(|e| e.message().to_string())
}

fn url_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len() * 3);
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

// Commands are async so they run on a worker thread, not the UI thread, while
// they wait for the player.

#[tauri::command]
pub async fn media_now() -> Option<NowPlaying> {
    now_playing()
}

#[tauri::command]
pub async fn media_toggle() {
    if let Some(s) = session() {
        let _ = s.TryTogglePlayPauseAsync().and_then(|op| op.get());
    }
}

#[tauri::command]
pub async fn media_next() {
    if let Some(s) = session() {
        let _ = s.TrySkipNextAsync().and_then(|op| op.get());
    }
}

#[tauri::command]
pub async fn media_prev() {
    if let Some(s) = session() {
        let _ = s.TrySkipPreviousAsync().and_then(|op| op.get());
    }
}

#[tauri::command]
pub async fn media_seek(seconds: f64) {
    if let Some(s) = session() {
        let _ = s
            .TryChangePlaybackPositionAsync((seconds * 10_000_000.0) as i64)
            .and_then(|op| op.get());
    }
}
