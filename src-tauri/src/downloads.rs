//! Browser downloads in progress, from the Downloads folder: Chromium browsers
//! write "name.crdownload", Firefox "name.part", and rename to "name" when done.
//!
//! A download can be saved anywhere through the browser's "Save as" dialog, so
//! the Downloads folder alone is not enough. Browser history records the real
//! target path (and the final size), so recent folders are watched too. The
//! folder watcher wakes a once-a-second pass, which runs only while such files
//! exist. When history knows the final size the notch shows a real percentage;
//! otherwise it falls back to the size so far and the speed.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant, UNIX_EPOCH};

use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use serde_json::json;
use tauri::{AppHandle, Emitter};
use windows::core::GUID;
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::{FOLDERID_Downloads, SHGetKnownFolderPath, KF_FLAG_DEFAULT};

use crate::peek;

#[derive(Serialize, Clone, PartialEq, Debug)]
pub struct Download {
    /// File name as it will be, e.g. "Laporan.pdf".
    name: String,
    bytes: u64,
    /// Bytes per second over the last pass.
    speed: u64,
    /// Final size when the browser history knows it, else 0.
    total: u64,
}

pub fn downloads_dir() -> Option<PathBuf> {
    unsafe {
        let p = SHGetKnownFolderPath(&FOLDERID_Downloads as *const GUID, KF_FLAG_DEFAULT, None).ok()?;
        let path = p.to_string().ok().map(PathBuf::from);
        CoTaskMemFree(Some(p.0 as _));
        path
    }
}

fn same_path(a: &Path, b: &Path) -> bool {
    a.to_string_lossy().eq_ignore_ascii_case(&b.to_string_lossy())
}

fn push_dir(dirs: &mut Vec<PathBuf>, dir: PathBuf) {
    if !dir.is_dir() {
        return;
    }
    if !dirs.iter().any(|existing| same_path(existing, &dir)) {
        dirs.push(dir);
    }
}

/// Chromium profiles keep their history in a file called `History`.
fn history_files() -> Vec<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let roots = [
        local.clone().map(|p| p.join(r"Google\Chrome\User Data")),
        local.clone().map(|p| p.join(r"Microsoft\Edge\User Data")),
        local.map(|p| p.join(r"BraveSoftware\Brave-Browser\User Data")),
    ];
    let mut files = Vec::new();
    for root in roots.into_iter().flatten() {
        let Ok(entries) = fs::read_dir(root) else { continue };
        for entry in entries.flatten() {
            let profile = entry.path();
            if !profile.is_dir() {
                continue;
            }
            let history = profile.join("History");
            if history.is_file() {
                files.push(history);
            }
        }
    }
    files
}

fn base_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(dir) = downloads_dir() {
        push_dir(&mut dirs, dir);
    }

    // Chromium stores the user's custom download directory in each profile's
    // Preferences file. This is often empty, but when set it is authoritative.
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    for root in [
        local.clone().map(|p| p.join(r"Google\Chrome\User Data")),
        local.clone().map(|p| p.join(r"Microsoft\Edge\User Data")),
        local.map(|p| p.join(r"BraveSoftware\Brave-Browser\User Data")),
    ].into_iter().flatten() {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let profile = entry.path();
                if !profile.is_dir() {
                    continue;
                }
                let prefs = profile.join("Preferences");
                let Ok(text) = fs::read_to_string(prefs) else { continue };
                let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else { continue };
                if let Some(path) = json.pointer("download.default_directory").and_then(|v| v.as_str()) {
                    push_dir(&mut dirs, PathBuf::from(path));
                }
            }
        }
    }
    dirs
}

/// A download row from a browser history database.
#[derive(Clone, Debug)]
struct Target {
    /// Chromium's current path; during a download this is usually the
    /// "name.crdownload" file, which is the exact path the watcher sees.
    current: PathBuf,
    /// The name the browser will give the finished file.
    target: PathBuf,
    /// Final size in bytes, or 0 when the browser does not know it.
    total: u64,
}

#[derive(Default, Clone, Debug)]
struct History {
    dirs: Vec<PathBuf>,
    targets: Vec<Target>,
}

