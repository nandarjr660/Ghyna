//! Local bridge for the companion browser extension.
//!
//! Chrome keeps the size of a running download in memory only; the extension
//! reads it through `chrome.downloads` and posts it here, so the notch can show
//! a real percentage instead of guessing. Anything posted here is trusted only
//! while it is fresh; otherwise the disk watcher takes over again.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::Deserialize;
use tauri::{AppHandle, Emitter};

use crate::downloads::{self, Download};

/// Loopback only, and a port nothing else is likely to want.
const PORT: u16 = 47823;
/// While bytes are moving the extension posts about once a second. A longer
/// grace than one interval keeps the notch calm if a post or two is missed,
/// without letting a closed browser leave a number up for a minute.
const ACTIVE_FRESH: Duration = Duration::from_secs(15);
/// A paused download only posts its state change plus a slow heartbeat.
const PAUSED_FRESH: Duration = Duration::from_secs(60);
const MAX_BODY: usize = 64 * 1024;

struct State {
    item: Option<Item>,
    updated: Instant,
    /// Smoothed bytes per second, from consecutive posts for the same download.
    speed: f64,
    last_bytes: u64,
    last_at: Instant,
    /// Last progress line written, so a long download leaves a readable trail
    /// without one line per post.
    last_note: Instant,
    shown: Option<Option<Download>>,
}

static STATE: OnceLock<Mutex<State>> = OnceLock::new();

fn state() -> &'static Mutex<State> {
    STATE.get_or_init(|| {
        Mutex::new(State {
            item: None,
            updated: Instant::now()
                .checked_sub(Duration::from_secs(3600))
                .unwrap_or_else(Instant::now),
            speed: 0.0,
            last_bytes: 0,
            last_at: Instant::now(),
            last_note: Instant::now(),
            shown: None,
        })
    })
}

fn lock() -> std::sync::MutexGuard<'static, State> {
    state().lock().unwrap_or_else(|e| e.into_inner())
}

#[derive(Deserialize)]
struct Payload {
    #[serde(default)]
    downloads: Vec<Item>,
}

#[derive(Deserialize, Clone)]
struct Item {
    #[serde(default)]
    id: i64,
    /// Final file name as Chrome knows it, possibly still its placeholder.
    #[serde(default)]
    name: String,
    #[serde(default)]
    bytes: u64,
    /// -1 when Chrome has not learned the size yet.
    #[serde(default)]
    total: i64,
    #[serde(default)]
    paused: bool,
}

fn ttl(item: &Option<Item>) -> Duration {
    if item.as_ref().is_some_and(|i| i.paused) {
        PAUSED_FRESH
    } else {
        ACTIVE_FRESH
    }
}

/// What the notch should show right now, or None when the bridge has gone quiet
/// and the disk watcher should decide.
fn current(s: &State) -> Option<Option<Download>> {
    if s.updated.elapsed() > ttl(&s.item) {
        return None;
    }
    let Some(item) = &s.item else {
        return Some(None);
    };
    Some(Some(Download {
        name: display_name(&item.name),
        bytes: item.bytes,
        speed: if item.paused { 0 } else { s.speed as u64 },
        total: item.total.max(0) as u64,
        paused: item.paused,
    }))
}

fn display_name(path: &str) -> String {
    let base = path.rsplit(['\\', '/']).next().unwrap_or(path);
    if base.is_empty() {
        return "Mengunduh…".to_string();
    }
    let stem = base.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(base);
    if downloads::placeholder(stem) {
        "Mengunduh…".to_string()
    } else {
        base.to_string()
    }
}

/// True while the extension is reporting; the disk watcher then stays quiet.
pub fn fresh() -> bool {
    let s = lock();
    s.updated.elapsed() <= ttl(&s.item)
}

/// Latest state for the webview after a reload, while the bridge is live.
#[tauri::command]
pub fn download_state() -> Option<Download> {
    let s = lock();
    current(&s).flatten()
}

