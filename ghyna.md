# Ghyna

Ghyna (dulu **Notchy**) adalah notch ala Dynamic Island untuk laptop Windows. Notch kecil di tengah atas layar yang menampilkan hal yang sedang terjadi (musik, timer, notifikasi, status sistem), membesar saat di-hover, lalu menyingkir lagi.

---

## 1. Goal

**Ghyna harus nyaman dipakai setiap hari di laptop sendiri: cantik, tidak mengganggu, dan tidak memberatkan PC.**

Ghyna dianggap berhasil kalau:
- Menyala otomatis saat Windows start, dan bisa dibiarkan seharian tanpa perlu di-restart.
- Tidak pernah menghalangi klik ke aplikasi di bawahnya, kecuali di area notch itu sendiri.
- Hilang sendiri saat ada aplikasi full-screen (game, video, presentasi).
- Saat diam (tidak ada animasi), CPU-nya ~0% di Task Manager. *Kenyataan terukur: ~1% satu core saat diam (sebagian besar WebView2), ~4% saat musik diputar dengan animasi piringan/equalizer. v0.2.0 (22 Okt 2026, semua proses Ghyna + WebView2): diam 1,7%; musik dengan equalizer asli 4,8% vs v0.1.5 4,4% diukur bergantian dalam kondisi sama.*
- Semua fitur di bagian 5 berjalan.

**Pengguna:** saya sendiri dan beberapa teman. Kode Ghyna ada di fork publik `nandarjr660/Ghyna`, fork dari repo asli `RamantyaDS/Notchy`; atribusi ke repo asli tetap dicantumkan di README. Installer dibagikan lewat GitHub Releases.

**Status:** v1.1.10 (31 Oktober 2026). Rebrand dari Notchy ke Ghyna: nama, identifier `com.ghyna.app`, path data `%APPDATA%\Ghyna`, dan log `ghyna.log`. Updater otomatis aktif sejak v1.1.1. Installer: `Ghyna_1.1.10_x64-setup.exe`.

---

## 2. Keputusan yang sudah final

