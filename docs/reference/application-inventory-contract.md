# Kontrak Application Inventory

**Issue:** [#1 — Define unified application table across sources](https://github.com/iyank4/mangapp/issues/1)

**Status:** `Documented Only`

**Tanggal:** 2026-09-07

**Owner role:** FE/BE/QA; owner individu belum ditetapkan

## Tujuan

Menetapkan kontrak source-agnostic untuk tabel Application Inventory agar aplikasi yang dikumpulkan dari semua source dapat dibandingkan dengan kolom, arti data, status, dan perilaku interaksi yang konsisten.

Dokumen ini adalah acuan desain dan implementasi berikutnya. Collector aplikasi, halaman inventory, dan perubahan kode belum termasuk hasil issue ini.

## Scope

### In Scope

- Satu bentuk data normalisasi untuk satu baris aplikasi.
- Urutan dan makna kolom yang tetap untuk semua source.
- Representasi yang berbeda untuk data valid, kosong, unavailable, error, dan tidak relevan.
- Aturan layout untuk terminal lebar dan sempit.
- Kontrak sorting, filtering, focus, selection, dan scrolling.
- Acceptance test dan traceability untuk issue #1.

### Out of Scope

- Menjalankan command collector atau membaca daftar aplikasi dari source.
- Menentukan parser/version resolver untuk source tertentu.
- Menyimpan data secara persisten.
- Menggabungkan atau menghapus duplikat lintas source secara otomatis.
- Mengubah perilaku halaman Sources yang sudah diimplementasikan.

## Istilah dan identitas baris

- **Application record** adalah hasil normalisasi satu aplikasi dari satu source.
- Satu baris mewakili satu record `source_id + identifier`; aplikasi dengan nama sama dari dua source tetap menjadi dua baris agar provenance dan perbandingan source tidak hilang.
- `No.` hanya nomor visual setelah filter dan sorting diterapkan. Nomor bukan identifier dan boleh berubah setelah refresh, filter, atau sorting.
- `source_id` mengikuti id stabil pada registry source. Nama tampilan boleh berubah tanpa mengubah identitas record.

## Kontrak data baris

Representasi konseptual berikut menjadi batas antara collector dan UI. Tipe aktual dapat menyesuaikan bahasa Rust saat implementasi, tetapi arti field dan state harus dipertahankan.

```text
ApplicationRecord {
  record_key:  StableRecordKey,
  source_id:   StableSourceId,
  source_name: DisplayText,
  name:        CellValue,
  version:     CellValue,
  identifier:  CellValue,
  status:      RecordStatus,
  location:    CellValue,
  note:        CellValue,
}
```

| Field | Tampil sebagai | Aturan | Status |
| --- | --- | --- | --- |
| `record_key` | tidak ditampilkan; dipakai untuk selection/refresh | stabil dalam satu hasil collection; dibentuk dari `source_id` + identifier, atau fallback deterministik saat identifier unavailable | Planned |
| `source_id` | tidak wajib menjadi kolom terpisah; dipakai untuk provenance | harus stabil dan cocok dengan registry | Planned |
| `source_name` | `Source` | nama tampilan dari registry; tidak boleh kosong | Planned |
| `name` | `Application` | nama aplikasi yang dilaporkan source | Planned |
| `version` | `Version` | versi mentah dari source; UI tidak mengubah makna semver/non-semver | Planned |
| `identifier` | `Identifier` | nama package/bundle/id source; boleh unavailable | Planned |
| `status` | `Status` | status record atau hasil pengambilan data | Planned |
| `location` | `Location` | lokasi install/cache bila source menyediakannya | Planned |
| `note` | `Keterangan` | alasan status, warning, atau informasi tambahan | Planned |

## Kontrak kolom tabel

Urutan berikut adalah kontrak tetap. Semua source mengisi kolom yang sama; source tidak boleh menambah, menghapus, atau menggeser kolom berdasarkan field yang tersedia.

| Urutan | Header | Isi | Jika data tidak tersedia |
| ---: | --- | --- | --- |
| 1 | `No.` | nomor visual 1-based | tetap tampil; tidak berasal dari source |
| 2 | `Application` | nama aplikasi | `UNAVAILABLE` bila source tidak dapat menyediakan nama; record tanpa nama tidak boleh tampil sebagai data valid |
| 3 | `Version` | versi yang dilaporkan source | `EMPTY` bila source sukses tetapi tidak melaporkan versi; `N/A` bila field tidak berlaku; `UNAVAILABLE` bila collector/source tidak dapat mengambilnya |
| 4 | `Source` | nama source | selalu diisi dari registry; jika source gagal, status/error dijelaskan di `Status` dan `Keterangan` |
| 5 | `Identifier` | package name, bundle id, atau identifier setara | `N/A` bila konsep tidak berlaku; `UNAVAILABLE` bila seharusnya ada tetapi gagal dibaca |
| 6 | `Status` | `AVAILABLE`, `EMPTY`, `UNAVAILABLE`, atau `ERROR` | status tidak boleh disamakan dengan isi kosong |
| 7 | `Location` | path atau lokasi yang dilaporkan source | `N/A`, `UNAVAILABLE`, atau `ERROR` sesuai state data |
| 8 | `Keterangan` | alasan, warning, atau konteks | tetap tampil; gunakan `—` jika tidak ada keterangan |

Header dan urutan kolom harus sama pada seluruh source dan seluruh refresh. Perbedaan kemampuan source hanya memengaruhi nilai/state cell, bukan struktur tabel.

## State cell dan visual

Setiap nilai yang tidak berupa data valid harus memiliki state eksplisit. UI menggunakan token teks sekaligus warna agar informasi tetap terbaca pada terminal tanpa warna.

| State | Arti | Token minimum | Gaya minimum | Beda dari |
| --- | --- | --- | --- | --- |
| `VALUE` | data berhasil tersedia | nilai asli | gaya normal | — |
| `EMPTY` | collector berhasil, tetapi source mengembalikan string kosong atau tidak mengisi nilai | `EMPTY` | kuning/dim | `UNAVAILABLE`: bukan kegagalan ketersediaan |
| `UNAVAILABLE` | field seharusnya dikenal, tetapi source/collector tidak dapat menyediakannya | `UNAVAILABLE` | abu-abu atau cyan redup | `ERROR`: bukan kegagalan proses |
| `NOT_APPLICABLE` | field memang tidak bermakna untuk source/record tersebut | `N/A` | abu-abu redup | `UNAVAILABLE`: bukan data yang hilang |
| `ERROR` | proses probe/collection gagal | `ERROR` | merah atau kuning kuat | `EMPTY`: proses tidak berhasil |

Aturan tambahan:

- Cell unavailable atau tidak relevan tetap mengambil lebar kolomnya.
- `—` tanpa label hanya boleh dipakai untuk `Keterangan` yang memang tidak memiliki catatan; jangan gunakan `—` untuk menyamarkan `EMPTY`, `UNAVAILABLE`, atau `ERROR`.
- `Status` adalah status yang dapat difilter. State setiap cell tetap dipertahankan agar detail kegagalan tidak hilang.
- Warna tidak boleh menjadi satu-satunya penanda; token teks harus tetap membedakan state saat warna tidak tersedia.

## Layout terminal

### Terminal lebar

Pada terminal lebar (target minimum 120 kolom), semua delapan kolom ditampilkan dalam urutan kontrak. Lebar kolom boleh memakai pembagian fixed/weighted, tetapi header dan cell pada row yang sama harus menggunakan definisi lebar yang sama.

Prioritas ruang minimum:

1. `No.` dan `Status` selalu terlihat penuh.
2. `Application`, `Source`, dan `Version` mendapat ruang utama untuk perbandingan.
3. `Identifier`, `Location`, dan `Keterangan` boleh dipotong secara visual, tetapi tidak boleh berpindah ke kolom lain.

### Terminal sempit

Pada terminal di bawah 120 kolom:

- urutan dan jumlah kolom tetap sama;
- kolom tidak boleh dihapus, di-reflow menjadi kartu, atau digabung berdasarkan source;
- isi cell boleh dipotong dengan ellipsis pada batas cell;
- header dan cell harus memakai lebar kolom yang sama setelah viewport dihitung;
- detail penuh tersedia melalui panel detail/row focus;
- jika total lebar kontrak melebihi viewport, tabel menggunakan horizontal scroll dengan offset yang terpisah dari vertical scroll.

Acceptance test harus memeriksa terminal lebar dan sempit untuk memastikan separator/header tetap sejajar dan token unavailable tidak masuk ke kolom tetangga.

## Perilaku interaksi

Application Inventory menggunakan model interaksi yang sama dengan Sources: selection row, panel detail, fokus section, refresh, dan scrolling berbasis viewport.

### Sorting

- Urutan default: `Application` ascending, case-insensitive.
- Tie-breaker: `Source` mengikuti urutan registry, lalu `Version` descending sebagai teks natural jika dapat dibandingkan, lalu `Identifier` ascending.
- Sorting bersifat stable; record yang semua key-nya sama mempertahankan urutan hasil collector.
- `No.` dihitung ulang setelah sorting.
- Nilai `EMPTY`, `UNAVAILABLE`, `N/A`, dan `ERROR` diurutkan setelah nilai valid pada kolom yang sedang di-sort.

### Filtering

- Filter adalah pencarian teks case-insensitive terhadap `Application`, `Version`, `Source`, `Identifier`, `Status`, dan `Keterangan`.
- Filter tidak mengubah data asli; hanya membatasi row yang ditampilkan.
- Row yang tidak cocok tidak dihitung dalam `No.` dan tidak dapat dipilih.
- Jika tidak ada hasil, tabel tetap menampilkan header dan pesan `Tidak ada aplikasi yang cocok dengan filter`.
- Membersihkan filter mengembalikan seluruh row dan mempertahankan key sorting terakhir.

### Focus dan selection

- Hanya row yang sedang tampil setelah filter yang dapat dipilih.
- `↑`/`↓`, `j`/`k`, `Home`, `End`, `PageUp`, dan `PageDown` memakai semantics Sources.
- `Enter` membuka atau menutup detail record terpilih.
- `Tab` berpindah antara section Application Inventory dan section pendukung lain tanpa mengubah selection.
- Refresh mempertahankan filter/sort bila record terpilih masih ada; jika tidak, selection kembali ke row pertama yang tersedia.

### Scrolling

- Vertical scroll mengikuti row aktif dan tidak boleh menampilkan row setengah di luar viewport.
- Horizontal scroll hanya digunakan pada terminal sempit dan tidak mengubah selection.
- Jika tidak ada row, selection kosong dan tombol navigasi tidak boleh panic atau mengubah state secara tidak terduga.

## Error, empty, dan unavailable

| Kondisi | Tampilan row | Dampak interaksi |
| --- | --- | --- |
| Source tersedia dan record valid | data normal, `Status = AVAILABLE` | row dapat dipilih |
| Source berhasil tetapi field tertentu kosong | field bertoken `EMPTY`; `Keterangan` menjelaskan bila perlu | row tetap dapat dipilih |
| Source/field tidak tersedia | field bertoken `UNAVAILABLE` atau `N/A` | row tetap tampil dan kolom tetap utuh |
| Collection/probe gagal | `Status = ERROR`, detail error di `Keterangan` | row tetap tampil agar error dapat ditinjau |
| Tidak ada record setelah filter | header tetap tampil dan empty-state jelas | selection kosong; clear filter tersedia |

Error pada satu source tidak boleh menghilangkan record/source lain. Detail error tidak boleh ditampilkan sebagai `DISABLED`, karena `DISABLED` adalah status katalog Sources untuk command yang tidak ditemukan dan bukan status data inventory.

## Acceptance dan traceability

| Requirement issue #1 | Kontrak | Evidence saat ini | Status |
| --- | --- | --- | --- |
| Semua source memakai layout kolom yang sama | urutan delapan kolom tetap | `src/ui.rs` saat ini baru memiliki tabel Sources; inventory belum ada | Planned / Gap |
| Data unavailable tidak menggeser atau menghilangkan kolom | token/state mengambil lebar cell yang sama | belum ada renderer inventory | Planned / Gap |
| Unavailable berbeda dari error dan data kosong | state cell eksplisit + token teks + warna | `SourceStatus` membedakan status Sources di `src/model.rs`; state inventory belum ada | Partially Implemented |
| Sorting, filtering, focus, scrolling konsisten | aturan interaksi di dokumen ini | navigasi/focus/scroll Sources ada di `src/ui.rs`; filter inventory belum ada | Partially Implemented |
| Alignment diuji pada terminal lebar dan sempit | contract test untuk dua ukuran viewport | test UI Sources tersedia di `tests/ui_contract.rs`; test inventory belum ada | Planned / Gap |

## Test obligations

Implementasi berikutnya wajib menambahkan contract test yang:

- merender minimal dua source dengan field yang berbeda-beda dan memverifikasi header/column order identik;
- merender `EMPTY`, `UNAVAILABLE`, `N/A`, dan `ERROR`, lalu memverifikasi token dan style state masing-masing;
- merender fixture yang sama pada terminal lebar dan sempit dan memverifikasi separator/header/cell tetap berada pada batas kolom yang sama;
- memverifikasi sorting default, tie-breaker, dan posisi nilai unavailable/error;
- memverifikasi filter, empty-state, row numbering, selection, detail, refresh, dan vertical/horizontal scroll;
- memverifikasi bahwa satu source error tidak menghapus row dari source lain.

Test yang direncanakan mengikuti pola test renderer Ratatui yang sudah dipakai di `tests/ui_contract.rs`. Karena issue ini hanya menghasilkan dokumentasi, belum ada hasil test baru yang dapat diklaim.

## Dependensi, risiko, dan open questions

- Collector setiap source perlu menyepakati pemetaan nama, versi, identifier, dan location sebelum kontrak dapat diberi status `Implemented`.
- Perbandingan versi lintas ecosystem belum memiliki aturan universal; implementasi awal boleh memakai natural-text ordering dan harus mencatat batasannya.
- Kunci filter dan toggle sort harus diselaraskan dengan halaman Sources ketika fitur filter ditambahkan ke Sources; saat ini Sources belum memiliki filter.
- Horizontal scroll menambah state UI yang belum dimiliki `App`; detail implementasinya harus tetap terpisah dari vertical scroll.
- Keputusan deduplikasi lintas source masih terbuka; kontrak saat ini mempertahankan satu row per `source_id + identifier`.

## Evidence repository

- `src/model.rs` — status source yang sudah ada (`AVAILABLE`, `DISABLED`, `ERROR`).
- `src/registry.rs` — registry source dan source id stabil.
- `src/ui.rs` — pola layout tabel, detail, selection, focus, dan scrolling halaman Sources.
- `tests/ui_contract.rs` — bukti test renderer dan interaksi UI yang sudah ada.
- `README.md` — status fitur publik dan scope halaman Sources.
