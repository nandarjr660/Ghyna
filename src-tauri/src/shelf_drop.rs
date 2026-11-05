//! Drop target for the notch.
//!
//! Windows' own drag-and-drop handler (the one wry installs) only understands
//! `CF_HDROP`: real files with a path on disk. Browsers hand a dragged image
//! over differently — Chromium writes it to `%TEMP%\chrome_drag…` and only does
//! so after the target asks for an async operation — so wry rejects the drag
//! before Ghyna ever hears about it.
//!
//! This module replaces that handler with one of our own. It still accepts plain
//! files from Explorer (untouched paths), but for browser images it waits for
//! the temporary file, copies it into `%APPDATA%\Ghyna\Shelf` before the drag
//! ends (Chromium deletes the temp folder right after), and falls back to the
//! image URL when no file is offered.
//!
//! The events use the same shape Tauri used, so the notch's drop-zone logic
//! (`ghyna-drag` in `+page.svelte`) stays unchanged.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use windows::core::{implement, HSTRING, Interface, Ref, Result};
use windows::Foundation::Uri;
use windows::Storage::Streams::DataReader;
use windows::Web::Http::HttpClient;
use windows::Win32::Foundation::{HWND, POINT, POINTL};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::System::Com::{
    CoInitializeEx, CoUninitialize, IDataObject, COINIT_MULTITHREADED, DVASPECT_CONTENT, FORMATETC,
    TYMED_HGLOBAL,
};
use windows::Win32::System::DataExchange::RegisterClipboardFormatW;
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
use windows::Win32::System::Ole::{
    IDropTarget, IDropTarget_Impl, RegisterDragDrop, ReleaseStgMedium, RevokeDragDrop, CF_HDROP,
    CF_UNICODETEXT, DROPEFFECT, DROPEFFECT_COPY, DROPEFFECT_NONE,
};
use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;
use windows::Win32::UI::Shell::{DragQueryFileW, HDROP, IDataObjectAsyncCapability};
use windows::Win32::UI::WindowsAndMessaging::{GetWindow, GW_CHILD, GW_HWNDNEXT};

#[derive(Clone, Serialize)]
struct DragEvent {
    #[serde(rename = "type")]
    kind: &'static str,
    x: i32,
    y: i32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    paths: Vec<String>,
}

fn event(kind: &'static str, hwnd: HWND, pt: &POINTL, paths: Vec<String>) -> DragEvent {
    // OLE reports screen coordinates; the frontend hit-tests with webview
    // coordinates divided by devicePixelRatio.
    let mut client = POINT { x: pt.x, y: pt.y };
    unsafe {
        let _ = ScreenToClient(hwnd, &mut client);
    }
    DragEvent { kind, x: client.x, y: client.y, paths }
}

fn format(cf: u16) -> FORMATETC {
    FORMATETC {
        cfFormat: cf,
        ptd: std::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT.0,
        lindex: -1,
        tymed: TYMED_HGLOBAL.0 as u32,
    }
}

fn clipboard_format(name: &str) -> u16 {
    // Registered formats all live below 0xC000, so they fit in a WORD.
    unsafe { RegisterClipboardFormatW(&HSTRING::from(name)) as u16 }
}

/// Chromium only materialises its temporary file after this call.
fn begin_async(obj: &IDataObject) {
    if let Ok(cap) = obj.cast::<IDataObjectAsyncCapability>() {
        unsafe {
            let _ = cap.StartOperation(None::<&windows::Win32::System::Com::IBindCtx>);
        }
    }
}

fn can_get(obj: &IDataObject, cf: u16) -> bool {
    unsafe { obj.QueryGetData(&format(cf)).is_ok() }
}

fn accepts(obj: &IDataObject) -> bool {
    can_get(obj, CF_HDROP.0)
        || can_get(obj, clipboard_format("text/uri-list"))
        || can_get(obj, clipboard_format("HTML Format"))
        || can_get(obj, CF_UNICODETEXT.0)
}

fn hdrop_paths(obj: &IDataObject) -> Vec<String> {
    let Ok(mut medium) = (unsafe { obj.GetData(&format(CF_HDROP.0)) }) else {
        return Vec::new();
    };
    let mut paths = Vec::new();
    unsafe {
        let drop = HDROP(medium.u.hGlobal.0);
        let count = DragQueryFileW(drop, u32::MAX, None);
        for i in 0..count {
            let mut buf = [0u16; 1024];
            let len = DragQueryFileW(drop, i, Some(&mut buf));
            if len > 0 {
                paths.push(String::from_utf16_lossy(&buf[..len as usize]));
            }
        }
        ReleaseStgMedium(&mut medium);
    }
    paths
}