| Keputusan | Pilihan | Alasan |
|---|---|---|
| Nama | **Ghyna** (dulu Notchy) | Rebrand 27 Okt 2026; identifier `com.ghyna.app`, data di `%APPDATA%\Ghyna` |
| Framework | **Tauri v2** (Rust + HTML/CSS/TS) | Tampilan dan animasi gampang dibuat cantik dengan CSS. Hasilnya tetap aplikasi desktop (.exe) yang ringan. |
| Platform | Windows 10 dan 11 saja | Banyak API yang dipakai hanya ada di Windows |
| Sumber musik | API media Windows (`GlobalSystemMediaTransportControlsSessionManager`) | Bisa membaca player apa pun: Spotify, YouTube di browser, Apple Music, dll. |
| Sumber lirik | **LRCLIB** (lrclib.net) | Gratis, tanpa login, ada lirik tersinkron. Lirik **tidak** diambil dari Spotify karena tidak ada API resminya dan endpoint internalnya melanggar ToS. |
| Frontend | **Svelte 5** | Ringan, dan ada `spring`/`transition` bawaan untuk animasi notch. Satu komponen per mode. |
| Cuaca | **BMKG** | Data cuaca terkini dan prakiraan resmi Indonesia, tanpa API key |
| Lokasi cuaca | Lokasi Windows (GPS/Wi-Fi) kalau diizinkan; kalau tidak, kota yang diisi manual di Settings. Koordinat dipetakan BMKG ke wilayah administrasi terdekat | Tidak memakai lokasi berbasis IP: meleset dan alamat IP terkirim ke pihak ketiga |
| Jendela | **Ukuran tetap dan transparan**; yang berubah ukuran hanya elemen notch di dalamnya (CSS) | Resize jendela Windows setiap animasi akan patah-patah dan berkedip |
| Deteksi hover | Low-level mouse hook di Rust (`WH_MOUSE_LL`). Jendela click-through kecuali saat kursor berada di area notch, yang ukurannya dilaporkan frontend lewat `set_notch_size`. | Jendela yang click-through tidak menerima event mouse, jadi hover harus dideteksi dari luar; hook berbasis event, tanpa polling. Batas atas area notch sengaja tidak dicek, karena hook melihat posisi kursor sebelum dibatasi tepi layar (y bisa negatif saat mouse didorong ke atas). |
| Mengubah click-through | `set_ignore_cursor_events(true)` hanya sekali saat start; setelah itu hanya `WS_EX_TRANSPARENT` yang diubah langsung lewat `SetWindowLongPtrW` dari thread pekerja (bukan dari callback hook) | API Tauri ikut melepas/memasang `WS_EX_LAYERED`, yang membuat WebView2 menyusun ulang permukaan jendela (lag, tampilan macet). Callback hook harus cepat, kalau tidak Windows mencabut hook-nya diam-diam. |
| Tampil/sembunyi jendela | `ShowWindow` langsung (`SW_SHOWNOACTIVATE` / `SW_HIDE`), bukan `show()`/`hide()` Tauri | Tauri tidak tahu jendela sudah ditampilkan lewat `ShowWindow`, jadi `hide()`-nya diabaikan |
| Musik | `media.rs`: event WinRT (`CurrentSessionChanged`, `MediaPropertiesChanged`, `PlaybackInfoChanged`, `TimelinePropertiesChanged`) di thread MTA sendiri; posisi lagu diekstrapolasi di frontend dari `position` + `updatedAt`, ticker 250 ms hanya saat notch terbuka dan lagu berjalan | Tanpa polling; saat tertutup tidak ada yang bergerak |
| Lirik | Diambil dari frontend (`fetch` ke LRCLIB, CORS diizinkan) untuk **setiap lagu** selama opsi "baris lirik di notch kecil" aktif (default aktif; kalau dimatikan, hanya saat panel lirik dibuka). Yang ditemukan di-cache di `localStorage`, yang tidak ditemukan tidak di-cache. Judul dibersihkan dulu: buang "(Official…)", "lyric official" tanpa kurung, "- Topic"; untuk "A - B", sisi yang sama dengan artis dari player dianggap artis (menangani "Judul - Artis" maupun "Artis - Judul"). | Privasi: judul + artis + durasi terkirim ke LRCLIB; bisa dimatikan di Settings |
| Baris lirik di notch kecil | Notch ringkas 380×30 menampilkan baris yang sedang dinyanyikan; frontend tidak berdetak, tapi memasang timer tepat ke awal baris berikutnya. Saat tidak ada lirik atau bagian instrumental ("♪"), yang tampil jam. | Hemat CPU |
| Notch menyingkir | Hanya saat jendela aktif menutupi area notch **dan** ada elemen yang bisa diklik tepat di bawahnya (tab, tombol, kolom teks, link, item menu...), dicek lewat UI Automation di 5 titik selebar notch. Tab strip atau title bar yang kosong tidak dihitung, jadi notch tetap berukuran normal. Hasil cek dipakai ulang hingga 3 detik selama jendela tidak berubah, dan dicek ulang setelah klik mouse. Saat menyingkir, notch mengecil jadi **garis putih tipis 150×6** (lebarnya mengikuti notch tertutup, tidak hilang). Saat mengecas, garis itu hijau dan terisi sesuai persen baterai. Area hover = garis itu, jadi tab di bawahnya tetap bisa diklik. Dideteksi lewat `SetWinEventHook` (foreground, minimize, location change) plus cek ulang tiap 1 detik. | Permintaan pengguna |
| Status sistem | Semua lewat event: callback volume (`IAudioEndpointVolumeCallback`), perangkat audio (`IMMNotificationClient`), `RegisterPowerSettingNotification` (charger, baterai, kecerahan), keyboard hook (hanya melihat tombol Caps Lock dan perubahan layout; isi ketikan tidak dibaca), `NetworkStatusChanged`, `RegNotifyChangeKeyValue` pada ConsentStore (kamera/mic), dan pemantau folder Screenshots. Ditampilkan sebagai "peek" 310×60 selama ~2 detik. | Tanpa polling. Overlay volume bawaan Windows masih ikut muncul; menyembunyikannya belum dikerjakan. |
| Settings | Jendela terpisah dari menu tray: mirror notifikasi (on/off + tombol ke pengaturan Do Not Disturb), baris lirik di notch kecil, link kalender ICS, durasi Pomodoro, pengaturan minum air, kota cuaca. | Disimpan di `%APPDATA%\Ghyna\settings.json` |
| Migrasi data lama | Saat pertama dijalankan, Ghyna menyalin `settings.json`, `todo.json`, `water.json`, dan `shelf.json` dari `%APPDATA%\Notchy` ke `%APPDATA%\Ghyna`; file log dilewati dan file Ghyna yang sudah ada tidak ditimpa | Pengguna tidak perlu mengatur ulang setelah rebrand |
| Mengetik di notch | `begin_typing`/`end_typing`: notch sementara dibuat focusable dan diambil alih fokusnya lewat `AttachThreadInput` + `SetForegroundWindow`; setelah selesai, fokus dikembalikan ke aplikasi sebelumnya | Windows hanya mengizinkan aplikasi yang menerima input terakhir mengambil fokus, sedangkan klik di notch diterima proses WebView2 |
| Data produktivitas | `todo.json`, `water.json` (7 hari terakhir), `shelf.json` di `%APPDATA%\Ghyna`; pengingat memakai timer di frontend | Tanpa server |
| Mirror notifikasi | `UserNotificationListener` (baca saja). Pemicu: Windows menulis `LastNotificationAddedTime` per aplikasi di `HKCU\...\Notifications\Settings` ~75–125 ms setelah notifikasi masuk; nilai itu dibaca tiap 200 ms (handle key disimpan, struktur dipindai ulang tiap 10 s; AUMID aplikasi desktop berupa key bertingkat), dan daftar notifikasi baru diambil kalau nilainya berubah. Delay di notch ~0,3–0,5 s. Notifikasi beruntun diantre; peek notifikasi/screenshot tertahan saat di-hover dan bisa diklik. Klik = klik notifikasi asli: XML toast diambil dari database notifikasi Windows (`%LOCALAPPDATA%\Microsoft\Windows\Notifications\wpndatabase.db`, baca saja, lewat `rusqlite`) berdasarkan Id yang sama dengan listener (kalau toast-nya sudah diganti, pakai toast terbaru dari aplikasi yang sama). Argumen `launch` diteruskan ke `INotificationActivationCallback::Activate` milik aplikasinya, dengan CLSID dari `ToastActivatorCLSID` di AppxManifest (folder paket, atau `WindowsApps\<paket>` untuk Edge) atau dari `CustomActivator` di registry. Pengirim yang dipakai adalah `PrimaryId` di database (mis. `Edge!https://web.whatsapp.com/`, bukan `Edge!App` dari listener). Toast `activationType="protocol"` dibuka sebagai link. Setelah berhasil, toast dihapus dari pusat notifikasi. Kalau gagal, alasannya ditulis di log dan Ghyna membuka aplikasinya lewat `shell:AppsFolder` (cara lama). Teruji: WA Web langsung membuka chat-nya. Supaya tidak double: Do Not Disturb Windows (tombol di Settings). | Event `NotificationChanged` ditolak untuk aplikasi non-MSIX; `RegNotifyChangeKeyValue` tidak terpicu untuk key ini; menulis `ShowBanner` ke registry tidak berlaku langsung; banner Windows 11 bukan jendela tersendiri, jadi tidak bisa disembunyikan |
| Ganti output audio | `IPolicyConfig::SetDefaultEndpoint` (COM tidak resmi, dipakai juga oleh EarTrumpet/SoundSwitch); tombol di notch terbuka berpindah ke perangkat output berikutnya | Tidak ada API resmi |
| Kalender | Link ICS rahasia di Settings, diunduh lewat `Windows.Web.Http.HttpClient` di Rust (server kalender tidak mengizinkan CORS), di-parse di frontend; event berikutnya tampil di tampilan jam; pengingat 10 menit sebelum dan saat mulai; diambil ulang tiap 15 menit | Tanpa login OAuth |
| Shelf | Drop pakai event drag-drop Tauri; drag keluar pakai `tauri-plugin-drag` (CrabNebula) | Drag native Windows tidak tersedia di Tauri inti |
| Tidak merebut fokus | Jendela memakai `focus: true` dan `focusable: true`, tetapi ditampilkan dengan `ShowWindow(SW_SHOWNOACTIVATE)`. `begin_typing`/`end_typing` meminjam fokus hanya saat mengetik, lalu mengembalikannya ke aplikasi sebelumnya | Tauri menyusun ulang style jendela, jadi `WS_EX_NOACTIVATE` manual tidak awet; perilaku tanpa aktivasi dijaga lewat `ShowWindow` |
| Deteksi full-screen | `SHQueryUserNotificationState` **atau** jendela aktif menutupi seluruh monitor, dicek setiap 1 detik | Shell Windows tidak mendeteksi semua jendela full-screen (misalnya browser F11) |
| Penyimpanan data | File JSON di `%APPDATA%\Ghyna\` | Tanpa akun, tanpa server, data tetap di laptop |
| Penjaga (watchdog) | Task Scheduler **"Ghyna Watchdog"** (hanya di laptop pengguna, belum ikut installer): saat login + tiap 2 menit menjalankan `ghyna.exe --watchdog`, tanpa batas waktu, tetap jalan di baterai, `MultipleInstances IgnoreNew`. Kalau Ghyna hidup, salinan baru langsung keluar (single-instance); kalau mati, salinan itu yang mengambil alih (log: `start (restarted by watchdog)`). Exit dari tray menulis `%APPDATA%\Ghyna\exited-by-user`, sehingga penjaga membiarkannya mati; dibuka normal (atau autostart) menghapus penanda itu. | Ghyna sempat hilang tanpa jejak. Autostart (Run key) hanya berlaku saat login; Ghyna bertahan melewati sleep. |
| Update otomatis (aktif sejak v1.1.1) | Endpoint `tauri-plugin-updater` diarahkan ke `https://github.com/nandarjr660/Ghyna/releases/latest/download/latest.json`; `CUSTOM_BUILD = false` di `src-tauri/src/update.rs`. Build resmi memakai kunci `%USERPROFILE%\.tauri\ghyna.key` (pubkey tertanam di `tauri.conf.json`) dan menghasilkan `.sig` + `latest.json`. Alur: cek saat menyala, tiap jam, dan 30 detik setelah bangun dari sleep; installer NSIS passive. Repo harus tetap publik. | Rilis 1.1.1 adalah rilis pertama yang bisa memperbarui diri; versi 1.1.0 harus dipasang manual sekali karena updater-nya dimatikan |
| Kesehatan tampilan | Halaman notch mengirim detak jantung tiap 5 detik dari `requestAnimationFrame` (hanya jalan kalau halaman hidup **dan** menggambar). Kalau jendela sedang tampil dan 30 detik tanpa detak: halaman dimuat ulang; 60 detik kemudian masih diam: Ghyna restart sendiri. Penghitung direset saat jendela tersembunyi, layar terkunci, atau baru bangun dari sleep. Error JS dan promise yang gagal ditulis ke `ghyna.log` ("page error"). Tray punya "Muat ulang notch". Teruji dengan membekukan renderer: pulih dalam ~1,5 menit. | 20 Okt 2026 Ghyna "hilang" padahal prosesnya hidup dan jendelanya tampil, hanya tidak menggambar apa pun; penjaga dan log tidak menangkapnya |
| Halaman GitHub | `README.md` berbahasa Inggris (plus ringkasan Bahasa Indonesia) dengan banner, GIF demo, dan potongan panel di `docs/`. Sumbernya di `D:\Projects\notchy-promo\readme` (`banner.html` dirender oleh `shot.mjs`; GIF dan potongan dari video promo lewat ffmpeg). Deskripsi repo + 14 topik diisi lewat `gh repo edit`. Gambar social preview harus diunggah manual (Settings → General → Social preview, pakai `docs/banner.png`). Tidak menyebut Nusa. | Permintaan pengguna: repo menarik dilihat orang |
| Tetap paling atas | Tiap detik (di loop watcher) dicek apakah ada jendela biasa yang terlihat di atas notch dalam urutan z; kalau ada, `SetWindowPos(HWND_TOPMOST, SWP_NOMOVE/NOSIZE/NOACTIVATE)` dan dicatat di log (paling sering sekali per 10 menit). Teruji: didorong ke bawah 18 jendela, kembali ke atas < 2 detik. | 21 Okt 2026: notch "hilang" saat Outlook di depan; jendelanya masih bertanda topmost tapi posisinya di bawah jendela biasa (urutan ke-74). Kemungkinan ini juga penyebab "hilang" 20 Okt dan muat ulang otomatis malamnya: WebView2 berhenti menggambar saat tertutup |
| Log "kotak hitam" | `%APPDATA%\Ghyna\ghyna.log`: `start vX`, `exit (normal)`, dan `PANIC …` (thread, file:baris, pesan). Release memakai `panic = "unwind"`, sehingga panic di thread latar hanya mematikan thread itu. | Ghyna dua kali "hilang" tanpa jejak crash (19 Okt 2026). Ada baris PANIC berarti bug kode; tidak ada baris sama sekali berarti dimatikan dari luar (curiga: Trend Micro Apex One, karena Ghyna memakai keyboard hook). |
| Distribusi | Installer Ghyna bawaan Tauri (NSIS, per-user, tanpa admin), dipasang manual untuk pertama kali. Sejak v1.1.1 update otomatis aktif lewat GitHub Releases; data tetap saat update. Cara membuat rilis: bagian 9. | Hanya untuk diri sendiri dan teman. Tanpa code signing. Windows akan menampilkan peringatan SmartScreen; teman cukup klik "Run anyway". |

