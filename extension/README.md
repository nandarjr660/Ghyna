# Ghyna Download Bridge

Ekstensi kecil untuk Chrome/Edge yang mengirim progres unduhan ke notch Ghyna.
Dengan ekstensi ini persentase yang tampil adalah persentase asli dari browser,
termasuk saat unduhan dijeda.

## Pasang (sekali saja)

1. Jalankan Ghyna.
2. Buka `chrome://extensions` (atau `edge://extensions`).
3. Nyalakan **Developer mode**.
4. Klik **Load unpacked** dan pilih folder `extension` ini.
5. Selesai. Mulai unduhan apa saja dari browser; notch akan menampilkan persen
   yang sebenarnya.

Kalau kode ekstensi diperbarui, buka `chrome://extensions` lalu klik **Reload**
pada kartu "Ghyna Download Bridge". Chrome tidak memuat ulang otomatis untuk
ekstensi unpacked.

## Cara kerjanya

- Ekstensi membaca `chrome.downloads` (`bytesReceived`, `totalBytes`, `paused`)
  dan mengirim JSON ke `http://127.0.0.1:47823/downloads`.
- Selama ada unduhan berjalan, ekstensi mengirim tiap 1 detik (bukan hanya saat
  ada perubahan), jadi persen tetap muncul walau transfernya lambat.
- Ghyna hanya mendengarkan di `127.0.0.1`, jadi tidak bisa diakses dari jaringan.
- Tanpa ekstensi, Ghyna tetap mendeteksi unduhan dari file `.crdownload`, tetapi
  ukuran akhirnya belum diketahui sampai unduhan selesai.

Kalau Ghyna tidak berjalan, ekstensi diam saja dan mencoba lagi pada unduhan
berikutnya.

## Izin

- `downloads` — membaca daftar dan progres unduhan.
- `alarms` — heartbeat tiap 30 detik selama ada unduhan yang dijeda.