impl History {
    /// Finds the history row for a partial file: first by its exact current
    /// path, then by its finished name in the same folder.
    fn resolve(&self, partial: &Path) -> Option<(&Path, u64)> {
        for target in &self.targets {
            if !target.current.as_os_str().is_empty() && same_path(&target.current, partial) {
                return Some((&target.target, target.total));
            }
        }

        let file = partial.file_name()?.to_string_lossy();
        let final_name = final_name(&file)??;
        for target in &self.targets {
            let Some(name) = target.target.file_name() else { continue };
            if !name.to_string_lossy().eq_ignore_ascii_case(&final_name) {
                continue;
            }
            let (Some(target_parent), Some(partial_parent)) = (target.target.parent(), partial.parent()) else {
                continue;
            };
            if same_path(target_parent, partial_parent) {
                return Some((&target.target, target.total));
            }
        }
        None
    }
}

type HistoryRow = (String, String, u64);

fn query_rows(conn: &Connection) -> rusqlite::Result<Vec<HistoryRow>> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(current_path, ''), COALESCE(target_path, ''), COALESCE(total_bytes, 0)
         FROM downloads ORDER BY start_time DESC LIMIT 80",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?.max(0) as u64,
        ))
    })?;
    rows.collect()
}

/// Older Chromium versions may not have `current_path`; the target and size
/// are still useful.
fn query_rows_compat(conn: &Connection) -> rusqlite::Result<Vec<HistoryRow>> {
    let mut stmt = conn.prepare(
        "SELECT '', COALESCE(target_path, ''), COALESCE(total_bytes, 0)
         FROM downloads ORDER BY start_time DESC LIMIT 80",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            String::new(),
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?.max(0) as u64,
        ))
    })?;
    rows.collect()
}

fn query_history(path: &Path) -> Option<Vec<HistoryRow>> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ).ok()?;
    let _ = conn.busy_timeout(Duration::from_millis(150));
    query_rows(&conn).or_else(|_| query_rows_compat(&conn)).ok()
}

/// A URI filename with `immutable=1` tells SQLite to ignore the browser's locks.
/// It skips the WAL, but it is cheap enough to poll for new finished downloads.
fn query_history_immutable(path: &Path) -> Option<Vec<HistoryRow>> {
    let text = path.to_string_lossy().replace('\\', "/");
    let text = text
        .replace('%', "%25")
        .replace(' ', "%20")
        .replace('#', "%23")
        .replace('?', "%3F");
    let uri = if text.starts_with('/') {
        format!("file://{text}?immutable=1")
    } else {
        format!("file:///{text}?immutable=1")
    };
    let conn = Connection::open_with_flags(
        &uri,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX | OpenFlags::SQLITE_OPEN_URI,
    ).ok()?;
    query_rows(&conn).or_else(|_| query_rows_compat(&conn)).ok()
}

/// Chrome keeps its history open. A read-only connection often works, but when
/// it does not, a copy of the database (plus its WAL sidecars) is safe to read.
/// The copy is 20+ MB, so it is only made on the periodic deep reads.
fn read_history(path: &Path, deep: bool) -> Option<Vec<HistoryRow>> {
    if !deep {
        if let Some(rows) = query_history(path) {
            return Some(rows);
        }
        if let Some(rows) = query_history_immutable(path) {
            return Some(rows);
        }
    }

    static NEXT: AtomicU64 = AtomicU64::new(0);
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let copy = std::env::temp_dir().join(format!("ghyna-history-{}-{n}.db", std::process::id()));
    fs::copy(path, &copy).ok()?;
    for suffix in ["-wal", "-shm"] {
        let from = PathBuf::from(format!("{}{suffix}", path.to_string_lossy()));
        if from.is_file() {
            let _ = fs::copy(&from, PathBuf::from(format!("{}{suffix}", copy.to_string_lossy())));
        }
    }

    let rows = Connection::open_with_flags(&copy, OpenFlags::SQLITE_OPEN_READ_ONLY).ok().and_then(|conn| {
        let _ = conn.busy_timeout(Duration::from_millis(150));
        query_rows(&conn).or_else(|_| query_rows_compat(&conn)).ok()
    });
    for suffix in ["", "-wal", "-shm"] {
        let _ = fs::remove_file(PathBuf::from(format!("{}{suffix}", copy.to_string_lossy())));
    }
    rows
}

fn history_index(deep: bool) -> History {
    let mut all = History::default();
    for file in history_files() {
        let Some(rows) = read_history(&file, deep) else { continue };
        for (current, target, total) in rows {
            if target.is_empty() {
                continue;
            }
            let target = PathBuf::from(target);
            if let Some(parent) = target.parent() {
                if !parent.as_os_str().is_empty() {
                    push_dir(&mut all.dirs, parent.to_path_buf());
                }
            }
            let current = if current.is_empty() { PathBuf::new() } else { PathBuf::from(current) };
            all.targets.push(Target { current, target, total });
        }
    }
    all
}