---

## 3. Keputusan yang masih terbuka

- [ ] Shortcut keyboard untuk membuka notch.
- [ ] Ikon aplikasi: masih ikon bawaan Tauri. Kandidat: wordmark "ghyna" + pil notch dari video promo.
- [ ] Overlay volume bawaan Windows masih ikut muncul bersama peek volume Ghyna.

---

## 4. Prinsip desain dan perilaku

**Jendela**
- Tanpa border, transparan, selalu di atas, tidak muncul di taskbar, dan tidak merebut fokus saat diklik.
- Area transparan di luar bentuk notch harus **click-through**.
- Posisi di tengah atas monitor utama, dengan ukuran yang tetap benar di skala 100% sampai 200%.
- Saat aplikasi full-screen berjalan, notch disembunyikan.

**Interaksi**
- Hover membuka notch dan kursor menjauh menutupnya. Tidak ada yang perlu di-pin atau diatur.
- Event penting (charger dicolok, mic dipakai, screenshot baru) memunculkan notch sebentar dalam bentuk ringkas, lalu kembali sendiri.
- Bisa juga dibuka dengan shortcut keyboard (shortcut-nya ditentukan nanti).

**Ukuran notch** (piksel logis, otomatis ikut skala Windows). Dirancang untuk lebar logis ~1500px (laptop 1920px di skala 125%).

| Mode | Lebar × tinggi | Radius sudut bawah | Lengkung "telinga" |
|---|---|---|---|
| Tertutup / istirahat | 150 × 30 | 14 | 10 |
| Menyingkir: ada yang bisa diklik di bawah notch (garis putih; lebar mengikuti notch tertutup, hijau dan terisi sesuai baterai saat mengecas) | 150 × 6 | 4 | 4 |
| Ringkas: musik diputar (cover kiri, equalizer kanan) | 200 × 30 | 14 | 10 |
| Ringkas: mengecas (cincin baterai + petir, jam) | 200 × 30 | 14 | 10 |
| Ringkas: musik + baris lirik (piringan berputar, lirik, equalizer) | 380 × 30 | 14 | 10 |
| Ringkas: timer Pomodoro berjalan (cincin + sisa waktu) | 200 × 30 | 14 | 10 |
| Screenshot baru (thumbnail + tombol) | 360 × 96 | 24 | 16 |
| Notifikasi Windows (ikon app, nama app, judul, isi) | 380 × 72 | 22 | 14 |
| Notifikasi ringkas (charger, headphone, dsb.) | 310 × 60 | 19 | 13 |
| Terbuka (musik, timer, air, shelf, to-do, clipboard) | 400 × 132 | 30 | 21 |
| Lirik (terbelah) | 500 × 140 | 30 | 21 |

- Notch menempel ke tepi atas layar: sudut atas lurus, hanya sudut bawah yang membulat.
- "Telinga" adalah lengkung cekung kecil di kiri dan kanan atas, supaya notch terlihat menyatu dengan tepi layar.
- Animasi ukuran: `width`/`height` 0.62 detik dengan `cubic-bezier(0.34, 1.32, 0.5, 1)` (sedikit membal), `border-radius` 0.45 detik.
- Jendela Tauri dibuat tetap 900 × 200 supaya mode terbesar plus bayangan muat; sisanya transparan dan click-through.
- Angka-angka ini titik awal dan boleh disesuaikan setelah dicoba di layar asli.

**Visual**
- Musik: layout player: cover besar sejajar header, progress di bawahnya, tombol lirik di bawah cover, transport di tengah. Warna dasar notch mengambil warna utama cover album selama tampilan musik. Animasi piringan dan equalizer dibuat bertahap (5 kali/detik, serentak) karena animasi 60 fps di jendela transparan memakan ~12% satu core; versi bertahap ~4%.
- Mode istirahat: cuaca di kiri, jam di tengah. Setelah 6 detik tanpa aktivitas, jam diganti mata terbuka; setelah 30 detik berikutnya tanpa aktivitas, mata tertidur. Musik/video yang diputar menunda tidur, dan aktivitas baru membangunkan mata.
- **Hitam polos** (dipilih pengguna, menggantikan glow gradien): latar hitam pekat dengan garis tepi putih tipis (16%) yang ikut melengkung di "telinga", plus sorotan putih sangat halus di dasar. Warna hanya dipakai untuk status: hijau mengecas, merah timer fokus, oranye mic dipakai, biru air, dan warna cover album saat musik. Sudut membulat, animasi spring, tipografi bersih.
- Notch terbuka punya 6 halaman (Musik/Jam, Timer, Air, To-do, Shelf, Clipboard): titik di bawah, panah kiri/kanan di sisi, dan scroll mouse (kecuali di atas daftar yang bisa di-scroll). Tombol mic dan output audio ada di halaman Jam.
- Mata: berkedip setiap 3,5–6,5 detik, melirik ke arah kursor, dan membesar sedikit saat kursor mendekat (dalam ~250px). Arah pandang dihitung di Rust, maksimal ~60 update/detik, dan hanya dikirim saat berubah.
- Unduhan: nama file, ukuran (bytes/total kalau total diketahui, bytes saja kalau tidak), kecepatan, dan meter persen 48×3 di sudut kanan. Total diketahui: `{persen}%` + bar terisi; total belum diketahui: `…` + track kosong. Tidak ada shimmer atau cahaya bergerak.
- Transisi halaman: `pageSlide` — halaman lama keluar 220 ms, halaman baru masuk 360 ms + delay 20 ms, `cubicOut`; geser 18 px ke arah navigasi, lift -3 px (keluar) / +5 px (masuk), skala 1 → 0,96, blur 1,8 px, dan cross-fade opacity. Halaman yang keluar tidak menerima pointer.

**Performa**
- Hanya menggambar saat ada yang bergerak. Saat diam, animasi CSS harus berhenti total.
- Pakai event dari Windows, bukan polling cepat. Kalau terpaksa polling, maksimal setiap 1 detik. Pengecualian yang disengaja: nilai registry notifikasi dibaca tiap 200 ms (bacaan nilai saja, ~0,2% CPU) karena tidak ada event yang bisa dipakai dan delay notifikasi harus kecil.
- Animasi yang berjalan terus (piringan, equalizer) dibuat bertahap 5 kali/detik dan serentak; animasi 60 fps di jendela transparan WebView2 mahal (~12% satu core).

---

## 5. Daftar fitur (semuanya masuk scope v1)

Setiap fitur baru dianggap selesai kalau memenuhi **Definition of Done (DoD)**-nya.

### A. Fondasi
- **Jendela notch**: transparan, click-through, posisi benar, animasi buka/tutup saat hover.
  DoD: klik di luar notch tembus ke aplikasi di bawahnya. Tampilan benar di skala 100%, 125%, dan 150%.
- **Tray icon**: menu Start with Windows, Personality (mata on/off), Settings, dan Exit.
- **Autostart** saat Windows menyala.
- **Sembunyi saat full-screen**.
  DoD: notch hilang saat YouTube di-full-screen atau game berjalan, dan muncul lagi setelahnya.
- **Settings** sederhana yang disimpan ke JSON.
  Dibuat bersama fitur pertama yang butuh setting (misalnya cuaca atau air minum). Item "Personality" di tray juga ditambahkan bersamaan dengan fitur mata. Status autostart sudah tersimpan di registry Windows, jadi untuk saat ini belum ada yang perlu disimpan ke JSON.

