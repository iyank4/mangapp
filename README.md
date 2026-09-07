# manapp

`manapp` adalah aplikasi terminal untuk melihat sumber instalasi aplikasi dan package manager yang umum digunakan di macOS. Aplikasi ini dibuat dengan Rust, Ratatui, dan Crossterm.

Tahap saat ini hanya menyediakan halaman **Sources**. Semua sumber yang didukung selalu ditampilkan dan diurutkan berdasarkan nama secara alfabetis. Sumber yang command-nya tidak tersedia di sistem tetap terlihat sebagai `DISABLED`, sehingga user dapat membedakan antara sumber yang tidak terpasang dan sumber yang tidak didukung.

Tabel Sources memiliki kolom `No.` untuk nomor urut visual. Sumber yang enabled ditampilkan lebih dulu, lalu sumber `DISABLED`; masing-masing kelompok tetap diurutkan secara alfabetis.

## Sumber yang didukung

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
- `DISABLED`: command sumber tidak ditemukan, tetapi sumber tetap didukung oleh `manapp`.
- `ERROR`: pemeriksaan command mengalami error.

## Menjalankan

Prasyarat:

- macOS
- Rust stable dan Cargo

Jalankan dari root project:

```bash
cargo run
```

Build binary release:

```bash
cargo build --release
./target/release/manapp
```

Kontrol keyboard:

- `↑`/`↓` atau `j`/`k`: navigasi
- `PageUp`/`PageDown` atau `Space`: berpindah halaman
- `Home`/`End`: ke awal/akhir
- `Enter`: tampilkan/sembunyikan detail
- `Tab`: berpindah fokus antara section Sources dan PATH
- `p`: tampilkan/sembunyikan daftar folder dari `PATH` di bawah Detail dan fokus ke PATH
- `↑`/`↓` atau `j`/`k`: scroll daftar folder `PATH` saat fokus berada di section PATH
- `r`: deteksi ulang sumber
- `q` atau `Esc`: keluar

Section `PATH directories` berada di bawah Detail, default hidden, dan mempertahankan urutan pembacaan environment variable `PATH`, termasuk duplikat. Daftarnya memiliki kolom `PATH`, `SOURCE HINT`, dan `NOTE / CATATAN`. Kolom `SOURCE HINT` diisi jika path dapat dicocokkan dengan `/etc/paths`, `/etc/paths.d/*`, atau path literal pada file shell user; selain itu dibiarkan kosong. Kolom `NOTE / CATATAN` berisi `Warning: path duplikat` atau `Error: folder tidak ada` jika relevan. `Tab` akan membukanya saat fokus berpindah dari Sources; section aktif diberi label `[FOCUS]`, dan row PATH aktif diberi reverse highlight. Tingginya dibatasi maksimal 50% tinggi terminal dan dapat di-scroll dengan `↑`/`↓` atau `j`/`k` saat fokus berada di section PATH; viewport mengikuti row aktif.

## How to develop

### Struktur kode

- `src/model.rs`: tipe data sumber dan status deteksi.
- `src/registry.rs`: katalog sumber yang didukung dalam urutan alfabetis.
- `src/detect.rs`: resolver command pada `PATH`.
- `src/ui.rs`: state aplikasi, layout tabel, warna, detail, dan navigasi.
- `src/main.rs`: lifecycle terminal dan event loop.
- `tests/`: test kontrak model, registry, detector, UI, dan binary.

### Perintah development

Format kode:

```bash
cargo fmt --all
```

Validasi format, test, dan lint:

```bash
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
```

Ukur coverage:

```bash
cargo install cargo-llvm-cov --locked
cargo llvm-cov --all-targets --summary-only --fail-under-lines 80
```

### Menambahkan sumber baru

1. Tambahkan `SourceDefinition` baru di `src/registry.rs`.
2. Letakkan entry pada posisi alfabetis berdasarkan `name`.
3. Isi `id`, `name`, minimal satu command kandidat, dan `category`.
4. Tambahkan atau perbarui test registry dan detector.
5. Jalankan format, test, Clippy, dan coverage.

Detector tidak menjalankan shell atau mengevaluasi command sebagai string. Ia hanya mencari executable kandidat pada `PATH`, sehingga setiap sumber dapat diuji secara deterministik menggunakan resolver palsu.

## Roadmap

Tahap berikutnya akan menambahkan daftar aplikasi terpasang dari setiap sumber dengan kolom terpadu. Data yang tidak tersedia dari sumber tertentu akan diberi flag/warna yang jelas, tanpa menghilangkan kolomnya dari tabel.