/// Modification times and sizes of every history database and its WAL files.
/// Polling this is cheap; reading the database only happens when it changes.
/// `-shm` is deliberately left out: SQLite touches it for every read, which
/// would make the stamp look changed even when no download was added.
fn history_stamp() -> Vec<(PathBuf, u128, u64)> {
    let mut stamps = Vec::new();
    for path in history_files() {
        for suffix in ["", "-wal"] {
            let path = if suffix.is_empty() {
                path.clone()
            } else {
                PathBuf::from(format!("{}{suffix}", path.to_string_lossy()))
            };
            let Ok(meta) = fs::metadata(&path) else { continue };
            let modified = meta
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            stamps.push((path, modified, meta.len()));
        }
    }
    stamps
}

fn watched_dirs() -> Vec<PathBuf> {
    let mut dirs = base_dirs();
    for dir in history_index(true).dirs {
        push_dir(&mut dirs, dir);
    }
    dirs
}

/// The finished file's name for a partial download, or None if it isn't one.
/// Chromium's first name, "Unconfirmed 123456.crdownload", has no real name yet.
pub fn final_name(partial: &str) -> Option<Option<String>> {
    let lower = partial.to_ascii_lowercase();
    let stem = [".crdownload", ".part", ".partial"]
        .iter()
        .find_map(|ext| lower.ends_with(ext).then(|| &partial[..partial.len() - ext.len()]))?;
    if stem.is_empty() {
        return None;
    }
    let unconfirmed = stem.starts_with("Unconfirmed ") && stem[12..].chars().all(|c| c.is_ascii_digit());
    Some((!unconfirmed).then(|| stem.to_string()))
}

fn partials(dirs: &[PathBuf]) -> HashMap<PathBuf, u64> {
    dirs.iter()
        .flat_map(|dir| fs::read_dir(dir).into_iter().flatten().flatten())
        .filter(|e| final_name(&e.file_name().to_string_lossy()).is_some())
        // The folder listing keeps the size from when the file was opened; asking
        // the file itself gives what the browser has written so far.
        .filter_map(|e| Some((e.path(), fs::metadata(e.path()).ok()?.len())))
        .collect()
}

fn describe(path: &Path, history: &History, bytes: u64, speed: u64) -> Download {
    let file = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    if let Some((target, total)) = history.resolve(path) {
        let name = target
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| final_name(&file).flatten().unwrap_or_else(|| "Mengunduh…".into()));
        return Download { name, bytes, speed, total };
    }
    let name = final_name(&file).flatten().unwrap_or_else(|| "Mengunduh…".into());
    Download { name, bytes, speed, total: 0 }
}