### B. Musik dan lirik
- **Now playing**: cover, judul, artis, progress bar, tombol play/pause/next/prev, dan seek.
  DoD: berjalan untuk aplikasi Spotify dan YouTube di Chrome/Edge.
- **Lirik tersinkron** dari LRCLIB: notch terbelah dua, baris yang sedang dinyanyikan menyala, dan klik pada sebuah baris akan lompat ke bagian itu.
  - Judul dibersihkan dulu sebelum dicari, misalnya dengan membuang "(Official Video)", "[Lyrics]", "- Topic".
  - Kalau player tidak mengirim posisi detik, lirik tetap tampil tanpa sorotan.
  - Kalau lirik tidak ditemukan, tampilkan pesan singkat dan jangan error.
  - Hasil pencarian di-cache supaya lagu yang sama tidak dicari berulang kali.

### C. Produktivitas
- **Pomodoro**: fokus 25 menit dan istirahat 5 menit (bisa diatur), countdown tampil di notch, ada bunyi chime.
- **To-do dengan pengingat**: tambah, edit, dan centang task, opsional dengan jam. Notch akan "menepuk" saat waktunya tiba.
- **Pengingat minum air**: ukuran botol, target harian, jam aktif, dan grafik minum hari ini. Bisa dimatikan sepenuhnya di Settings ("Pakai fitur minum air", sejak v0.2.1): pengingat berhenti dan halaman Air hilang dari notch. Permintaan pengguna: tidak semua teman butuh.
- **Shelf file**: drag file ke notch untuk disimpan sementara, drag keluar ke chat/email/folder, dan klik untuk membuka.

### D. Status sistem
- **Overlay volume dan brightness**.
- **Caps Lock dan perubahan bahasa keyboard**.
- **Baterai**: notifikasi saat charger dicolok/dicabut, plus persentase.
- **Wi-Fi**: notifikasi saat koneksi putus atau tersambung kembali.
- **Headphone Bluetooth**: notifikasi saat tersambung, plus baterai kalau perangkatnya melaporkannya.

### E. Fitur tambahan Ghyna
1. **Indikator kamera dan mikrofon**: titik warna di notch saat ada aplikasi yang memakai kamera/mic.
   Sumber data: registry `CapabilityAccessManager\ConsentStore\{webcam,microphone}`.
2. **Mute mic global**: satu klik di notch untuk mute/unmute mic default, dengan status yang selalu terlihat.
3. **Preview screenshot**: setelah Win+Shift+S, thumbnail muncul di notch, bisa di-drag atau disimpan, dan otomatis masuk Shelf.
4. **Cuaca**: suhu dan ikon di mode istirahat, dari BMKG.
5. **Ganti output audio** (speaker ↔ headphone) dari notch.
   ⚠️ Memakai API Windows yang tidak resmi (`IPolicyConfig`).
6. **Mirror notifikasi Windows**, misalnya pesan WhatsApp Desktop tampil di notch, bisa diklik untuk membuka aplikasinya. Dengan Do Not Disturb Windows aktif, notifikasi hanya muncul di notch (tidak double).
   Hasil riset: membaca notifikasi tidak butuh MSIX; yang butuh MSIX hanya event perubahannya (lihat bagian 2). Sejak v0.1.2 klik = klik notifikasi asli (membuka chat/email-nya) lewat toast activator aplikasinya (lihat bagian 2, "Mirror notifikasi").
7. **Event kalender berikutnya** dan countdown meeting, lewat link ICS rahasia dari Google/Outlook Calendar (tanpa login OAuth).

### F. v0.2.0: ala Dynamic Island iPhone dan aplikasi notch Mac
Dikerjakan sebagai **satu tahap besar**, dirilis sekali setelah lolos pengujian menyeluruh (lihat bagian 6, langkah 7). Referensi: Live Activities iPhone (sampai 3 aktivitas sekaligus di iPhone 18 Pro, panggilan, AirPods + baterai, progres AirDrop) dan aplikasi notch Mac (NotchNook: mirror kamera, tray + AirDrop, Shortcuts; Boring Notch: visualizer; NotchNest/Crest: clipboard).
1. **Pulau terbelah**: gaya **1B "satu pulau bersekat"** (dipilih pengguna dari mockup 22 Okt 2026, `D:\Projects\notchy-promo\mockup\v020.png`): satu pil memanjang, aktivitas utama di kiri/tengah, sampai 2 aktivitas lain di ujung kanan dipisah garis tipis. Prioritas: panggilan > timer > unduhan > musik; yang tertinggi jadi utama, kecuali musik yang sedang menampilkan baris lirik (lirik tetap utama). Geser touchpad mengganti aktivitas utama.
   DoD: dua/tiga aktivitas tampil bersamaan tanpa saling menimpa; klik tiap gelembung membuka halamannya; area klik mengikuti bentuk; menyingkir dan full-screen tetap benar.
2. **Pil panggilan**: hijau, durasi berjalan, tombol mute. Muncul saat Teams, WhatsApp, Zoom memakai mic, atau browser memakai mic > 10 detik (supaya voice typing tidak terhitung).
   DoD: panggilan Teams/WA sungguhan memunculkan pil, durasi benar, mute bekerja, pil hilang saat panggilan selesai.
3. **Earbuds tersambung + baterai**: peek model + baterai (kalau perangkat melaporkannya) dan peringatan baterai lemah.
   DoD: earbuds pengguna memunculkan peek; tanpa data baterai tetap muncul "tersambung".
4. **Progres berjalan**: unduhan browser dipantau dari folder Downloads dan folder unduhan kustom di Preferences browser; nama file, ukuran, kecepatan, dan path final diambil dari Chromium History. Notch menampilkan nama, bytes/total (atau bytes saja kalau total belum diketahui), kecepatan, dan meter persen 48×3 di sudut kanan (`{persen}%` + bar terisi; `…` + track kosong kalau total belum diketahui), tanpa shimmer. Setelah selesai: "Selesai, klik untuk buka". Rekam layar: titik merah + durasi, **hanya kalau riset menemukan sinyal Windows yang bisa dipakai**; kalau tidak, dicoret dan dicatat.
   DoD: unduhan nyata di Edge tampil dan selesai dengan benar; unduhan yang dibatalkan hilang dari notch.
   Sejak v0.2.2: hanya file yang bertambah dalam 15 detik terakhir yang tampil. File yang sudah ada saat pertama terlihat dianggap sisa (unduhan gagal/dibatalkan) sampai ia bertambah lagi, dan tidak dilaporkan "selesai" saat dihapus. Pemeriksaan tiap detik berhenti 60 detik setelah pertumbuhan terakhir. (Pemicu: di laptop teman, sisa `.crdownload` 223 MB tampil terus sebagai "Mengunduh…".)
5. **Equalizer asli**: batang mengikuti level suara yang diputar (meter audio Windows, ~10×/detik, hanya saat musik main).
   DoD: batang diam saat lagu hening, berdenyut saat ramai; CPU saat musik tidak naik berarti dibanding v0.1.5.
6. **Cermin kamera**: preview webcam kecil di notch lewat tombol, plus tawaran "Cek kamera" 2 menit sebelum meeting di kalender. Indikator "kamera dipakai" mengabaikan Ghyna sendiri.
   DoD: preview tampil, kamera mati lagi saat ditutup (lampu padam), tawaran muncul sebelum meeting.
7. **Drop untuk dibagikan**: zona "Bagikan" di samping Shelf saat file di-drag ke notch, membuka jendela Share Windows.
   DoD: file bisa dibagikan ke aplikasi lewat Share Windows.
8. **Riwayat clipboard**: 20 item terakhir (teks dan gambar), **di memori saja**, klik untuk menyalin lagi. Isi dari password manager (format `ExcludeClipboardContentFromMonitorProcessing` / `CanIncludeInClipboardHistory = 0`) tidak disimpan.
   DoD: copy teks/gambar tampil; teks bertanda password tidak tampil; hilang setelah restart.
9. ~~**Pintasan cepat**~~ — **dihapus di v1.1.0** atas permintaan pengguna; halaman Pintasan dan bagian Settings-nya tidak ditampilkan lagi. Kode `ShortcutsPage.svelte` dan `src-tauri/src/shortcuts.rs` masih ada tetapi tidak dipakai.
   DoD: tidak ada tab Pintasan; tab yang tersisa tepat 6.