fn apply(payload: Payload) -> Option<Option<Download>> {
    let now = Instant::now();
    let mut s = lock();
    // If the bridge had gone quiet, the disk watcher may have shown something
    // else in the meantime. Forget the last published state so the same payload
    // is emitted again instead of being deduped.
    if s.updated.elapsed() > ttl(&s.item) {
        s.shown = None;
    }
    let Some(mut item) = payload.downloads.into_iter().max_by_key(|i| i.bytes) else {
        if let Some(prev) = s.item.take() {
            crate::log_line(&format!(
                "bridge: selesai {} ({} bytes)",
                display_name(&prev.name),
                prev.bytes
            ));
        }
        s.updated = now;
        s.speed = 0.0;
        return Some(None);
    };
    item.total = item.total.max(0);
    // Only the interesting transitions go to the log, so a download that runs
    // for an hour is a handful of lines instead of thousands.
    match &s.item {
        Some(prev) if prev.id == item.id => {
            if prev.total == 0 && item.total > 0 {
                crate::log_line(&format!(
                    "bridge: total {} = {} bytes",
                    display_name(&item.name),
                    item.total
                ));
            }
            if prev.paused != item.paused {
                crate::log_line(&format!(
                    "bridge: {} {} pada {} / {}",
                    if item.paused { "dijeda" } else { "dilanjutkan" },
                    display_name(&item.name),
                    item.bytes,
                    item.total
                ));
            }
        }
        _ => crate::log_line(&format!(
            "bridge: mulai {} ({} / {} bytes, total {})",
            display_name(&item.name),
            item.bytes,
            item.total,
            if item.total > 0 { "diketahui" } else { "belum" }
        )),
    }
    // Every now and then while it is moving, so a report of "no percentage"
    // can be checked against what the extension actually sent.
    if !item.paused && s.last_note.elapsed() >= Duration::from_secs(5) {
        crate::log_line(&format!(
            "bridge: progres {} {} / {}",
            display_name(&item.name),
            item.bytes,
            item.total
        ));
        s.last_note = now;
    }
    if s.item.as_ref().is_some_and(|prev| prev.id == item.id) {
        let dt = now.duration_since(s.last_at).as_secs_f64();
        if dt >= 0.05 {
            let instant = item.bytes.saturating_sub(s.last_bytes) as f64 / dt;
            s.speed = if s.speed <= 0.0 {
                instant
            } else {
                s.speed * 0.65 + instant * 0.35
            };
        }
    } else {
        // A different download: the previous speed says nothing about this one.
        s.speed = 0.0;
    }
    s.last_bytes = item.bytes;
    s.last_at = now;
    s.item = Some(item);
    s.updated = now;
    current(&s)
}

fn publish(app: &AppHandle, current: Option<Option<Download>>) {
    let Some(current) = current else { return };
    {
        let mut s = lock();
        if s.shown.as_ref() == Some(&current) {
            return;
        }
        s.shown = Some(current.clone());
    }
    let _ = app.emit("download", &current);
}

fn respond(stream: &mut TcpStream, status: &str) {
    let _ = write!(
        stream,
        "HTTP/1.1 {status}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: POST, OPTIONS\r\n\
         Access-Control-Allow-Headers: content-type\r\n\
         Access-Control-Allow-Private-Network: true\r\n\
         Content-Length: 0\r\n\
         Connection: close\r\n\r\n"
    );
}

fn handle(mut stream: TcpStream, app: AppHandle) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let header_end = loop {
        match stream.read(&mut chunk) {
            Ok(0) => break None,
            Ok(n) => {
                buf.extend_from_slice(&chunk[..n]);
                if let Some(at) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                    break Some(at + 4);
                }
                if buf.len() > MAX_BODY {
                    break None;
                }
            }
            Err(_) => break None,
        }
    };
    let Some(end) = header_end else { return };
    let head = String::from_utf8_lossy(&buf[..end]).to_string();
    let mut lines = head.lines();
    let method = lines
        .next()
        .and_then(|line| line.split_whitespace().next())
        .unwrap_or("");
    let mut length = 0usize;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else { continue };
        if name.eq_ignore_ascii_case("content-length") {
            length = value.trim().parse().unwrap_or(0);
        }
    }
    if method == "OPTIONS" {
        respond(&mut stream, "204 No Content");
        return;
    }
    if method != "POST" {
        respond(&mut stream, "405 Method Not Allowed");
        return;
    }
    let mut body = buf[end..].to_vec();
    while body.len() < length && body.len() <= MAX_BODY {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => body.extend_from_slice(&chunk[..n]),
            Err(_) => break,
        }
    }
    if body.len() <= MAX_BODY {
        if let Ok(payload) = serde_json::from_slice::<Payload>(&body) {
            publish(&app, apply(payload));
        }
    }
    respond(&mut stream, "204 No Content");
}

pub fn install(app: AppHandle) {
    std::thread::spawn(move || {
        let Ok(listener) = TcpListener::bind(("127.0.0.1", PORT)) else {
            crate::log_line(&format!("bridge: port {PORT} sedang dipakai, ekstensi tidak tersambung"));
            return;
        };
        crate::log_line(&format!("bridge: mendengarkan di 127.0.0.1:{PORT}"));
        for stream in listener.incoming().flatten() {
            let app = app.clone();
            std::thread::spawn(move || handle(stream, app));
        }
    });
}