pub fn start(app: AppHandle) {
    use notify::{RecursiveMode, Watcher};
    let mut dirs = base_dirs();
    let mut history = history_index(true);
    for dir in &history.dirs {
        push_dir(&mut dirs, dir.clone());
    }
    if dirs.is_empty() {
        return;
    }
    std::thread::spawn(move || {
        let (tx, rx) = mpsc::channel();
        let Ok(mut watcher) = notify::recommended_watcher(tx) else { return };
        let mut watched: Vec<PathBuf> = Vec::new();
        for dir in &dirs {
            if watcher.watch(dir, RecursiveMode::NonRecursive).is_ok() {
                watched.push(dir.clone());
            }
        }
        if watched.is_empty() {
            return;
        }

        let mut seen: HashMap<PathBuf, Seen> = HashMap::new();
        let mut last_pass = Instant::now();
        let mut shown: Option<Download> = None;
        // Start empty so the first poll always re-reads history: a download may
        // have started between the initial read and the watcher becoming ready.
        let mut history_stamp_now = Vec::new();
        let mut last_history = Instant::now();
        let mut last_deep = Instant::now();

        loop {
            // Sleep until something happens in a watched folder; while a download
            // grows, look every second so the size and speed stay fresh.
            let busy = seen.values().any(|s| s.changed.elapsed() < KEEP_LOOKING);
            let wait = if busy { Duration::from_secs(1) } else { Duration::from_secs(2) };
            let event = rx.recv_timeout(wait).ok();
            if event.is_some() {
                // Writes fire events constantly; one pass a second is plenty and
                // keeps the speed honest.
                if let Some(wait) = Duration::from_millis(950).checked_sub(last_pass.elapsed()) {
                    std::thread::sleep(wait);
                }
                while rx.try_recv().is_ok() {}
            }

            // A download saved through "Save as" can land in a folder that was not
            // watched yet. Recent browser history names that folder (and the final
            // size); check it periodically without reading the database unless it
            // has actually changed.
            let every = if busy { Duration::from_secs(10) } else { Duration::from_secs(2) };
            // The cheap read skips the WAL, so a download started a moment ago can
            // sit only in the copy-on-write log. A full copy every so often catches
            // those rows without paying for a 20+ MB copy on every poll.
            let deep = last_deep.elapsed() >= DEEP_EVERY;
            if deep || last_history.elapsed() >= every {
                let next_stamp = history_stamp();
                if deep || next_stamp != history_stamp_now {
                    history_stamp_now = next_stamp;
                    history = history_index(deep);
                    for dir in &history.dirs {
                        push_dir(&mut dirs, dir.clone());
                        if !watched.iter().any(|existing| same_path(existing, dir))
                            && watcher.watch(dir, RecursiveMode::NonRecursive).is_ok()
                        {
                            watched.push(dir.clone());
                        }
                    }
                    if deep {
                        last_deep = Instant::now();
                    }
                    // A partial may already be there; look at it in this pass.
                    last_pass = Instant::now() - Duration::from_secs(1);
                }
                last_history = Instant::now();
            }

            let secs = last_pass.elapsed().as_secs_f64().max(0.2);
            last_pass = Instant::now();
            let step = step(&mut seen, partials(&dirs), Instant::now());

            // Partials that disappeared while in use: done if the final file is there, else cancelled.
            for gone in step.finished {
                let name = gone.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                if let Some(Some(final_name)) = final_name(&name) {
                    let path = gone.parent().unwrap_or_else(|| Path::new("")).join(&final_name);
                    if path.is_file() {
                        peek(&app, "download-done", json!({ "name": final_name, "path": path }));
                    }
                }
            }

            // The biggest download in progress is the one shown.
            let current = step.growing.iter().max_by_key(|(_, bytes, _)| *bytes).map(|(path, bytes, grew)| {
                describe(path, &history, *bytes, (*grew as f64 / secs) as u64)
            });
            if current != shown {
                let _ = app.emit("download", &current);
                shown = current;
            }
        }
    });
}

/// A partial download that hasn't grown for this long is a leftover (failed or
/// cancelled earlier), not a download in progress.
const STALE: Duration = Duration::from_secs(15);
/// How long after the last growth the folder is still looked at every second.
const KEEP_LOOKING: Duration = Duration::from_secs(60);
/// How often the history database is copied to read rows still in its WAL.
const DEEP_EVERY: Duration = Duration::from_secs(30);

pub struct Seen {
    bytes: u64,
    changed: Instant,
}

pub struct Step {
    /// In progress: path, size, bytes added since the last pass.
    pub growing: Vec<(PathBuf, u64, u64)>,
    /// Gone since the last pass after being written to recently.
    pub finished: Vec<PathBuf>,
}

/// Folds one look at the folders into what we know of each partial file. A file
/// first seen counts as old until it grows, so leftovers never show.
pub fn step(seen: &mut HashMap<PathBuf, Seen>, now: HashMap<PathBuf, u64>, at: Instant) -> Step {
    let finished = seen
        .iter()
        .filter(|(p, s)| !now.contains_key(*p) && at.duration_since(s.changed) < KEEP_LOOKING)
        .map(|(p, _)| p.clone())
        .collect();
    seen.retain(|p, _| now.contains_key(p));
    let mut growing = vec![];
    for (path, bytes) in now {
        // Back-dated so a leftover is neither shown nor reported finished when it goes.
        let entry = seen.entry(path.clone()).or_insert(Seen { bytes, changed: at.checked_sub(KEEP_LOOKING).unwrap_or(at) });
        let grew = bytes.saturating_sub(entry.bytes);
        if bytes != entry.bytes {
            entry.bytes = bytes;
            entry.changed = at;
        }
        if at.duration_since(entry.changed) < STALE {
            growing.push((path, bytes, grew));
        }
    }
    Step { growing, finished }
}