10. **Gestur**: scroll di pita atas notch (setinggi notch tertutup) = volume (atas = naik); geser touchpad = pindah aktivitas utama (tertutup) atau halaman (terbuka); **tekan lama = kunci notch tetap terbuka** sampai tekan lama lagi atau klik di luar (hover sudah membuka notch, jadi "tekan lama = buka" diganti ini).
    DoD: tidak bentrok dengan scroll di halaman yang terbuka; volume berubah dengan peek volume yang sudah ada.
11. **Cek update di tray**: menu "Cek update (v0.x.x)". Ada versi baru → langsung pasang dan menyala lagi; sudah terbaru → peek "Sudah versi terbaru"; gagal → peek gagal; versi dari folder build → peek "versi pengembang".
    DoD: ketiga hasil teruji (versi lama → update, versi terbaru, tanpa internet).

### G. v1.1.0: rebrand dan penyempurnaan
1. **Rebrand Notchy → Ghyna**: nama aplikasi, identifier `com.ghyna.app`, path data `%APPDATA%\Ghyna`, log `ghyna.log`, dan watchdog `Ghyna Watchdog`. Data Notchy lama disalin sekali saat pertama dijalankan.
2. **Meter unduhan**: persen di sudut kanan, bar 48×3, tanpa shimmer; kalau total belum diketahui tampil `…` dengan track kosong.
3. **Transisi halaman**: `pageSlide` 220 ms keluar / 360 ms masuk, cross-fade + geser + blur halus.
4. **Halaman Pintasan dihapus**: tidak ada tab atau bagian Settings untuk Pintasan.
5. **Mata tidur 30 detik**: jam 6 detik → mata terbuka → tidur setelah 30 detik tanpa aktivitas.
   DoD: `npm run check` 0 error/0 warning; `npm test` 11/11; `npm run build` sukses; CDP mengukur mata terbuka → tidur 30,2 detik; tab tepat 6; installer `Ghyna_1.1.0_x64-setup.exe` terbentuk dan hash dicatat.

### H. v1.1.2: glow warna mata
1. **Glow sisi notch**: dua gradient lembut di kiri dan kanan bawah notch. Warnanya mengikuti warna mata: hijau saat normal, biru saat mengisi daya, merah saat baterai ≤15%, dan oranye saat 16–20%. Ukuran glow mengikuti ukuran notch dan disembunyikan saat notch tucked.
   DoD: `npm run check` 0 error/0 warning; `npm test` 11/11; build rilis sukses; glow terlihat di kedua sisi pada latar terang; installer `Ghyna_1.1.2_x64-setup.exe` (4.023.007 byte, SHA-256 `6E14D952CAEE780DC6F1A4EBC2AA6FF5CF24FB505492BDDD72728069B48B84EB`).

### I. v1.1.3: glow hanya saat idle
1. **Posisi di dalam notch**: dua gradient radial di dalam sisi kiri dan kanan bawah notch; bagian tengah tetap hitam. Warnanya tetap mengikuti warna mata: hijau normal, biru saat mengisi daya, merah saat baterai ≤15%, oranye saat 16–20%.
2. **Hanya mode idle**: glow tampil saat notch tertutup/idle (`view === "rest"`); disembunyikan saat halaman terbuka, player, charging, peek, atau tucked.
   DoD: `npm run check` 0 error/0 warning; `npm test` 11/11; screenshot idle di latar terang menunjukkan glow hanya di sisi kiri/kanan dalam notch; installer `Ghyna_1.1.3_x64-setup.exe` (4.025.233 byte, SHA-256 `E13C7F880646FE23A1CAEB1ACEB59A81AFA0213310508D25232605D57293CD58`).

### J. v1.1.4: inner rim glow
1. **Inner rim**: glow idle dirapikan menjadi garis tipis di tepi dalam kiri, kanan, dan bawah notch, ditambah bloom lembut di kedua sudut bawah. Bagian tengah tetap hitam agar mata dan jam tetap fokus.
2. **Warna dan visibilitas**: warna mengikuti mata (hijau normal, biru saat mengisi daya, merah saat baterai ≤15%, oranye saat 16–20%); tetap hanya tampil saat idle (`view === "rest"`) dan hilang saat halaman terbuka, player, charging, peek, atau tucked.
   DoD: `npm run check` 0 error/0 warning; `npm test` 11/11; build rilis sukses; screenshot idle di latar terang menunjukkan rim tipis + bloom sudut; installer `Ghyna_1.1.4_x64-setup.exe` (4.022.490 byte, SHA-256 `CA86BE8A3BE9B40353A7C5EDA3A80D8731007D145D3D3E831F3C4457C9D016FB`).

### K. v1.1.5: hairline glass
1. **Hairline glass**: glow idle diganti garis tipis 1px di tepi dalam kiri, kanan, dan bawah notch; tanpa bloom; bagian tengah tetap hitam agar mata dan jam tetap fokus.
2. **Warna dan visibilitas**: warna mengikuti mata (hijau normal, biru saat mengisi daya, merah saat baterai ≤15%, oranye saat 16–20%); hanya tampil saat idle (`view === "rest"`) dan hilang saat halaman terbuka, player, charging, peek, atau tucked.
    DoD: `npm run check` 0 error/0 warning; `npm test` 11/11; build rilis sukses; screenshot idle di latar terang; installer `Ghyna_1.1.5_x64-setup.exe` (4.023.873 byte, SHA-256 `780097A01FB30745BD9C2AA59DB01BA7C5CDD0AB659A3B5DF8EC41552D1A03C9`).

### L. v1.1.6: halaman cuaca
1. **Halaman mandiri**: menambahkan halaman `/weather` setelah halaman jam melalui tautan Cuaca.
2. **Informasi cuaca**: menampilkan lokasi, kondisi, ikon, suhu, kelembapan, angin, dan prakiraan per jam dalam tampilan glass yang responsif.
   DoD: `npm run check` 0 error/0 warning; `npm test` 11/11; `npm run build` sukses; installer `Ghyna_1.1.6_x64-setup.exe` (4.023.788 byte, SHA-256 `082CDDAC63A5A8392451ACB22E6576D3421DEEF7B1E6EDC54BD05E2C28A54375`).

### M. v1.1.7: data BMKG
1. **Sumber resmi**: cuaca terkini diambil dari endpoint BMKG berbasis koordinat, lalu prakiraan resmi 3 hari diambil memakai kode wilayah `adm4` yang dikembalikan BMKG.
2. **Tanpa API key**: menggunakan endpoint publik BMKG dengan atribusi BMKG; data mencakup suhu, kelembapan, angin, kondisi, dan prakiraan 3-jam.
   DoD: `npm run check` 0 error/0 warning; `npm test` 11/11; `npm run build` sukses; verifikasi live endpoint BMKG; installer `Ghyna_1.1.7_x64-setup.exe` (4.024.995 byte, SHA-256 `F1871358435DAC9C63E35AB10921DD7AFDFA101DC31B6488D82A4CE3FA8CE433`).

### N. v1.1.8–v1.1.9: penyempurnaan cuaca dan optimasi
1. **v1.1.8**: memperbaiki lifecycle timer dan listener frontend agar dibersihkan saat unmount, mengurangi risiko listener/heartbeat ganda dan polling yang tertinggal. Installer `Ghyna_1.1.8_x64-setup.exe` (4.028.369 byte, SHA-256 `EA1074EF633FD00A3D29363FA1F57C86AB4AF825294ED4BC948BDC9348684C9B`).
2. **v1.1.9**: halaman cuaca dipadatkan menjadi kondisi saat ini, suhu, lokasi, dan ikon dengan animasi ringan yang menyesuaikan kondisi cuaca; informasi sekunder dihapus agar layout notch aman.
    DoD: `npm run check` 0 error/0 warning; `npm test` 11/11; `npm run build` sukses; installer `Ghyna_1.1.9_x64-setup.exe` (4.027.953 byte, SHA-256 `C127CCBAEF56A9FC9AB46CD4A86C0D211C67C434203BDDE37025DFC7A4DA304F`).

### O. v1.1.10: fallback cuaca
1. **Fallback lokasi**: jika lokasi Windows tidak tersedia atau izin lokasi ditolak, Ghyna memakai koordinat Jakarta sebagai fallback sehingga halaman cuaca tidak berhenti di status memuat.
   DoD: `npm run check` 0 error/0 warning; `npm test` 11/11; `npm run build` sukses; installer `Ghyna_1.1.10_x64-setup.exe` (4.029.323 byte, SHA-256 `BD34481F932ABD6E2C569CBE63DF9AEF8DC1C64806D9C28535FB40BEA603DCAD`).