fn get_string(obj: &IDataObject, cf: u16) -> Option<String> {
    let Ok(mut medium) = (unsafe { obj.GetData(&format(cf)) }) else {
        return None;
    };
    let text = unsafe {
        let global = medium.u.hGlobal;
        let ptr = GlobalLock(global);
        if ptr.is_null() {
            ReleaseStgMedium(&mut medium);
            return None;
        }
        let size = GlobalSize(global);
        let bytes = std::slice::from_raw_parts(ptr as *const u8, size).to_vec();
        let _ = GlobalUnlock(global);
        ReleaseStgMedium(&mut medium);
        decode(&bytes)
    };
    Some(text)
}

fn decode(bytes: &[u8]) -> String {
    // CF_UNICODETEXT is UTF-16; HTML/uri-list formats are UTF-8.
    let utf16 = bytes.len() >= 4
        && bytes.len() % 2 == 0
        && bytes.chunks_exact(2).filter(|c| c[1] == 0).count() > bytes.len() / 6;
    let text = if utf16 {
        let units: Vec<u16> = bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        String::from_utf16_lossy(&units)
    } else {
        String::from_utf8_lossy(bytes).into_owned()
    };
    text.trim_matches('\0').trim().to_string()
}

fn attr_value(html: &str, name: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let mut from = 0;
    while let Some(i) = lower[from..].find(name) {
        let start = from + i + name.len();
        let rest = html[start..].trim_start();
        if let Some(rest) = rest.strip_prefix('=') {
            let rest = rest.trim_start();
            let quote = rest.chars().next()?;
            if quote == '"' || quote == '\'' {
                let end = rest[1..].find(quote)? + 1;
                return Some(rest[1..end].to_string());
            }
        }
        from = start;
    }
    None
}

fn best_srcset(value: &str) -> Option<String> {
    let mut best: Option<(f32, String)> = None;
    for part in value.split(',') {
        let mut words = part.trim().split_whitespace();
        let Some(url) = words.next() else { continue };
        if url.is_empty() {
            continue;
        }
        let weight = words
            .next()
            .and_then(|d| d.trim_end_matches(|c: char| c.is_ascii_alphabetic()).parse::<f32>().ok())
            .unwrap_or(1.0);
        if best.as_ref().is_none_or(|(w, _)| weight > *w) {
            best = Some((weight, url.to_string()));
        }
    }
    best.map(|(_, url)| url)
}

fn best_from_html(html: &str) -> Option<String> {
    if let Some(srcset) = attr_value(html, "srcset") {
        if let Some(url) = best_srcset(&srcset) {
            return Some(url);
        }
    }
    attr_value(html, "src").or_else(|| {
        let href = attr_value(html, "href")?;
        is_image_url(&href).then_some(href)
    })
}

fn first_url(text: &str) -> Option<String> {
    text.split_whitespace()
        .map(|t| t.trim_matches(|c: char| matches!(c, '"' | '\'' | ')' | ']' | '>' | ',')))
        .find(|t| t.starts_with("http://") || t.starts_with("https://"))
        .map(str::to_string)
}

fn image_url(obj: &IDataObject) -> Option<String> {
    if let Some(list) = get_string(obj, clipboard_format("text/uri-list")) {
        if let Some(url) = list
            .lines()
            .map(str::trim)
            .find(|line| !line.starts_with('#') && (line.starts_with("http://") || line.starts_with("https://")))
        {
            return Some(url.to_string());
        }
    }
    if let Some(html) = get_string(obj, clipboard_format("HTML Format")) {
        if let Some(url) = best_from_html(&html) {
            return Some(url);
        }
    }
    get_string(obj, CF_UNICODETEXT.0).and_then(|text| first_url(&text))
}

fn shelf_dir() -> PathBuf {
    crate::data_dir().join("Shelf")
}

fn is_temp(path: &Path) -> bool {
    let text = path.to_string_lossy().to_ascii_lowercase();
    let temp = std::env::temp_dir().to_string_lossy().to_ascii_lowercase();
    text.starts_with(&temp) || text.contains("\\temp\\") || text.contains("chrome_drag")
}

fn sanitize(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c => c,
        })
        .collect();
    let cleaned = cleaned.trim().trim_matches('.').to_string();
    if cleaned.is_empty() { "gambar".into() } else { cleaned }
}