/// Opens a finished download. Only files in the Downloads folder or in a folder
/// that browser history says a recent download used.
#[tauri::command]
pub fn open_download(path: String) {
    let path = PathBuf::from(path);
    let valid = watched_dirs().iter().any(|dir| path.parent().map(|parent| same_path(parent, dir)).unwrap_or(false));
    if valid && path.is_file() {
        let _ = std::process::Command::new("explorer").arg(&path).spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sizes(list: &[(&str, u64)]) -> HashMap<PathBuf, u64> {
        list.iter().map(|(p, b)| (PathBuf::from(p), *b)).collect()
    }

    #[test]
    fn leftover_partials_never_show() {
        let mut seen = HashMap::new();
        let t0 = Instant::now() + Duration::from_secs(100);
        let old = sizes(&[("big.zip.crdownload", 223_000_000)]);
        assert!(step(&mut seen, old.clone(), t0).growing.is_empty());
        assert!(step(&mut seen, old, t0 + Duration::from_secs(1)).growing.is_empty());
    }

    #[test]
    fn growing_downloads_show_and_fade_when_they_stop() {
        let mut seen = HashMap::new();
        let t0 = Instant::now() + Duration::from_secs(100);
        step(&mut seen, sizes(&[("a.pdf.crdownload", 0)]), t0);
        let s = step(&mut seen, sizes(&[("a.pdf.crdownload", 3_000_000)]), t0 + Duration::from_secs(1));
        assert_eq!(s.growing, vec![(PathBuf::from("a.pdf.crdownload"), 3_000_000, 3_000_000)]);
        // Stuck for longer than STALE: no longer shown.
        let s = step(&mut seen, sizes(&[("a.pdf.crdownload", 3_000_000)]), t0 + Duration::from_secs(20));
        assert!(s.growing.is_empty());
    }

    #[test]
    fn finished_only_for_recent_downloads() {
        let mut seen = HashMap::new();
        let t0 = Instant::now() + Duration::from_secs(100);
        step(&mut seen, sizes(&[("a.pdf.crdownload", 0), ("old.zip.part", 5)]), t0);
        step(&mut seen, sizes(&[("a.pdf.crdownload", 9), ("old.zip.part", 5)]), t0 + Duration::from_secs(1));
        let s = step(&mut seen, sizes(&[]), t0 + Duration::from_secs(2));
        assert_eq!(s.finished, vec![PathBuf::from("a.pdf.crdownload")]);
        assert!(seen.is_empty());
    }

    #[test]
    fn names_partial_downloads() {
        assert_eq!(final_name("Laporan Q3.pdf.crdownload"), Some(Some("Laporan Q3.pdf".into())));
        assert_eq!(final_name("setup.exe.part"), Some(Some("setup.exe".into())));
        assert_eq!(final_name("video.MP4.CRDOWNLOAD"), Some(Some("video.MP4".into())));
        assert_eq!(final_name("Unconfirmed 482913.crdownload"), Some(None));
        assert_eq!(final_name("Unconfirmed plan.pdf.crdownload"), Some(Some("Unconfirmed plan.pdf".into())));
        assert_eq!(final_name("report.pdf"), None);
        assert_eq!(final_name(".crdownload"), None);
    }

    #[test]
    fn history_finds_total_by_partial_path() {
        let history = History {
            dirs: vec![],
            targets: vec![Target {
                current: PathBuf::from(r"D:\Unduhan\Laporan.pdf.crdownload"),
                target: PathBuf::from(r"D:\Unduhan\Laporan.pdf"),
                total: 4_200_000,
            }],
        };
        let (target, total) = history.resolve(Path::new(r"d:\unduhan\laporan.pdf.crdownload")).unwrap();
        assert_eq!(target, Path::new(r"D:\Unduhan\Laporan.pdf"));
        assert_eq!(total, 4_200_000);
    }

    #[test]
    fn history_falls_back_to_the_finished_name() {
        let history = History {
            dirs: vec![],
            targets: vec![Target {
                current: PathBuf::from(r"D:\Unduhan\Unconfirmed 9123.crdownload"),
                target: PathBuf::from(r"D:\Unduhan\Laporan.pdf"),
                total: 4_200_000,
            }],
        };
        let (target, total) = history.resolve(Path::new(r"D:\Unduhan\Laporan.pdf.crdownload")).unwrap();
        assert_eq!(target, Path::new(r"D:\Unduhan\Laporan.pdf"));
        assert_eq!(total, 4_200_000);
    }
}
