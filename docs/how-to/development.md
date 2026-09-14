# Panduan Development

Dokumen ini berisi panduan pengembangan MangApp — Manage Applications, aplikasi terminal khusus MacBook dengan macOS. Nama package dan binary di repository adalah `mangap`.

## Prasyarat

- MacBook dengan macOS.
- Rust stable dan Cargo.

`cargo-llvm-cov` diperlukan jika ingin memeriksa line coverage.

## Menjalankan aplikasi

Jalankan dari root project:

```bash
cargo run
```

Build binary release:

```bash
cargo build --release
./target/release/mangap
```

## Kontrol aplikasi

- `↑`/`↓` atau `j`/`k`: navigasi.
- `PageUp`/`PageDown` atau `Space`: berpindah halaman.
- `Home`/`End`: ke awal/akhir.
- Pada Sources, `Enter`: buka halaman Application Inventory sesuai source yang dipilih; `i`: tampilkan/sembunyikan detail source.
- Pada Inventory, `Enter`: tampilkan/sembunyikan detail aplikasi; `s` atau `Esc`: kembali ke Sources; `←`/`→` atau `h`/`l`: horizontal scroll.
- `f`: aktifkan filter pada Inventory.
- `Tab`: berpindah fokus antara section Sources dan PATH.
- `p`: tampilkan/sembunyikan daftar folder dari `PATH` di bawah Detail dan fokus ke PATH.
- `↑`/`↓` atau `j`/`k`: scroll daftar folder `PATH` saat fokus berada di section PATH.
- `r`: deteksi ulang sumber dan mengambil ulang inventory aplikasi.
- `q` atau `Esc`: keluar.

Section `PATH directories` berada di bawah Detail, default hidden, dan mempertahankan urutan resolve yang diberikan sistem melalui environment variable `PATH`, termasuk duplikat; path tidak diurutkan alfabetis. Daftarnya memiliki kolom `No.`, `PATH`, `SOURCE HINT`, dan `NOTE / CATATAN`. Kolom `SOURCE HINT` diisi jika path dapat dicocokkan dengan `/etc/paths`, `/etc/paths.d/*`, atau path literal pada file shell user; selain itu dibiarkan kosong. Kolom `NOTE / CATATAN` berisi `Warning: path duplikat` atau `Error: folder tidak ada` jika relevan. `Tab` akan membukanya saat fokus berpindah dari Sources; section aktif diberi label `[FOCUS]`, dan row PATH aktif diberi reverse highlight. Tingginya dibatasi maksimal 50% tinggi terminal dan dapat di-scroll dengan `↑`/`↓` atau `j`/`k` saat fokus berada di section PATH; viewport mengikuti row aktif.

## Struktur kode

| Lokasi | Tanggung jawab |
| --- | --- |
| `src/model.rs` | Tipe data source dan status deteksi. |
| `src/registry.rs` | Katalog source yang didukung dalam urutan alfabetis. |
| `src/detect.rs` | Resolver command pada `PATH` dan inferensi source hint untuk entry PATH. |
| `src/inventory.rs` | Collector dan parser runtime untuk daftar aplikasi per source. |
| `src/ui.rs` | State aplikasi, layout tabel, warna, detail, filter, scroll, dan navigasi. |
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

## History dan backlog

- [Dokumentasi MangApp](../README.md)
- [Kontrak Application Inventory](../reference/application-inventory-contract.md)
- [History: MangApp Sources](../history/2026-09-07-mangapp-sources.md)

Pengambilan inventory berjalan di background setelah aplikasi dibuka dan saat refresh. Halaman Applications menampilkan `LOADING` selama proses, `EMPTY` bila source sukses tetapi tidak mengembalikan record, `UNAVAILABLE` untuk source yang `DISABLED` atau field yang tidak disediakan, dan `ERROR` bila collector gagal.

Status repository saat ini:

| Area | Status |
| --- | --- |
| Halaman Sources untuk source yang didukung | Implemented |
| Status `AVAILABLE`, `ERROR`, dan `DISABLED` | Implemented |
| Panel PATH, source hint, warning duplicate, dan error missing path | Implemented |
| Application Inventory lintas source | Implemented |

Backlog aktif dikelola melalui [GitHub Issues](https://github.com/iyank4/mangapp/issues). Kontrak Application Inventory di atas menjadi acuan runtime untuk issue #1.

## Prinsip perubahan

- Pertahankan dukungan macOS sebagai scope produk.
- Pertahankan urutan source alfabetis di registry; UI menampilkan source enabled lebih dahulu dan `DISABLED` setelahnya.
- Pertahankan test yang menjelaskan kontrak perilaku dan jalankan validasi sebelum commit.
