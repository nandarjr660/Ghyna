//! Small JSON files in %APPDATA%\Ghyna for tasks, water log and shelf, plus
//! what the shelf needs: opening files and letting the notch preview them.

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use tauri::{AppHandle, Manager};

fn file(name: &str) -> Result<PathBuf, String> {
    // Only plain names, so nothing outside the Ghyna folder can be touched.
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err("invalid name".into());
    }
    Ok(crate::data_dir().join(format!("{name}.json")))
}

#[tauri::command]
pub fn load_data(name: String) -> Result<serde_json::Value, String> {
    let path = file(&name)?;
    Ok(std::fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(serde_json::Value::Null))
}

#[tauri::command]
pub fn save_data(name: String, value: serde_json::Value) -> Result<(), String> {
    let path = file(&name)?;
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(path, serde_json::to_string(&value).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

/// Lets the notch show previews of files on the shelf, and returns their sizes in bytes.
#[tauri::command]
pub fn shelf_add(app: AppHandle, paths: Vec<String>) -> Vec<Option<u64>> {
    paths
        .iter()
        .map(|p| {
            let meta = std::fs::metadata(p).ok()?;
            // Asset protocol scope no longer requires explicit allow_file in Tauri v2
            Some(if meta.is_dir() { 0 } else { meta.len() })
        })
        .collect()
}

/// Deletes files Ghyna copied for the shelf (browser images). Anything outside
/// `%APPDATA%\Ghyna\Shelf` is left alone: those are the user's own files.
#[tauri::command]
pub fn shelf_discard(paths: Vec<String>) {
    let root = crate::data_dir().join("Shelf");
    for path in paths {
        let path = PathBuf::from(path);
        if path.starts_with(&root) && path.is_file() {
            let _ = std::fs::remove_file(path);
        }
    }
}

/// Removes shelf copies that are no longer listed, e.g. after a drag-out.
/// Only files older than ten minutes go: a chat app may still be reading one
/// that was just dropped into it.
#[tauri::command]
pub fn shelf_gc(keep: Vec<String>) {
    let root = crate::data_dir().join("Shelf");
    let keep: HashSet<String> = keep.iter().map(|p| p.to_lowercase()).collect();
    let Some(cutoff) = SystemTime::now().checked_sub(Duration::from_secs(600)) else { return };
    let Ok(entries) = std::fs::read_dir(&root) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() || keep.contains(&path.to_string_lossy().to_lowercase()) {
            continue;
        }
        let old = entry.metadata().and_then(|m| m.modified()).is_ok_and(|t| t < cutoff);
        if old {
            let _ = std::fs::remove_file(path);
        }
    }
}

/// Opens a file or folder the user put on the shelf.
#[tauri::command]
pub fn open_path(path: String) {
    if std::path::Path::new(&path).exists() {
        let _ = std::process::Command::new("explorer").arg(&path).spawn();
    }
}