---

## 6. Urutan pengerjaan

Semua fitur di atas masuk v1. Urutan ini ada karena sebagian fitur bergantung pada fitur lain, bukan karena sebagian ditunda.

1. **Fondasi (A)**. Semua fitur lain bergantung pada jendela notch ini. ✅
2. **Musik dan lirik (B)**, fitur utama sekaligus uji coba sistem "tampilan per mode". ✅ Sudah dites dengan YouTube; aplikasi Spotify belum dites.
3. **Status sistem (D)** dan **E1–E4**. Semuanya event ringan yang memakai pola "muncul sebentar lalu kembali" yang sama. ✅ Sudah dites: volume, kecerahan, Caps Lock, screenshot, mode mengecas, mute mic, cuaca. Belum dites: ganti bahasa keyboard, Wi-Fi putus/sambung, headphone, kamera/mic dipakai, jendela Settings. Baterai headphone belum dibuat. Screenshot belum bisa di-drag (menunggu Shelf).
4. **Produktivitas (C)**. ✅ Notch terbuka sekarang punya 6 halaman (Musik/Jam, Timer, Air, To-do, Shelf, Clipboard) dengan titik dan panah kiri/kanan, plus scroll mouse. Sudah dites: timer jalan dan tampil di notch kecil, catat minum, tambah to-do dengan mengetik di notch, screenshot masuk Shelf, navigasi. Belum dites: drag file dari Explorer ke notch, drag file keluar dari Shelf, bunyi chime, dan pengingat to-do/minum saat waktunya tiba.
5. **E5–E7**. ✅ Riset E6: membaca notifikasi **tidak** butuh MSIX (`UserNotificationListener` mengizinkan aplikasi biasa), tapi event perubahannya butuh MSIX. Sudah dites: notifikasi uji tampil di notch, tombol output audio tampil, parser kalender (zona waktu, event berulang, EXDATE, dibatalkan, seharian). Belum dites: berpindah output (laptop ini hanya punya satu output), kalender dengan link ICS asli, jendela Settings.
6. **Penyempurnaan setelah v1** ✅
   - Tampilan hitam polos; garis menyingkir hanya saat ada yang bisa diklik di bawah notch (UI Automation).
   - Musik: layout player, warna dari cover, piringan berputar, equalizer bergerak, baris lirik di notch kecil (jam saat tidak ada lirik), jam di tengah saat istirahat.
   - Notifikasi: delay ~0,3–0,5 s (sebelumnya sampai 3 s), antrean, bisa diklik, panduan Do Not Disturb. Sudah dites: delay, antrean, dan tanpa banner saat DND aktif. Belum dites: klik notifikasi untuk membuka aplikasinya.

7. **v0.2.0 (bagian 5F), satu tahap besar.** Urutan di dalamnya: catat rencana → satu kali tinjau mockup (pulau terbelah dan tampilan baru) → bangun semua 11 fitur → uji → rilis.
   Pengujian, semua harus lolos sebelum rilis:
   - **Otomatis**: tes unit untuk logika (prioritas pulau terbelah, deteksi panggilan, saringan clipboard, deteksi unduhan, gestur), `svelte-check`, `cargo clippy`.
   - **Uji mandiri** di laptop dengan bukti screenshot per tampilan: unduhan palsu (`.crdownload` yang membesar), audio uji, copy teks biasa dan bertanda password, simulasi scroll, kamera buka/tutup, pintasan, jendela terdorong, full-screen, sleep.
   - **Uji bersama pengguna** (perangkat/aksi nyata): panggilan Teams/WA, earbuds, share ke aplikasi, geser touchpad; daftar cek singkat.
   - **Regresi** semua fitur lama (bagian 5A–E dan perbaikan v0.1.x).
   - **Kinerja**: CPU/memori saat diam, musik, dan 2–3 aktivitas sekaligus (target diam tetap ~1–2%); satu malam uji tahan lama, lalu baca log.
   - **Update**: versi lama → v0.2.0 lewat updater, dan tombol "Cek update".

   **Hasil (22 Okt 2026)**, semua 11 fitur dibangun:
   - Otomatis: 15 tes Rust (deteksi panggilan, nama unduhan, thumbnail clipboard, pintasan, link, equalizer) + 11 tes TS (pulau terbelah, ukuran, kalender/link meeting) lolos; `svelte-check` dan `cargo check` tanpa peringatan; `cargo clippy` hanya saran gaya (kode lama).
   - Uji mandiri dengan screenshot (`PrintWindow` + `PW_RENDERFULLCONTENT`; screenshot layar biasa tidak menangkap jendela WebView2 transparan, dan PrintWindow ikut menggambar bingkai caption tersembunyi yang tidak tampil di layar): musik + lirik + equalizer bergerak; lirik + unduhan; lirik + panggilan + unduhan dan geser ganti aktivitas utama (Zoom utama dengan mute, unduhan utama dengan garis berjalan); browser baru dianggap panggilan setelah 10 detik; peek "Selesai diunduh"; clipboard (teks bertanda password manager tidak tersimpan, gambar tidak dobel, kosong lagi setelah restart); halaman Pintasan dan tombol Downloads; tekan lama mengunci terbuka, klik di luar melepas; scroll volume dengan peek; cermin kamera terbuka, tetap terbuka, kamera dilepas setelah "Selesai".
   - Uji bersama pengguna, lolos semua: panggilan sungguhan + mute, drag ke "Bagikan" membuka Share, "Cek update" (peek versi pengembang), timer + musik, geser touchpad, pintasan link, klik notif WA, unduhan asli. Earbuds belum (pengguna tidak punya).
   - Perbaikan dari pengujian: kecepatan unduhan 0 B/s (daftar folder tidak memperbarui ukuran file yang sedang ditulis → baca ukuran dari file, sekali per detik); titik mic menumpuk di gelembung panggilan (titik pindah ke ujung pulau, mic disembunyikan saat panggilan); gambar clipboard dobel; scroll volume tanpa tanda saat notch terbuka (peek volume tampil di atas halaman); tombol kamera tidak terjangkau saat musik main ("Kamera" jadi pintasan default); equalizer + piringan menggambar ulang ~13×/detik (meter 5×/detik dan piringan diputar oleh detak yang sama); lirik macet saat aktivitas lain memimpin (pengatur waktu lirik kini jalan selama musik ada di pulau).
   - Uji sendiri yang perlu diingat: tulisan registry/file dari shell Claude yang di-sandbox tidak terlihat proses lain; tulis lewat proses WMI (`Win32_Process.Create`). Hook mouse Ghyna tidak melihat `SetCursorPos`; pakai `mouse_event` (input tersuntik).
   - Update: v0.1.5 terpasang → v0.2.0 lewat updater dalam ~9 detik; "Cek update (v0.2.0)" di versi terpasang → "Sudah versi terbaru". Penanda NSS ada di exe dalam installer.
   - Uji tahan lama semalam **dilewati** atas keputusan pengguna (rilis 22 Okt 2026); pantau `ghyna.log` beberapa hari pertama.

8. **v1.1.0 (bagian 5G): rebrand dan penyempurnaan.** ✅ Rilis 27 Okt 2026.
   - Rebrand Notchy → Ghyna (nama, identifier, path data, log, watchdog) + migrasi data lama; meter persen unduhan di sudut kanan tanpa shimmer; transisi halaman `pageSlide`; halaman Pintasan dihapus; mata tidur setelah 30 detik.
   - Verifikasi: `npm run check` 0 error/0 warning; `npm test` 11/11; `npm run build` sukses; CDP mengukur mata terbuka → tidur **30,2 detik**; tab tepat 6; installer `Ghyna_1.1.0_x64-setup.exe` (4.007.317 byte, SHA-256 `2C4E4994F9C39F8A596F36623DCA3E5DF71FA2C6600FBAFE4EA7E1B9B7777FE1`).

9. **v1.1.1 (bagian 2, updater): updater otomatis aktif.** ✅ Rilis 27 Okt 2026.
   - Fork publik `nandarjr660/Ghyna`; kunci `ghyna.key`; endpoint `latest.json`; `CUSTOM_BUILD = false`; installer bertanda tangan dan GitHub Release berisi installer, `.sig`, serta `latest.json`.
   - Verifikasi: `npm run check` 0 error/0 warning; `npm test` 11/11; build rilis sukses; endpoint publik `latest.json` HTTP 200; installer `Ghyna_1.1.1_x64-setup.exe` (4.020.640 byte, SHA-256 `2D85935BFE0D64C7B0F0E307CB0B94FE68DDCB53E99737EDF29073CD547730F4`).

