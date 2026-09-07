# Panduan Development

Dokumen ini berisi panduan pengembangan MangApp — Manage Applications, aplikasi terminal khusus MacBook dengan macOS. Nama package dan binary di repository adalah `mangap`.

## Prasyarat

- MacBook dengan macOS.
- Rust stable dan Cargo.
- `cargo-llvm-cov` jika ingin memeriksa line coverage.

## Struktur kode

| Lokasi | Tanggung jawab |
| --- | --- |
| `src/model.rs` | Tipe data source dan status deteksi. |
| `src/registry.rs` | Katalog source yang didukung dalam urutan alfabetis. |
| `src/detect.rs` | Resolver command pada `PATH` dan inferensi source hint untuk entry PATH. |
| `src/ui.rs` | State aplikasi, layout tabel, warna, detail, dan navigasi. |
| `src/main.rs` | Lifecycle terminal dan event loop. |
| `tests/` | Test kontrak model, registry, detector, UI, dan binary. |

## Siklus kerja

Jalankan perintah berikut dari root project.

### Format

```bash
cargo fmt --all
```

### Validasi

```bash
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
```

### Coverage

```bash
cargo install cargo-llvm-cov --locked
cargo llvm-cov --all-targets --summary-only --fail-under-lines 80
```

## Menambahkan source

1. Tambahkan `SourceDefinition` baru di `src/registry.rs`.
2. Letakkan entry pada posisi alfabetis berdasarkan `name`.
3. Isi `id`, `name`, minimal satu command kandidat, dan `category`.
4. Tambahkan atau perbarui test registry dan detector.
5. Jalankan format, test, Clippy, dan coverage.

Semua source didaftarkan di registry agar tetap dapat ditampilkan sebagai `DISABLED` ketika command-nya tidak tersedia pada MacBook pengguna.

Detector tidak menjalankan shell atau mengevaluasi command sebagai string. Detector hanya mencari executable kandidat pada `PATH`, sehingga setiap source dapat diuji secara deterministik menggunakan resolver palsu.

## Dokumentasi desain dan progress

- [Design dan status fitur](../superpowers/specs/2026-09-07-mangap-sources-design.md)
- [Implementation plan dan progress](../superpowers/plans/2026-09-07-mangap-sources.md)

Status repository saat ini:

| Area | Status |
| --- | --- |
| Halaman Sources untuk source yang didukung | Implemented |
| Status `AVAILABLE`, `ERROR`, dan `DISABLED` | Implemented |
| Panel PATH, source hint, warning duplicate, dan error missing path | Implemented |
| Daftar aplikasi dari setiap source | Planned |
| Kolom aplikasi yang konsisten lintas source | Planned |
| Deteksi penggunaan storage per aplikasi | Planned |

## Prinsip perubahan

- Pertahankan dukungan macOS sebagai scope produk.
- Pertahankan urutan source alfabetis di registry; UI menampilkan source enabled lebih dahulu dan `DISABLED` setelahnya.
- Jangan menghapus kolom hanya karena data tidak tersedia dari suatu source; gunakan flag, warna, atau catatan yang jelas pada tahap daftar aplikasi.
- Pertahankan test yang menjelaskan kontrak perilaku dan jalankan validasi sebelum commit.
