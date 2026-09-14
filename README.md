# MangApp - Manage Applications

`mangap` adalah aplikasi terminal MangApp (Manage Applications) yang dibuat khusus untuk MacBook. Aplikasi ini membantu melihat sumber instalasi yang tersedia pada sistem. Aplikasi ini dibuat dengan Rust, Ratatui, dan Crossterm.

## Preview

![MangApp Sources](docs/images/mangapp-sources.png)

## Status Fitur

| Fitur | Status |
| --- | --- |
| Nama produk MangApp dan binary `mangap` | Implemented |
| Halaman Sources dengan sumber yang didukung | Implemented |
| Source `AVAILABLE`, `ERROR`, dan `DISABLED` | Implemented |
| Pengurutan source enabled sebelum `DISABLED` | Implemented |
| Panel PATH dengan urutan resolve sistem | Implemented |
| Source hint, duplicate warning, dan missing-path error | Implemented |
| Application Inventory lintas source | Implemented |

Semua source yang didukung selalu ditampilkan. Source yang command-nya tidak tersedia di sistem tetap terlihat sebagai `DISABLED`, sehingga user dapat membedakan antara source yang tidak terpasang dan source yang tidak didukung.

Kontrak layout dan data untuk Application Inventory tersedia di [Kontrak Application Inventory](docs/reference/application-inventory-contract.md). Dari Sources, tekan `Enter` untuk membuka Inventory sesuai source yang dipilih atau `i` untuk menampilkan detail source; gunakan `f` untuk filter, `u` untuk meng-upgrade semua aplikasi pada source aktif, dan `s` atau `Esc` untuk kembali ke Sources.

Tabel Sources memiliki kolom `No.` untuk nomor urut visual. Sumber yang enabled ditampilkan lebih dulu, lalu sumber `DISABLED`; masing-masing kelompok tetap diurutkan secara alfabetis.

Halaman Applications menampilkan status `UNAVAILABLE` untuk source yang `DISABLED`, sehingga source yang belum terpasang tetap dapat ditinjau tanpa menggagalkan inventory source lain. Saat pengambilan inventory berlangsung, halaman menampilkan state `LOADING`; jika tidak ada hasil, tabel menampilkan empty state yang jelas.

Kolom `Versi Baru` menampilkan versi terbaru yang tersedia melalui pengecekan native tiap source. Jika aplikasi sudah mutakhir atau source tidak menyediakan metadata update, nilainya ditampilkan sebagai `N/A`; jika pemeriksaan gagal, tampil `UNAVAILABLE`. Field yang sama juga tersedia pada panel detail aplikasi.

## Sumber yang didukung (alfabetis)

Daftar berikut diurutkan secara alfabetis berdasarkan nama source.

- Cargo
- Composer
- Conda / Mamba
- Dart
- Flutter
- Go
- Homebrew
- Mac App Store (`mas`)
- MacPorts
- Nix
- npm
- pipx
- pnpm
- Python pip
- RubyGems
- uv
- Yarn

Status yang ditampilkan:

- `AVAILABLE`: command sumber ditemukan pada `PATH`.
- `DISABLED`: command sumber tidak ditemukan, tetapi sumber tetap didukung oleh `MangApp`.
- `ERROR`: pemeriksaan command mengalami error.

## Development

Panduan struktur kode, workflow validasi, cara menambahkan source, dan status progress tersedia di [Panduan Development](docs/how-to/development.md).

Riwayat implementasi Sources tersedia di [History: MangApp Sources](docs/history/2026-09-07-mangapp-sources.md). Backlog aktif dikelola melalui [GitHub Issues](https://github.com/iyank4/mangapp/issues).