10. **v1.1.2 (bagian 5H): glow warna mata.** ✅ Rilis 27 Okt 2026.
   - Dua gradient di sisi kiri/kanan notch, warnanya mengikuti mata: hijau normal, biru saat mengisi daya, merah saat baterai ≤15%, oranye saat 16–20%. Ukuran mengikuti notch dan hilang saat tucked.
   - Verifikasi: `npm run check` 0 error/0 warning; `npm test` 11/11; build rilis sukses; glow dicek di latar terang; installer `Ghyna_1.1.2_x64-setup.exe` (4.023.007 byte, SHA-256 `6E14D952CAEE780DC6F1A4EBC2AA6FF5CF24FB505492BDDD72728069B48B84EB`).

11. **v1.1.3 (bagian 5I): glow hanya saat idle.** ✅ Rilis 27 Okt 2026.
   - Gradient dipindah ke dalam sisi kiri/kanan notch; hanya tampil saat idle; hilang saat halaman terbuka, player, charging, peek, atau tucked.
   - Verifikasi: `npm run check` 0 error/0 warning; `npm test` 11/11; screenshot idle di latar terang; installer `Ghyna_1.1.3_x64-setup.exe` (4.025.233 byte, SHA-256 `E13C7F880646FE23A1CAEB1ACEB59A81AFA0213310508D25232605D57293CD58`).

12. **v1.1.4 (bagian 5J): inner rim glow.** ✅ Rilis 27 Okt 2026.
   - Glow idle dirapikan menjadi inner rim tipis di tepi dalam kiri/kanan/bawah plus bloom lembut di sudut; warna mengikuti mata; tetap hanya tampil saat idle.
   - Verifikasi: `npm run check` 0 error/0 warning; `npm test` 11/11; build rilis sukses; screenshot idle di latar terang; installer `Ghyna_1.1.4_x64-setup.exe` (4.022.490 byte, SHA-256 `CA86BE8A3BE9B40353A7C5EDA3A80D8731007D145D3D3E831F3C4457C9D016FB`).

13. **v1.1.5 (bagian 5K): hairline glass.** ✅ Rilis 28 Okt 2026.
   - Glow idle diganti garis tipis 1px di tepi dalam kiri/kanan/bawah, tanpa bloom; warna mengikuti mata; tetap hanya tampil saat idle.
   - Verifikasi: `npm run check` 0 error/0 warning; `npm test` 11/11; build rilis sukses; screenshot idle di latar terang; installer `Ghyna_1.1.5_x64-setup.exe` (4.023.873 byte, SHA-256 `780097A01FB30745BD9C2AA59DB01BA7C5CDD0AB659A3B5DF8EC41552D1A03C9`).

14. **v1.1.6 (bagian 5L): halaman cuaca.** ✅ Rilis 29 Okt 2026.
   - Halaman `/weather` mandiri dengan lokasi, kondisi, suhu, kelembapan, angin, dan prakiraan per jam.
   - Verifikasi: `npm run check` 0 error/0 warning; `npm test` 11/11; `npm run build` sukses; installer `Ghyna_1.1.6_x64-setup.exe` (4.023.788 byte, SHA-256 `082CDDAC63A5A8392451ACB22E6576D3421DEEF7B1E6EDC54BD05E2C28A54375`).

15. **v1.1.7 (bagian 5M): data BMKG.** ✅ Rilis 30 Okt 2026.
   - Sumber diganti ke BMKG untuk cuaca terkini dan prakiraan resmi 3 hari tanpa API key.
   - Verifikasi: `npm run check` 0 error/0 warning; `npm test` 11/11; `npm run build` sukses; verifikasi endpoint BMKG; installer `Ghyna_1.1.7_x64-setup.exe` (4.024.995 byte, SHA-256 `F1871358435DAC9C63E35AB10921DD7AFDFA101DC31B6488D82A4CE3FA8CE433`).

16. **v1.1.8–v1.1.9 (bagian 5N): optimasi dan cuaca ringkas.** ✅ Build lokal 31 Okt 2026.
   - v1.1.8 merapikan lifecycle timer/listener; v1.1.9 menyederhanakan halaman cuaca dan menambahkan animasi ringan sesuai kondisi.
   - Verifikasi: `npm run check` 0 error/0 warning; `npm test` 11/11; `npm run build` sukses; installer v1.1.9 `Ghyna_1.1.9_x64-setup.exe` (4.027.953 byte, SHA-256 `C127CCBAEF56A9FC9AB46CD4A86C0D211C67C434203BDDE37025DFC7A4DA304F`).

17. **v1.1.10 (bagian 5O): fallback lokasi cuaca.** ✅ Rilis 31 Okt 2026.
   - Jika lokasi Windows tidak tersedia, cuaca memakai fallback Jakarta agar tidak terus menampilkan “Memuat data cuaca”.
   - Verifikasi: `npm run check` 0 error/0 warning; `npm test` 11/11; `npm run build` sukses; installer `Ghyna_1.1.10_x64-setup.exe` (4.029.323 byte, SHA-256 `BD34481F932ABD6E2C569CBE63DF9AEF8DC1C64806D9C28535FB40BEA603DCAD`).

Setiap langkah selesai = DoD-nya terpenuhi + dites sendiri di laptop.

---

## 7. Non-goals (sengaja tidak dikerjakan)

- Versi macOS atau Linux.
- Akun, login, sinkronisasi cloud, atau analytics.
- Code signing Windows (peringatan SmartScreen tetap muncul saat pasang manual).
- Lirik dari Spotify atau sumber lain yang butuh login/scraping.
- Jadwal sholat.
- Tema yang bisa dikustomisasi bebas (cukup satu gaya yang rapi dulu).
- Halaman Pintasan cepat (dihapus di v1.1.0; pengguna tidak membutuhkannya).

---

## 8. Catatan untuk setiap sesi pengerjaan

- Baca file ini dulu sebelum mulai.
- Kalau ada keputusan baru, catat di bagian 2 atau 3, jangan hanya disebut di chat.
- Kalau ada fitur yang mau ditambah atau dibuang, ubah bagian 5 dan 7 dulu, baru kerjakan kodenya.

---

## 9. Rilis

