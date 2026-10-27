<p align="center"><b>Ghyna</b><br/>A Dynamic Island for Windows</p>

<p align="center">
  <a href="https://github.com/nandarjr660/Ghyna/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/nandarjr660/Ghyna?style=flat-square&color=000000&label=download" /></a>
  <img alt="Windows 10 and 11" src="https://img.shields.io/badge/Windows-10%20%26%2011-000000?style=flat-square&logo=windows" />
  <img alt="Built with Tauri" src="https://img.shields.io/badge/Tauri%202-Rust%20%2B%20Svelte-000000?style=flat-square&logo=tauri" />
  <a href="https://github.com/nandarjr660/Ghyna/releases"><img alt="Downloads" src="https://img.shields.io/github/downloads/nandarjr660/Ghyna/total?style=flat-square&color=000000" /></a>
</p>

<p align="center"><b>Your Windows laptop doesn't have a notch? Now it does.</b><br/>
A small black island at the top of your screen that shows what's happening (music, lyrics, notifications, timers) and grows when you hover it.</p>

<p align="center">
  <a href="https://github.com/nandarjr660/Ghyna/releases/latest"><b>⬇️ Download for Windows</b></a>
</p>

> **Ghyna is a fork of [Notchy](https://github.com/RamantyaDS/Notchy) by [RamantyaDS](https://github.com/RamantyaDS).**
> The original project, design, and code belong to them. This fork keeps the
> Notchy history, rebrands the app as Ghyna, and records every change in
> [`ghyna.md`](ghyna.md).

---

<table>
<tr>
<td width="38%" valign="top"><img src="docs/demo.gif" alt="Ghyna in action" width="100%" /></td>
<td valign="top">

### What it does

🎵 **Music from any player.** Spotify, YouTube in the browser, anything that shows up in Windows' media controls. Cover, progress, play/pause/skip, and the cover's color tints the notch.

🎤 **Live lyrics.** Synced lyrics appear line by line, even in the small notch while you work.

🔔 **Notifications, in the notch.** WhatsApp, Teams, Outlook… mirrored within half a second. Click one and it opens *that* chat or email, just like the real notification.

⏱️ **Focus timer**, 💧 **water reminder**, ✅ **to-do list** with reminders, 📁 **Shelf** to park files you'll drag somewhere else, 📅 **calendar** reminders from your calendar link.

🔊 **Quick status:** volume, brightness, Caps Lock, keyboard language, charging, Wi-Fi, headphones, camera and mic in use, mic mute, screenshot preview, weather, one-click audio output switch.

🫥 **Stays out of the way.** It tucks into a thin line when there's a tab or button right under it, hides for full-screen video and games, and dozes off when you're idle.

</td>
</tr>
</table>

<p align="center">
  <img src="docs/player.png" width="49%" alt="Music player with lyrics" />
  <img src="docs/timer.png" width="49%" alt="Focus timer" />
  <img src="docs/todo.png" width="49%" alt="To-do list" />
  <img src="docs/water.png" width="49%" alt="Water reminder" />
</p>

## Install

1. Download `Ghyna_x.y.z_x64-setup.exe` from the [latest release](https://github.com/nandarjr660/Ghyna/releases/latest).
2. Run it. Windows may say *"Windows protected your PC"* because the app isn't code-signed: click **More info → Run anyway**.
3. That's it. No admin rights, no account. Ghyna **updates itself** from now on.

**Tip:** turn on Windows' *Do not disturb* (there's a button in Ghyna's Settings) so notifications pop up only in the notch, not twice.

## Light and private

- **~1–2% CPU at rest**, a few % while music animates.
- **No account, no analytics.** Your to-dos, water log and settings stay in `%APPDATA%\Ghyna`.
- It only goes online for lyrics ([LRCLIB](https://lrclib.net)), weather ([Open-Meteo](https://open-meteo.com)), your own calendar link if you add one, and checking for updates on GitHub.

## Built with

[Tauri 2](https://tauri.app) · Rust ([windows-rs](https://github.com/microsoft/windows-rs)) for everything that talks to Windows · [Svelte 5](https://svelte.dev) for the notch itself · WebView2.

<details>
<summary>Build it yourself</summary>

Needs [Node.js](https://nodejs.org), [Rust](https://rustup.rs) and the MSVC build tools.

```bash
npm install
npm run tauri dev      # run with hot reload
npm run tauri build    # installer in src-tauri/target/release/bundle/nsis
```

Design notes and every decision behind the app are in [`ghyna.md`](ghyna.md) (in Indonesian).
</details>

## Fork and attribution

Ghyna is a fork of [Notchy](https://github.com/RamantyaDS/Notchy), created by
[RamantyaDS](https://github.com/RamantyaDS). The original repository and its
history are preserved here through GitHub's fork network. If you want the
original project, use [RamantyaDS/Notchy](https://github.com/RamantyaDS/Notchy).

---

<details>
<summary>🇮🇩 Bahasa Indonesia</summary>

**Laptop Windows-mu nggak punya notch? Sekarang punya.** Ghyna adalah pulau hitam kecil di atas layar yang menampilkan musik beserta lirik yang ikut jalan, notifikasi (klik langsung membuka chat-nya), timer fokus, pengingat minum, to-do, Shelf file, kalender, dan status sistem. Membesar saat di-hover, menyingkir saat menghalangi.

Unduh installer di [Releases](https://github.com/nandarjr660/Ghyna/releases/latest). Kalau muncul peringatan SmartScreen, klik **More info → Run anyway**. Setelah terpasang, Ghyna update sendiri. Data tetap di laptopmu, tanpa akun.

Ghyna adalah fork dari [Notchy](https://github.com/RamantyaDS/Notchy) karya [RamantyaDS](https://github.com/RamantyaDS).
</details>
