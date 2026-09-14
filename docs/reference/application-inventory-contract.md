# Kontrak Application Inventory

**Issue:** [#1 — Define unified application table across sources](https://github.com/iyank4/mangapp/issues/1)

**Status:** `Implemented`

**Tanggal:** 2026-09-07

**Owner role:** FE/BE/QA; owner individu belum ditetapkan

## Tujuan

Menetapkan kontrak source-agnostic untuk tabel Application Inventory agar aplikasi yang dikumpulkan dari semua source dapat dibandingkan dengan kolom, arti data, status, dan perilaku interaksi yang konsisten.

Dokumen ini adalah acuan desain dan implementasi runtime. Collector awal, halaman inventory, dan contract test untuk perilaku utamanya sudah tersedia di repository.

## Scope

### In Scope

- Satu bentuk data normalisasi untuk satu baris aplikasi.
- Urutan dan makna kolom yang tetap untuk semua source.
- Representasi yang berbeda untuk data valid, kosong, unavailable, error, dan tidak relevan.
- Aturan layout untuk terminal lebar dan sempit.
- Kontrak sorting, filtering, focus, selection, dan scrolling.
- Acceptance test dan traceability untuk issue #1.

### Out of Scope

- Menyimpan data secara persisten.
- Menggabungkan atau menghapus duplikat lintas source secara otomatis.
- Mengubah perilaku halaman Sources yang sudah diimplementasikan, selain navigasi ke halaman Inventory.

## Istilah dan identitas baris

- **Application record** adalah hasil normalisasi satu aplikasi dari satu source.
- Satu baris mewakili satu record `source + identifier`; aplikasi dengan nama sama dari dua source tetap menjadi dua baris agar provenance dan perbandingan source tidak hilang.
- `No.` hanya nomor visual setelah filter dan sorting diterapkan. Nomor bukan identifier dan boleh berubah setelah refresh, filter, atau sorting.
- `source` berisi stable source ID dari registry. Nilai ini adalah identifier teknis, bukan nama tampilan, dan tidak boleh berubah ketika label source diubah.

## Kontrak data baris

Representasi konseptual berikut menjadi batas antara collector dan UI. Tipe aktual dapat menyesuaikan bahasa Rust saat implementasi, tetapi arti field dan state harus dipertahankan.

```text
ApplicationRecord {
  record_key:  StableRecordKey,
  source:      StableSourceId,
  name:        CellValue,
  version:     CellValue,
  identifier:  CellValue,
  status:      RecordStatus,
  location:    CellValue,
  installed_at: CellValue,
  updated_at:   CellValue,
  note:        CellValue,
}
```

| Field | Tampil sebagai | Aturan | Status |
| --- | --- | --- | --- |
| `record_key` | tidak ditampilkan; dipakai untuk selection/refresh | stabil dalam satu hasil collection; dibentuk dari `source` + identifier, atau fallback deterministik saat identifier unavailable | Implemented |
| `source` | `Sumber` | stable source ID dari registry, misalnya `homebrew`, `cargo`, atau `mas`; tidak boleh kosong | Implemented |
| `name` | `Aplikasi` | nama aplikasi yang dilaporkan source | Implemented |
| `version` | `Versi` | nomor versi dan build number yang dilaporkan source; UI tidak mengubah makna semver/non-semver | Implemented |
| `identifier` | `Identifier` | nama package/bundle/id source; boleh unavailable | Implemented |
| `status` | `Status` | status record atau hasil pengambilan data | Implemented |
| `location` | `Lokasi` | path instalasi bila source menyediakannya | Implemented |
| `installed_at` | panel detail, bukan kolom tabel | tanggal instalasi yang dilaporkan source | Implemented |
| `updated_at` | panel detail, bukan kolom tabel | tanggal terakhir aplikasi diperbarui menurut source | Implemented |
| `note` | panel detail, bukan kolom tabel | alasan status, warning, atau informasi tambahan | Implemented |

Panel detail record menampilkan `Tanggal Install`, `Tanggal Update`, dan `note` bersama informasi lengkap record. Ketiga field tersebut tidak menjadi kolom tabel dan tidak mengubah lebar atau alignment tabel utama.

## Nilai kolom Status

`Status` adalah status level-record dan hanya menggunakan empat nilai berikut:

| Status | Arti | Contoh kondisi |
| --- | --- | --- |
| `AVAILABLE` | record berhasil dikumpulkan dan mewakili aplikasi yang ditemukan | nama aplikasi berhasil dibaca; field opsional boleh tetap `EMPTY`, `UNAVAILABLE`, atau `N/A` |
| `EMPTY` | collector berhasil, tetapi field yang diharapkan dari record tidak berisi nilai | source mengembalikan record dengan versi atau identifier kosong |
| `UNAVAILABLE` | record atau data wajib tidak dapat disediakan oleh source/collector, tanpa terjadi kegagalan proses | source tidak menyediakan daftar aplikasi atau identifier wajib untuk record |
| `ERROR` | proses probe atau collection gagal | command source gagal dijalankan atau output tidak dapat diproses |

`VALUE`, `EMPTY`, `UNAVAILABLE`, `NOT_APPLICABLE`, dan `ERROR` adalah state pada level cell. `N/A` adalah token tampilan untuk state `NOT_APPLICABLE`, bukan nilai pada kolom `Status`. Field opsional yang tidak didukung source tidak otomatis mengubah status row menjadi `UNAVAILABLE`; cell tersebut menggunakan `N/A` atau `UNAVAILABLE` sesuai kondisinya. Jika collection berhasil tetapi tidak menghasilkan satu pun record, gunakan empty state tabel; jangan membuat application row palsu hanya untuk menampilkan `EMPTY`.

## Nilai kolom Identifier per source

`Identifier` menggunakan identifier native yang dilaporkan source. Nilai ini bukan nama tampilan aplikasi dan bukan path instalasi. Jika source menyediakan format canonical, gunakan format tersebut; jangan mengganti identifier dengan nama aplikasi hanya karena identifier native tidak tersedia.

| Source ID | Nilai `Identifier` | Contoh |
| --- | --- | --- |
| `cargo` | nama crate/package | `ripgrep` |
| `composer` | nama package Composer dalam format `vendor/package` | `laravel/framework` |
| `conda` | nama package Conda | `numpy` |
| `dart` | nama package Pub | `http` |
| `flutter` | nama package Pub atau package global Flutter | `melos` |
| `go` | module path Go, atau nama binary jika module path tidak tersedia | `golang.org/x/tools/gopls` |
| `homebrew` | token formula atau cask | `git`, `visual-studio-code` |
| `mas` | numeric App Store ID | `497799835` |
| `macports` | nama port | `wget` |
| `nix` | attribute/installable package Nix seperti yang dilaporkan profile | `nixpkgs#ripgrep` |
| `npm` | nama package npm, termasuk scope bila ada | `@angular/cli` |
| `pipx` | nama project/package PyPI | `black` |
| `pnpm` | nama package pnpm, termasuk scope bila ada | `typescript` |
| `python-pip` | nama distribution/project Python | `requests` |
| `rubygems` | nama gem | `rails` |
| `uv` | nama project/package Python | `ruff` |
| `yarn` | nama package Yarn, termasuk scope bila ada | `react` |

Jika identifier native tidak didukung atau gagal dibaca, cell tetap menggunakan `N/A` atau `UNAVAILABLE` sesuai state-nya. Jangan menggunakan hash store path, nomor urut tabel, atau nama tampilan sebagai identifier native.

## Kontrak kolom tabel

Urutan berikut adalah kontrak tetap. Semua source mengisi kolom yang sama; source tidak boleh menambah, menghapus, atau menggeser kolom berdasarkan field yang tersedia.

Susunan kanonis yang direkomendasikan:

```text
No. | Aplikasi | Versi | Sumber | Status | Identifier | Lokasi
```

Urutan ini memprioritaskan perbandingan yang paling sering dilakukan pengguna di sebelah kiri, lalu provenance dan status, kemudian metadata yang biasanya lebih panjang atau bersifat penjelas di sebelah kanan:

1. `No.` — nomor visual; bukan bagian dari identitas record.
2. `Aplikasi` — identitas utama yang dibaca dan dibandingkan pengguna.
3. `Versi` — diletakkan dekat nama agar perbandingan versi cepat dilakukan.
4. `Sumber` — provenance, yaitu package manager atau katalog asal record.
5. `Status` — keadaan record atau collection; diletakkan dekat `Sumber` agar kondisi data cepat dipindai dan token status tetap terlihat tanpa mengandalkan warna.
6. `Identifier` — nama package, bundle id, app id, atau identifier setara.
7. `Lokasi` — path instalasi yang dilaporkan source.

Kolom `Identifier` adalah kolom teknis bersama. Jangan membuat kolom terpisah seperti `Formula`, `Bundle ID`, `Package`, atau `App ID` untuk source tertentu; semua nilai tersebut dipetakan ke `Identifier`. Demikian pula, path instalasi selalu dipetakan ke `Lokasi`. Nomor versi dan build number tetap berada dalam satu kolom `Versi`, misalnya `1.2.3 (Build 456)`. Detail error, warning, dan konteks tambahan ditampilkan pada panel detail, bukan sebagai kolom tabel. Dengan begitu, kemampuan source yang berbeda hanya memengaruhi nilai cell, bukan susunan tabel.

Format tampilan `Versi`:

- versi dan build tersedia: `1.2.3 (Build 456)`;
- hanya versi tersedia: `1.2.3`;
- hanya build tersedia: `Build 456`;
- keduanya tidak tersedia: gunakan state cell yang sesuai, bukan string kosong tanpa penanda.

Format tampilan `Tanggal Install` dan `Tanggal Update` pada panel detail adalah `YYYY-MM-DD`. Jika source menyediakan timestamp, waktu dikonversi ke zona waktu lokal pengguna lalu bagian tanggalnya yang ditampilkan. Waktu lengkap tetap dapat ditampilkan pada panel detail bila diperlukan.

| Urutan | Header | Isi | Jika data tidak tersedia |
| ---: | --- | --- | --- |
| 1 | `No.` | nomor visual 1-based | tetap tampil; tidak berasal dari source |
| 2 | `Aplikasi` | nama aplikasi | `UNAVAILABLE` bila source tidak dapat menyediakan nama; record tanpa nama tidak boleh tampil sebagai data valid |
| 3 | `Versi` | nomor versi dan build number yang dilaporkan source | `EMPTY` bila source sukses tetapi tidak melaporkan versi/build; `N/A` bila field tidak berlaku; `UNAVAILABLE` bila collector/source tidak dapat mengambilnya |
| 4 | `Sumber` | stable source ID dari registry | selalu diisi dari registry; jika source gagal, status ditampilkan di `Status` dan detailnya tersedia pada panel detail |
| 5 | `Status` | `AVAILABLE`, `EMPTY`, `UNAVAILABLE`, atau `ERROR` | status tidak boleh disamakan dengan isi kosong |
| 6 | `Identifier` | package name, bundle id, atau identifier setara | `N/A` bila konsep tidak berlaku; `UNAVAILABLE` bila seharusnya ada tetapi gagal dibaca |
| 7 | `Lokasi` | path instalasi yang dilaporkan source | `N/A`, `UNAVAILABLE`, atau `ERROR` sesuai state data |

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
- Detail error, warning, atau konteks tambahan ditampilkan pada panel detail; jangan gunakan `—` untuk menyamarkan `EMPTY`, `UNAVAILABLE`, atau `ERROR` pada cell tabel.
- `Status` adalah status yang dapat difilter. State setiap cell tetap dipertahankan agar detail kegagalan tidak hilang.
- Warna tidak boleh menjadi satu-satunya penanda; token teks harus tetap membedakan state saat warna tidak tersedia.

## Layout terminal

### Terminal lebar

Pada terminal lebar (target minimum 120 kolom), semua tujuh kolom ditampilkan dalam urutan kontrak. Lebar kolom boleh memakai pembagian fixed/weighted, tetapi header dan cell pada row yang sama harus menggunakan definisi lebar yang sama.

Prioritas ruang minimum:

1. `No.` dan `Status` selalu terlihat penuh.
2. `Aplikasi`, `Sumber`, dan `Versi` mendapat ruang utama untuk perbandingan.
3. `Identifier` dan `Lokasi` boleh dipotong secara visual, tetapi tidak boleh berpindah ke kolom lain.

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
- Tie-breaker: `Sumber` mengikuti urutan registry berdasarkan stable source ID, lalu `Versi` descending sebagai teks natural jika dapat dibandingkan, lalu `Identifier` ascending.
- Sorting bersifat stable; record yang semua key-nya sama mempertahankan urutan hasil collector.
- `No.` dihitung ulang setelah sorting.
- Nilai `EMPTY`, `UNAVAILABLE`, `N/A`, dan `ERROR` diurutkan setelah nilai valid pada kolom yang sedang di-sort.

### Filtering

- Filter adalah pencarian teks case-insensitive terhadap `Aplikasi`, `Versi`, `Sumber`, `Status`, `Identifier`, dan `Lokasi`.
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
| Source berhasil tetapi field tertentu kosong | field bertoken `EMPTY`; panel detail menjelaskan bila perlu | row tetap dapat dipilih |
| Source/field tidak tersedia | field bertoken `UNAVAILABLE` atau `N/A` | row tetap tampil dan kolom tetap utuh |
| Collection/probe gagal | `Status = ERROR`, detail error di panel detail | row tetap tampil agar error dapat ditinjau |
| Tidak ada record setelah filter | header tetap tampil dan empty-state jelas | selection kosong; clear filter tersedia |

Error pada satu source tidak boleh menghilangkan record/source lain. Detail error tidak boleh ditampilkan sebagai `DISABLED`, karena `DISABLED` adalah status katalog Sources untuk command yang tidak ditemukan dan bukan status data inventory.

## Acceptance dan traceability

| Requirement issue #1 | Kontrak | Evidence saat ini | Status |
| --- | --- | --- | --- |
| Semua source memakai layout kolom yang sama | urutan tujuh kolom tetap | renderer `render_inventory` di `src/ui.rs` dan `inventory_contract.rs` | Implemented |
| Data unavailable tidak menggeser atau menghilangkan kolom | token/state mengambil lebar cell yang sama | `CellValue`, fixed constraints, dan renderer inventory | Implemented |
| Unavailable berbeda dari error dan data kosong | state cell eksplisit + token teks + warna | `src/model.rs`, `src/ui.rs`, serta collector error isolation | Implemented |
| Sorting, filtering, focus, scrolling konsisten | aturan interaksi di dokumen ini | navigasi, filter, refresh preservation, dan scroll inventory di `src/ui.rs` | Implemented |
| Alignment diuji pada terminal lebar dan sempit | contract test untuk dua ukuran viewport | `tests/inventory_contract.rs` menguji viewport 140 dan 80 kolom | Implemented |

## Test obligations

Contract test runtime yang tersedia sekarang:

- memverifikasi normalisasi record dan `record_key` berbasis stable source;
- memverifikasi fixture lintas source, filter, sorting, row numbering, selection, detail, dan refresh preservation;
- merender fixture pada terminal lebar dan sempit, termasuk horizontal scroll;
- memverifikasi bahwa satu source error tidak menghapus record dari source lain.

Test mengikuti pola renderer Ratatui yang sudah dipakai di `tests/ui_contract.rs`. Pada implementasi terakhir, seluruh suite berisi 32 test dan semuanya lulus.

## Dependensi, risiko, dan open questions

- Collector source tertentu masih dapat diperluas untuk memperkaya `Lokasi`, `Tanggal Install`, dan `Tanggal Update`; field tersebut sudah memiliki state runtime yang aman bila data belum disediakan.
- Perbandingan versi lintas ecosystem belum memiliki aturan universal; implementasi awal boleh memakai natural-text ordering dan harus mencatat batasannya.
- Kunci filter dan toggle sort harus diselaraskan dengan halaman Sources ketika fitur filter ditambahkan ke Sources; saat ini Sources belum memiliki filter.
- Horizontal scroll inventory memiliki state terpisah dari vertical scroll dan aktif saat lebar terminal lebih kecil dari viewport tabel.
- Keputusan deduplikasi lintas source masih terbuka; kontrak saat ini mempertahankan satu row per `source + identifier`.

## Evidence repository

- `src/model.rs` — model source dan application record, termasuk cell state serta status record.
- `src/inventory.rs` — collector runtime dan parser awal per source dengan error isolation.
- `src/registry.rs` — registry source dan source id stabil.
- `src/ui.rs` — layout Sources dan Inventory, fixed columns, detail, selection, filter, focus, dan horizontal/vertical scrolling.
- `tests/inventory_contract.rs` — contract test collector dan perilaku Inventory pada viewport lebar dan sempit.
- `tests/ui_contract.rs` — bukti test renderer dan interaksi UI yang sudah ada.
- `README.md` — status fitur publik dan scope halaman Sources.