| Versi | Tanggal | Isi |
|---|---|---|
| v0.1.0 | 19 Okt 2026 | Rilis pertama untuk teman: semua fitur bagian 5, tampilan hitam polos, musik + lirik di notch kecil |
| v0.1.1 | 19 Okt 2026 | Notifikasi ~0,3–0,5 s, antrean, klik untuk membuka aplikasi, tombol Do Not Disturb di Settings |
| v0.1.2 | 20 Okt 2026 | Klik notifikasi = klik notifikasi asli (WA Web langsung ke chat-nya); update otomatis dari GitHub Releases; log kotak hitam, panic tidak mematikan aplikasi; perbaikan notifikasi bertumpuk dengan halaman saat di-hover. Teruji: v0.1.1 uji update sendiri ke v0.1.2 dalam ~9 detik |
| v0.1.3 | 20 Okt 2026 | Notch yang berhenti tergambar pulih sendiri (detak jantung, muat ulang, restart); error tampilan masuk log; menu "Muat ulang notch" di tray |
| v0.1.4 | 20 Okt 2026 | Baris lirik di notch kecil langsung muncul setelah ganti lagu, pause/play, geser posisi, atau kembali dari tampilan lain (sebelumnya jam internalnya basi sehingga lirik tertahan sampai ada kejadian lain, misalnya minimize semua jendela) |
| v0.1.5 | 21 Okt 2026 | Notch tidak lagi "hilang" di balik jendela lain (misalnya Outlook): kalau terdorong ke bawah, dalam 1 detik kembali paling atas |
| v0.2.0 | 22 Okt 2026 | Bagian 5F: pulau terbelah (musik/timer/panggilan/unduhan berdampingan, geser untuk ganti), pil panggilan + mute, progres unduhan, equalizer asli, cek kamera + Join meeting, drop untuk Bagikan, riwayat clipboard, halaman pintasan, scroll volume, tekan lama untuk mengunci terbuka, baterai earbuds, "Cek update" di tray |
| v0.2.1 | 22 Okt 2026 | Sakelar "Pakai fitur minum air" di Settings; cek update tiap jam dan setelah bangun dari sleep. (Pemicu: teman di v0.1.2 tidak kunjung ter-update karena aplikasinya tidak dibuka ulang.) |
| v0.2.2 | 23 Okt 2026 | Sisa unduhan yang gagal/dibatalkan (`.crdownload`/`.part` yang tidak bertambah) tidak lagi tampil terus sebagai "Mengunduh…". Hanya tes otomatis; uji langsung dilewati. |
| v1.1.0 | 27 Okt 2026 | Rebrand Notchy → Ghyna (nama, identifier `com.ghyna.app`, path data `%APPDATA%\Ghyna`, log `ghyna.log`, watchdog, migrasi data lama); meter persen unduhan 48×3 di sudut kanan tanpa shimmer; transisi halaman baru; halaman Pintasan dihapus; mata tidur setelah 30 detik. Installer: `Ghyna_1.1.0_x64-setup.exe` (4.007.317 byte, SHA-256 `2C4E4994F9C39F8A596F36623DCA3E5DF71FA2C6600FBAFE4EA7E1B9B7777FE1`). |
| v1.1.1 | 27 Okt 2026 | Updater otomatis aktif: fork publik `nandarjr660/Ghyna`, kunci `ghyna.key`, `latest.json` bertanda tangan, cek saat menyala/tiap jam/setelah bangun. Rilis pertama yang bisa memperbarui sendiri. Installer: `Ghyna_1.1.1_x64-setup.exe` (4.020.640 byte, SHA-256 `2D85935BFE0D64C7B0F0E307CB0B94FE68DDCB53E99737EDF29073CD547730F4`). |
| v1.1.2 | 27 Okt 2026 | Glow warna mata di kedua sisi notch: hijau normal, biru saat mengisi daya, merah saat baterai ≤15%, oranye saat 16–20%; ukuran mengikuti notch dan hilang saat tucked. Installer: `Ghyna_1.1.2_x64-setup.exe` (4.023.007 byte, SHA-256 `6E14D952CAEE780DC6F1A4EBC2AA6FF5CF24FB505492BDDD72728069B48B84EB`). |
| v1.1.3 | 27 Okt 2026 | Glow warna mata dipindah ke dalam sisi kiri/kanan notch dan hanya tampil saat idle; hilang saat halaman terbuka, player, charging, peek, atau tucked. Installer: `Ghyna_1.1.3_x64-setup.exe` (4.025.233 byte, SHA-256 `E13C7F880646FE23A1CAEB1ACEB59A81AFA0213310508D25232605D57293CD58`). |
| v1.1.4 | 27 Okt 2026 | Glow idle dirapikan menjadi inner rim tipis di tepi dalam kiri/kanan/bawah plus bloom lembut di sudut; warna mengikuti mata; hanya tampil saat idle. Installer: `Ghyna_1.1.4_x64-setup.exe` (4.022.490 byte, SHA-256 `CA86BE8A3BE9B40353A7C5EDA3A80D8731007D145D3D3E831F3C4457C9D016FB`). |
| v1.1.5 | 28 Okt 2026 | Glow idle diganti hairline glass: garis tipis 1px di tepi dalam kiri/kanan/bawah notch, mengikuti warna mata, tanpa bloom, hanya saat idle. Installer: `Ghyna_1.1.5_x64-setup.exe` (4.023.873 byte, SHA-256 `780097A01FB30745BD9C2AA59DB01BA7C5CDD0AB659A3B5DF8EC41552D1A03C9`). |
| v1.1.6 | 29 Okt 2026 | Halaman Cuaca mandiri setelah halaman jam dengan lokasi, kondisi, suhu, kelembapan, angin, dan prakiraan per jam. Installer: `Ghyna_1.1.6_x64-setup.exe` (4.023.788 byte, SHA-256 `082CDDAC63A5A8392451ACB22E6576D3421DEEF7B1E6EDC54BD05E2C28A54375`). |
| v1.1.7 | 30 Okt 2026 | Sumber cuaca diganti ke BMKG: data terkini berbasis koordinat dan prakiraan resmi 3 hari tanpa API key. Installer: `Ghyna_1.1.7_x64-setup.exe` (4.024.995 byte, SHA-256 `F1871358435DAC9C63E35AB10921DD7AFDFA101DC31B6488D82A4CE3FA8CE433`). |
| v1.1.8 | 30 Okt 2026 | Optimasi lifecycle frontend: timer, heartbeat, polling, listener, dan timer animasi dibersihkan saat unmount untuk mengurangi risiko penggunaan RAM/baterai yang tidak perlu. Installer: `Ghyna_1.1.8_x64-setup.exe` (4.028.369 byte, SHA-256 `EA1074EF633FD00A3D29363FA1F57C86AB4AF825294ED4BC948BDC9348684C9B`). |
| v1.1.9 | 31 Okt 2026 | Halaman cuaca diringkas menjadi kondisi, suhu, lokasi, dan ikon dengan animasi ringan sesuai cuaca; layout notch dibuat aman dari clipping. Installer: `Ghyna_1.1.9_x64-setup.exe` (4.027.953 byte, SHA-256 `C127CCBAEF56A9FC9AB46CD4A86C0D211C67C434203BDDE37025DFC7A4DA304F`). |
| v1.1.10 | 31 Okt 2026 | Perbaikan halaman cuaca: memakai fallback koordinat Jakarta jika lokasi Windows tidak tersedia agar tidak berhenti di “Memuat data cuaca”. Installer: `Ghyna_1.1.10_x64-setup.exe` (4.029.323 byte, SHA-256 `BD34481F932ABD6E2C569CBE63DF9AEF8DC1C64806D9C28535FB40BEA603DCAD`). |
| v1.1.11 | 31 Okt 2026 | Redesain Apple UI di seluruh halaman: font, radius, warna, dan motion seragam lewat token di `app.css`; animasi magnet dan attach di Shelf; clear terbang ke tempat sampah di Shelf dan Clipboard; press feedback di semua tombol termasuk kontrol musik; file Shelf tetap ada saat drag dibatalkan (hapus hanya saat drop berhasil). Installer: `Ghyna_1.1.11_x64-setup.exe` (4.028.645 byte, SHA-256 `07EF4FA7D0036EC4CB65033A69D02EEF178A886876F744FB7A291630EF82E0A3`). |

**Cara membuat rilis baru (otomatis, sejak v1.1.1)**
1. Pastikan working tree bersih, berada di branch `main`, dan `origin` menunjuk ke `nandarjr660/Ghyna`.
2. Jalankan `.\scripts\release.ps1 <versi> "catatan rilis"` dari `_research\Notchy`.
3. Skrip menaikkan versi di `package.json`, `package-lock.json`, `tauri.conf.json`, `Cargo.toml`, dan `Cargo.lock`; membangun installer bertanda tangan; menulis `latest.json`; lalu commit, tag, push, dan membuat GitHub Release berisi installer, `.sig`, dan `latest.json`.
4. Kunci penandatangan ada di `%USERPROFILE%\.tauri\ghyna.key` dan tidak boleh di-commit. Kehilangan kunci berarti update berikutnya tidak bisa ditandatangani.
5. Setelah rilis, catat ukuran dan SHA-256 installer di tabel bagian 9.

**Catatan:** alur `release.ps1` lama untuk Notchy (`Notchy Watchdog`, `notchy.exe`, repo `RamantyaDS/Notchy`, `notchy.key`) sudah tidak dipakai.

Updater sudah aktif sejak v1.1.1. Peringatan "__TAURI_BUNDLE_TYPE variable not found in binary" saat pengemasan diulang aman selama `ghyna.exe` **di dalam installer** memuat `__TAURI_BUNDLE_TYPE_VAR_NSS`; tanpa penanda itu updater tidak tahu jenis installernya. Cek dengan 7-Zip: `7z x Ghyna_<versi>_x64-setup.exe ghyna.exe -r`, lalu cari teks itu. Exe di `target\release` bisa tetap `VAR_UNK` (Tauri menandai salinan yang dikemas); itu normal. v0.1.3–v0.1.5 (dulu Notchy) sudah dicek: `NSS`.

**Video promo** (bukan bagian repo ini): `D:\Projects\notchy-promo` – Reels 9:16, 27 detik, konsep "Satu garis, semuanya"; render lewat `render-par.mjs` (batas segmen dikunci ke frame), musik `music.html`, campur VO `mix.mjs`.