fn read_head(path: &Path) -> Vec<u8> {
    let mut head = Vec::new();
    if let Ok(file) = std::fs::File::open(path) {
        let _ = file.take(64).read_to_end(&mut head);
    }
    head
}

fn sniff(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        Some("png")
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("jpg")
    } else if bytes.starts_with(b"GIF8") {
        Some("gif")
    } else if bytes.len() > 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("webp")
    } else if bytes.starts_with(b"BM") {
        Some("bmp")
    } else if bytes.len() > 12 && &bytes[4..8] == b"ftyp" {
        Some("avif")
    } else if bytes.starts_with(b"<svg") || (bytes.starts_with(b"<?xml") && bytes.windows(4).any(|w| w == b"<svg")) {
        Some("svg")
    } else {
        None
    }
}

fn is_image_url(url: &str) -> bool {
    let path = url.split(['?', '#']).next().unwrap_or(url).to_ascii_lowercase();
    ["png", "jpg", "jpeg", "gif", "webp", "bmp", "avif", "svg"]
        .iter()
        .any(|ext| path.ends_with(&format!(".{ext}")))
}

fn unique_path(dir: &Path, name: &str) -> PathBuf {
    let candidate = dir.join(name);
    if !candidate.exists() {
        return candidate;
    }
    let stem = Path::new(name).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "gambar".into());
    let ext = Path::new(name).extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    for n in 2..1000 {
        let candidate = dir.join(format!("{stem} ({n}){ext}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    dir.join(format!("{stem} (baru){ext}"))
}

/// Copies a browser's temporary image into Ghyna's own folder, because the
/// original folder disappears as soon as the drag ends.
fn copy_into_shelf(src: &Path) -> Option<PathBuf> {
    let dir = shelf_dir();
    std::fs::create_dir_all(&dir).ok()?;
    let mut name = sanitize(&src.file_name()?.to_string_lossy());
    if Path::new(&name).extension().is_none() {
        if let Some(ext) = sniff(&read_head(src)) {
            name.push('.');
            name.push_str(ext);
        }
    }
    let dest = unique_path(&dir, &name);
    std::fs::copy(src, &dest).ok()?;
    Some(dest)
}

fn name_from_url(url: &str) -> String {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let last = path.rsplit('/').next().filter(|s| !s.is_empty()).unwrap_or("gambar");
    let name = sanitize(last);
    if Path::new(&name).extension().is_some() { name } else { format!("{name}.png") }
}

fn download(url: &str) -> Option<PathBuf> {
    let uri = Uri::CreateUri(&HSTRING::from(url)).ok()?;
    let client = HttpClient::new().ok()?;
    let buffer = client.GetBufferAsync(&uri).ok()?.get().ok()?;
    let size = buffer.Length().ok()? as usize;
    if size == 0 || size > 40 * 1024 * 1024 {
        return None;
    }
    let reader = DataReader::FromBuffer(&buffer).ok()?;
    let mut bytes = vec![0u8; size];
    reader.ReadBytes(&mut bytes).ok()?;
    let ext = sniff(&bytes).or_else(|| {
        let path = url.split(['?', '#']).next().unwrap_or(url).to_ascii_lowercase();
        ["png", "jpg", "jpeg", "gif", "webp", "bmp", "avif", "svg"].into_iter().find(|e| path.ends_with(&format!(".{e}")))
    })?;
    let mut name = name_from_url(url);
    // Trust the bytes over the URL when they disagree.
    if Path::new(&name).extension().is_none_or(|e| !e.to_string_lossy().eq_ignore_ascii_case(ext)) {
        let stem = Path::new(&name).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "gambar".into());
        name = format!("{stem}.{ext}");
    }
    let dir = shelf_dir();
    std::fs::create_dir_all(&dir).ok()?;
    let dest = unique_path(&dir, &name);
    std::fs::write(&dest, bytes).ok()?;
    Some(dest)
}

/// URL drags have to be downloaded after `Drop` returns, when the data object
/// is already gone. The file is announced with the same event.
fn fetch_later(app: AppHandle, url: String) {
    std::thread::spawn(move || {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        if let Some(path) = download(&url) {
            let _ = app.emit(
                "ghyna-drag",
                DragEvent { kind: "paths", x: 0, y: 0, paths: vec![path.to_string_lossy().into_owned()] },
            );
        }
        unsafe {
            CoUninitialize();
        }
    });
}

#[implement(IDropTarget)]
struct ShelfDrop {
    app: AppHandle,
    hwnd: HWND,
}

impl IDropTarget_Impl for ShelfDrop_Impl {
    fn DragEnter(
        &self,
        pdataobj: Ref<'_, IDataObject>,
        _keys: MODIFIERKEYS_FLAGS,
        pt: &POINTL,
        effect: *mut DROPEFFECT,
    ) -> Result<()> {
        let mut chosen = DROPEFFECT_NONE;
        if let Some(obj) = pdataobj.as_ref() {
            begin_async(obj);
            if accepts(obj) {
                chosen = DROPEFFECT_COPY;
            }
        }
        unsafe {
            *effect = chosen;
        }
        if chosen.0 == DROPEFFECT_COPY.0 {
            let _ = self.app.emit("ghyna-drag", event("enter", self.hwnd, pt, Vec::new()));
        }
        Ok(())
    }

    fn DragOver(&self, _keys: MODIFIERKEYS_FLAGS, pt: &POINTL, effect: *mut DROPEFFECT) -> Result<()> {
        unsafe {
            *effect = DROPEFFECT_COPY;
        }
        let _ = self.app.emit("ghyna-drag", event("over", self.hwnd, pt, Vec::new()));
        Ok(())
    }

    fn DragLeave(&self) -> Result<()> {
        let _ = self.app.emit(
            "ghyna-drag",
            DragEvent { kind: "leave", x: 0, y: 0, paths: Vec::new() },
        );
        Ok(())
    }

    fn Drop(
        &self,
        pdataobj: Ref<'_, IDataObject>,
        _keys: MODIFIERKEYS_FLAGS,
        pt: &POINTL,
        effect: *mut DROPEFFECT,
    ) -> Result<()> {
        unsafe {
            *effect = DROPEFFECT_COPY;
        }
        let mut saved: Vec<PathBuf> = Vec::new();
        let mut url = None;
        if let Some(obj) = pdataobj.as_ref() {
            begin_async(obj);
            // Chromium writes the temp file asynchronously; give it a moment.
            let mut paths = Vec::new();
            for _ in 0..10 {
                paths = hdrop_paths(obj);
                if !paths.is_empty() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(30));
            }
            for path in paths {
                let path = PathBuf::from(path);
                if path.is_file() && is_temp(&path) {
                    if let Some(copied) = copy_into_shelf(&path) {
                        saved.push(copied);
                    }
                } else if path.exists() {
                    saved.push(path);
                }
            }
            if saved.is_empty() {
                url = image_url(obj);
            }
        }
        let paths: Vec<String> = saved.iter().map(|p| p.to_string_lossy().into_owned()).collect();
        if let Some(url) = url {
            fetch_later(self.app.clone(), url);
        }
        let _ = self.app.emit("ghyna-drag", event("drop", self.hwnd, pt, paths));
        Ok(())
    }
}

