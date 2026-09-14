# History: MangApp Sources

**Tanggal:** 2026-09-07
**Status:** Archived

Dokumen ini mengarsipkan hasil implementasi tahap pertama MangApp — Manage Applications, aplikasi terminal khusus MacBook dengan macOS.

## Hasil implementasi

- Scaffold Rust dengan package dan binary `mangap`.
- Registry source instalasi yang tersusun alfabetis.
- Deteksi source dengan status `AVAILABLE`, `DISABLED`, dan `ERROR`.
- Halaman Sources berbasis Ratatui dengan pengelompokan source enabled sebelum `DISABLED`.
- Nomor urut visual pada Sources dan PATH.
- Panel PATH yang default hidden, berada di bawah Detail, mempertahankan urutan resolve sistem, dan dapat di-scroll.
- Fokus section dan row PATH dengan navigasi `Tab`, arrow, dan `j`/`k`.
- Source hint untuk entry PATH yang dapat diidentifikasi.
- Catatan terpisah untuk duplicate warning dan missing-folder error.
- Penghapusan script `list_*` lama beserta test terkait.
- README publik, panduan development, dan preview aplikasi.

## Bukti validasi

- `cargo fmt --all -- --check` — lulus.
- `cargo test --all-targets` — 26 test lulus.
- `cargo clippy --all-targets --all-features -- -D warnings` — lulus.
- `cargo llvm-cov --all-targets --summary-only --fail-under-lines 80` — lulus, line coverage 89,35%.

## Commit utama

Riwayat implementasi tersimpan dalam commit kecil, termasuk scaffold, registry, detector, UI Sources, penghapusan script lama, rename MangApp, dokumentasi, dan screenshot README.

Backlog pengembangan berikutnya dikelola melalui GitHub Issues, bukan di dokumen history ini.
