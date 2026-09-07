# manapp Sources Page Design

**Date:** 2026-09-07

## Goal

Membuat aplikasi terminal `manapp` berbasis Rust + Ratatui yang pada tahap pertama menampilkan katalog sumber instalasi yang didukung di macOS. Semua sumber yang didukung selalu terlihat; sumber yang executable-nya tidak tersedia di sistem ditampilkan sebagai `DISABLED` dengan gaya visual redup.

Tahap ini hanya mencakup halaman Sources. Pengambilan dan tampilan daftar aplikasi dari masing-masing sumber menjadi tahap berikutnya.

## User experience

Halaman Sources menampilkan tabel dengan kolom tetap:

| Status | Sumber | Command | Kategori | Keterangan |
|---|---|---|---|---|

Status yang digunakan:

- `AVAILABLE`: minimal satu executable kandidat ditemukan pada `PATH`.
- `DISABLED`: tidak ada executable kandidat yang ditemukan; baris tetap ditampilkan sebagai bukti dukungan `manapp`.
- `ERROR`: executable ditemukan tetapi pemeriksaan sumber gagal; berbeda dari `DISABLED` karena tool sebenarnya ada.

Baris `DISABLED` memakai warna abu-abu dan tidak menjadi pilihan aktif. Baris `ERROR` memakai warna peringatan. Panel detail menampilkan command kandidat, executable yang terdeteksi, dan alasan status.

Navigasi minimum:

- `↑`/`↓`: memilih baris.
- `PageUp`/`PageDown`: menggulir halaman.
- `Enter`: menampilkan detail sumber.
- `r`: menjalankan deteksi ulang.
- `q` atau `Esc`: keluar.

Katalog sumber awal:

| Sumber | Command kandidat | Kategori |
|---|---|---|
| Homebrew | `brew` | System package manager |
| Mac App Store | `mas` | App store |
| npm | `npm` | JavaScript package manager |
| Python pip | `pip3`, `pip`, `python3 -m pip` | Python package manager |
| uv | `uv` | Python tool manager |
| pipx | `pipx` | Python application manager |
| RubyGems | `gem` | Ruby package manager |
| Cargo | `cargo` | Rust package manager |
| Go | `go` | Go toolchain |
| Dart | `dart` | Dart/Flutter toolchain |
| Flutter | `flutter` | Dart/Flutter toolchain |
| MacPorts | `port` | System package manager |
| Nix | `nix` | System package manager |
| Conda | `conda`, `mamba` | Environment/package manager |
| pnpm | `pnpm` | JavaScript package manager |
| Yarn | `yarn` | JavaScript package manager |
| Composer | `composer` | PHP package manager |

Katalog bersifat compiled-in dan menjadi satu sumber kebenaran untuk nama, command, kategori, serta urutan tampilan. Menambahkan sumber baru harus cukup dengan menambahkan definisi katalog dan test deteksinya.

## Architecture

Komponen utama:

- `SourceDefinition`: metadata katalog sumber.
- `SourceStatus`: status hasil pemeriksaan (`Available`, `Disabled`, `Error`).
- `SourceSnapshot`: definisi sumber ditambah hasil deteksi dan pesan pengguna.
- `source_registry`: daftar semua `SourceDefinition` dalam urutan UI.
- `detector`: mencari command kandidat pada `PATH` tanpa menjalankan shell melalui string yang dievaluasi.
- `ui`: state halaman, layout tabel, warna, scrolling, input keyboard, dan panel detail.
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

Test unit akan memverifikasi:

- katalog selalu memuat seluruh sumber yang didukung dalam urutan yang stabil;
- setiap definisi memiliki nama, command kandidat, dan kategori non-kosong;
- command yang ditemukan menghasilkan `Available`;
- command yang tidak ditemukan menghasilkan `Disabled`;
- kegagalan probe menghasilkan `Error`, bukan `Disabled`;
- status dan pesan untuk `Available`, `Disabled`, dan `Error` berbeda.

Test UI/integrasi akan memverifikasi:

- semua sumber tampil pada halaman Sources;
- sumber yang tidak tersedia tetap terlihat dan memakai status disabled;
- keyboard `r` menjalankan refresh;
- keyboard `q` keluar dengan terminal bersih.

## Migration and scope

Setelah `manapp` memiliki implementasi Sources dan test yang lulus, file berikut dihapus:

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

1. `cargo test` lulus.
2. `cargo run` membuka aplikasi bernama `manapp` dan menampilkan halaman Sources.
3. Semua sumber dalam katalog terlihat, termasuk yang tidak terpasang.
4. Sumber yang tidak tersedia terlihat disabled dan tidak disalahartikan sebagai error.
5. Refresh dan exit tidak meninggalkan terminal dalam mode alternate/raw.
6. Seluruh script `list_*` dan test lamanya sudah dihapus setelah implementasi dinyatakan solid.