struct Targets {
    app: AppHandle,
    kept: Vec<IDropTarget>,
}

unsafe fn replace_tree(parent: HWND, targets: &mut Targets) {
    let mut child = unsafe { GetWindow(parent, GW_CHILD) }.ok();
    while let Some(hwnd) = child {
        // wry already put its own (CF_HDROP-only) target here.
        unsafe {
            let _ = RevokeDragDrop(hwnd);
        }
        let target: IDropTarget = ShelfDrop { app: targets.app.clone(), hwnd }.into();
        if unsafe { RegisterDragDrop(hwnd, &target) }.is_ok() {
            targets.kept.push(target);
        }
        unsafe {
            replace_tree(hwnd, targets);
        }
        child = unsafe { GetWindow(hwnd, GW_HWNDNEXT) }.ok();
    }
}

fn replace_targets(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else { return };
    let Ok(raw) = window.hwnd() else { return };
    let mut targets = Targets { app: app.clone(), kept: Vec::new() };
    let main = HWND(raw.0);
    unsafe {
        let _ = RevokeDragDrop(main);
    }
    let target: IDropTarget = ShelfDrop { app: app.clone(), hwnd: main }.into();
    if unsafe { RegisterDragDrop(main, &target) }.is_ok() {
        targets.kept.push(target);
    }
    unsafe {
        replace_tree(main, &mut targets);
    }
    // The COM objects must outlive the registration.
    std::mem::forget(targets.kept);
}

/// Called once at startup. WebView2 may create its child window a little late,
/// so the replacement is retried on the UI thread a couple of times.
pub fn install(app: &AppHandle) {
    replace_targets(app);
    for delay in [900u64, 2400] {
        let app = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(delay));
            let handle = app.clone();
            let _ = app.run_on_main_thread(move || replace_targets(&handle));
        });
    }
}
