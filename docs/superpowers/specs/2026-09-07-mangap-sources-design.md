# MangApp Sources Page Design

**Date:** 2026-09-07

## Status Implementasi

| Area | Status | Evidence / Catatan |
|---|---|---|
| Nama produk `MangApp` / binary `mangap` | Implemented | `Cargo.toml`, `src/main.rs`, `README.md` |
| Halaman Sources | Implemented | `src/ui.rs`, `tests/ui_contract.rs` |
| Source enabled sebelum `DISABLED` | Implemented | `src/ui.rs`, test ordering UI |
| Panel PATH dan urutan resolve sistem | Implemented | `src/detect.rs`, `src/ui.rs` |
| Source hint dan catatan error/warning PATH | Implemented | `src/detect.rs`, `src/ui.rs` |
| Daftar aplikasi dari setiap source | Planned | Belum ada collector atau halaman aplikasi |
| Kolom aplikasi konsisten lintas source | Planned | Menunggu desain kontrak data aplikasi |
| Deteksi storage per aplikasi | Planned | Menunggu daftar aplikasi dan strategi pengukuran |

## Goal

Membuat aplikasi terminal `mangap` (MangApp / Manage Applications) berbasis Rust + Ratatui yang dibuat khusus untuk MacBook dan pada tahap pertama menampilkan katalog sumber instalasi yang didukung. Semua sumber yang didukung selalu terlihat; sumber yang executable-nya tidak tersedia di sistem ditampilkan sebagai `DISABLED` dengan gaya visual redup.

Tahap ini hanya mencakup halaman Sources. Pengambilan dan tampilan daftar aplikasi dari masing-masing sumber menjadi tahap berikutnya.

## User experience

Halaman Sources menampilkan tabel dengan kolom tetap:

| No. | Status | Sumber | Command | Kategori | Keterangan |
|---|---|---|---|---|---|

Status yang digunakan:

- `AVAILABLE`: minimal satu executable kandidat ditemukan pada `PATH`.
- `DISABLED`: tidak ada executable kandidat yang ditemukan; baris tetap ditampilkan sebagai bukti dukungan `mangap`.
- `ERROR`: executable ditemukan tetapi pemeriksaan sumber gagal; berbeda dari `DISABLED` karena tool sebenarnya ada.

Baris `DISABLED` memakai warna abu-abu dan tidak menjadi pilihan aktif. Baris `ERROR` memakai warna peringatan. Panel detail menampilkan command kandidat, executable yang terdeteksi, dan alasan status.

Navigasi minimum:

- `↑`/`↓`: memilih baris.
- `PageUp`/`PageDown`: menggulir halaman.
- `Enter`: menampilkan detail sumber.
- `r`: menjalankan deteksi ulang.
- `Tab`: berpindah fokus antara Sources dan PATH.
- `p`: menampilkan atau menyembunyikan PATH.
- `q` atau `Esc`: keluar.

Panel PATH berada di bawah Detail dan default hidden. Panel ini memiliki kolom `No.`, `PATH`, `SOURCE HINT`, dan `NOTE / CATATAN`. Nomor dan urutan entry mengikuti hasil resolve sistem dari environment variable `PATH`, termasuk duplikat; entry tidak diurutkan alfabetis. `SOURCE HINT` diisi hanya jika dapat dicocokkan dengan file konfigurasi yang diketahui; jika tidak, kolom dibiarkan kosong. Note menampilkan warning untuk duplikat dan error untuk folder yang tidak ada. Panel dibatasi maksimal 50% tinggi terminal, dapat di-scroll dengan `↑`/`↓` atau `j`/`k`, dan row aktif diberi highlight.

Katalog sumber awal disimpan dalam registry berdasarkan nama sumber secara alfabetis. Pada UI, source enabled ditampilkan lebih dulu, kemudian `DISABLED`; masing-masing kelompok tetap alfabetis:

| Sumber | Command kandidat | Kategori |
|---|---|---|
| Cargo | `cargo` | Rust package manager |
| Composer | `composer` | PHP package manager |
| Conda | `conda`, `mamba` | Environment/package manager |
| Dart | `dart` | Dart/Flutter toolchain |
| Flutter | `flutter` | Dart/Flutter toolchain |
| Go | `go` | Go toolchain |
| Homebrew | `brew` | System package manager |
| Mac App Store | `mas` | App store |
| MacPorts | `port` | System package manager |
| Nix | `nix` | System package manager |
| npm | `npm` | JavaScript package manager |
| pipx | `pipx` | Python application manager |
| pnpm | `pnpm` | JavaScript package manager |
| Python pip | `pip3`, `pip`, `python3 -m pip` | Python package manager |
| RubyGems | `gem` | Ruby package manager |
| uv | `uv` | Python tool manager |
| Yarn | `yarn` | JavaScript package manager |

Katalog bersifat compiled-in dan menjadi satu sumber kebenaran untuk nama, command, kategori, serta urutan tampilan. Menambahkan sumber baru harus cukup dengan menambahkan definisi katalog dan test deteksinya.

## Architecture

Komponen utama:

- `SourceDefinition`: metadata katalog sumber.
- `SourceStatus`: status hasil pemeriksaan (`Available`, `Disabled`, `Error`).
- `SourceSnapshot`: definisi sumber ditambah hasil deteksi dan pesan pengguna.
- `source_registry`: daftar semua `SourceDefinition` dalam urutan UI.
- `detector`: mencari command kandidat pada `PATH` tanpa menjalankan shell melalui string yang dievaluasi.
- `path source hint`: mencocokkan entry PATH dengan `/etc/paths`, `/etc/paths.d/*`, dan path literal pada file shell user tanpa mengeksekusi file tersebut.
- `ui`: state halaman, layout tabel, warna, scrolling, input keyboard, panel detail, dan panel PATH.
- `main`: menjalankan pemeriksaan awal, membuka terminal alternate screen, dan memulai event loop.

Deteksi pertama dilakukan sebelum layar dibuka agar UI tidak menerima kondisi setengah terisi. Tombol `r` menjalankan deteksi ulang lalu mengganti snapshot secara atomik. Deteksi hanya memeriksa keberadaan executable pada tahap ini; collector daftar aplikasi tidak termasuk scope.

Dependensi runtime utama:

- `ratatui` untuk layout dan widget terminal.
- `crossterm` untuk event keyboard, alternate screen, dan terminal restoration.

Kode menggunakan Rust standard library untuk pencarian `PATH` dan error handling tahap pertama, sehingga tidak memerlukan dependency discovery tambahan.

## Error handling

- `PATH` kosong atau command tidak ditemukan menghasilkan `DISABLED`, bukan panic.
- Kegagalan terminal dikembalikan dari `main` dengan pesan ringkas dan exit code non-zero.
- Terminal harus dipulihkan melalui guard/cleanup walaupun event loop berhenti karena error.
- Kegagalan satu sumber tidak menggagalkan sumber lain.
- `DISABLED` tidak berarti sumber tidak didukung; itu berarti tool sumber tidak tersedia di sistem.

## Testing

Test yang tersedia memverifikasi:

- katalog selalu memuat seluruh sumber yang didukung dalam urutan yang stabil;
- setiap definisi memiliki nama, command kandidat, dan kategori non-kosong;
- command yang ditemukan menghasilkan `Available`;
- command yang tidak ditemukan menghasilkan `Disabled`;
- kegagalan probe menghasilkan `Error`, bukan `Disabled`;
- status dan pesan untuk `Available`, `Disabled`, dan `Error` berbeda.
- source enabled ditampilkan sebelum `DISABLED`;
- nomor urut Sources dan PATH ditampilkan;
- PATH mempertahankan urutan resolve, menampilkan source hint, serta memisahkan note error/warning;
- panel PATH dapat di-scroll dan row aktif mendapat highlight.

Test UI/integrasi akan memverifikasi:

- semua sumber tampil pada halaman Sources;
- sumber yang tidak tersedia tetap terlihat dan memakai status disabled;
- keyboard `r` menjalankan refresh;
- keyboard `q` keluar dengan terminal bersih.

Bukti validasi terakhir: `cargo test --all-targets` lulus dengan 26 test, Clippy lulus tanpa warning, format check lulus, dan line coverage mencapai 89,35%.

## Migration and scope

Setelah `mangap` memiliki implementasi Sources dan test yang lulus, file berikut dihapus:

- `list_homebrew.sh`
- `list_mas.sh`
- `list_npm.sh`
- `list_pip.sh`
- `list_uv.sh`
- `list_gem.sh`
- `list_cargo.sh`
- `test/test_list_homebrew_curses.zsh`
- `test/test_list_mas_ui.zsh`

Script yang tidak diawali `list_`—termasuk `cleanup.sh` dan `update.sh`—tetap dipertahankan. Penghapusan lama dan implementasi baru dilakukan dalam commit terpisah agar mudah direview.

## Acceptance criteria

1. `[Implemented]` `cargo test` lulus.
2. `[Implemented]` `cargo run` membuka aplikasi bernama `MangApp` (`mangap`) dan menampilkan halaman Sources di MacBook.
3. `[Implemented]` Semua sumber dalam katalog terlihat, termasuk yang tidak terpasang.
4. `[Implemented]` Sumber yang tidak tersedia terlihat disabled dan tidak disalahartikan sebagai error.
5. `[Implemented]` Refresh dan exit tidak meninggalkan terminal dalam mode alternate/raw.
6. `[Implemented]` Seluruh script `list_*` dan test lamanya sudah dihapus setelah implementasi dinyatakan solid.

## Gaps dan Rencana Berikutnya

| Tahap | Status | Rencana |
|---|---|---|
| Sources page | Implemented | Pertahankan sebagai baseline stabil sebelum menambah halaman baru. |
| Application inventory | Planned | Tambahkan adapter collector per source dan halaman daftar aplikasi. |
| Unified application table | Planned | Tetapkan kontrak kolom bersama; data yang tidak tersedia diberi flag/warna. |
| Storage inspection | Planned | Tambahkan pengukuran storage per aplikasi dengan status unavailable jika source tidak mendukung. |
