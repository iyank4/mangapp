# ManGApp Sources Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Current status:** Tasks 1–7 below are `Implemented`. The checkboxes preserve the original execution breakdown; the status tables in this document are the current source of truth.

**Goal:** Membuat aplikasi Rust + Ratatui bernama `mangap` (ManGApp / Manage Grouped Applications) yang menampilkan sumber instalasi, aplikasi yang dikelola oleh setiap source, dan penggunaan storage per aplikasi melalui UI terminal yang konsisten.

**Architecture:** `mangap` memakai katalog sumber compiled-in, resolver command yang dapat diuji, model snapshot status, dan UI Ratatui yang dipisahkan dari lifecycle terminal. Binary hanya mengatur probing awal, refresh, alternate screen, raw mode, dan event loop; registry, detector, model, dan rendering berada di library agar dapat diuji tanpa terminal nyata.

**Tech Stack:** Rust 2024, Ratatui 0.30.2, Crossterm 0.29, Cargo test, Ratatui `TestBackend`, dan `cargo-llvm-cov` untuk pengukuran coverage.

**Spec:** `docs/superpowers/specs/2026-09-07-mangap-sources-design.md`

## Status Saat Ini

| Workstream | Status | Evidence |
|---|---|---|
| ManGApp naming dan `mangap` package/binary | Implemented | `Cargo.toml`, `src/main.rs`, `README.md` |
| Sources page dan source registry | Implemented | `src/registry.rs`, `src/ui.rs` |
| Source status, grouping enabled sebelum `DISABLED`, dan nomor urut | Implemented | `src/model.rs`, `src/ui.rs`, `tests/ui_contract.rs` |
| PATH listing dengan urutan resolve sistem | Implemented | `src/detect.rs`, `src/ui.rs`, `tests/detect_contract.rs` |
| PATH source hint, duplicate warning, missing-folder note | Implemented | `src/detect.rs`, `src/ui.rs` |
| README publik dan dokumentasi status | Implemented | `README.md`, `docs/superpowers/` |
| Daftar aplikasi per source | Planned | Belum ada application collector |
| Kontrak kolom aplikasi lintas source | Planned | Belum ada model aplikasi terpadu |
| Deteksi storage per aplikasi | Planned | Belum ada storage collector |

## Bukti Validasi Terakhir

Pada rename terakhir ke ManGApp, repository telah melewati:

- `cargo fmt --all -- --check` — lulus.
- `cargo test --all-targets` — 26 test lulus.
- `cargo clippy --all-targets --all-features -- -D warnings` — lulus.
- `cargo llvm-cov --all-targets --summary-only --fail-under-lines 80` — lulus, line coverage 89,35%.

## Riwayat Pengerjaan

| Perubahan | Status | Commit / Evidence |
|---|---|---|
| Scaffold Rust + model, registry, detector, dan UI Sources | Implemented | `550be8d`, `a4bd5b9`, `bf68517`, `7b0148e` |
| Penghapusan script `list_*` lama setelah pengganti solid | Implemented | `526a4aa` |
| PATH panel, scrolling, focus row, source hint, dan kolom note | Implemented | `7e0ebc1`, `17e8ca0`, `87a2e7c` |
| Pengurutan enabled sebelum `DISABLED` dan nomor urut | Implemented | `f97de67`, `9e11e8d`, `8d21ac7` |
| Rename menjadi ManGApp / `mangap` | Implemented | `e096e8d`, `82c0da5` |
| Daftar aplikasi dari source | Planned | Menunggu tahap berikutnya |
| Deteksi dan ringkasan storage | Planned | Menunggu model aplikasi |

## Global Constraints

- Nama package dan binary harus `mangap`.
- Tahap ini hanya mengimplementasikan halaman Sources; daftar aplikasi belum termasuk.
- Registry menyimpan semua sumber dalam urutan alfabetis berdasarkan nama case-insensitive; UI menampilkan source non-`DISABLED` lebih dulu, lalu `DISABLED`, dengan urutan alfabetis di dalam setiap kelompok.
- Sumber yang command-nya tidak ditemukan harus terlihat sebagai `DISABLED`, bukan dihilangkan.
- Sumber yang ditemukan tetapi probe-nya gagal harus terlihat sebagai `ERROR`, bukan `DISABLED`.
- `cleanup.sh` dan `update.sh` tetap dipertahankan.
- Penghapusan script `list_*` dan test lama dilakukan setelah aplikasi dan test baru solid, dalam commit terpisah.
- Setiap task berakhir dengan test dan commit kecil yang hanya berisi perubahan task tersebut.

---

### Task 1: Scaffold Cargo dan kontrak model melalui test — Implemented

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/model.rs`
- Create: `tests/model_contract.rs`

**Interfaces:**
- Produces `SourceDefinition`, `SourceStatus`, `SourceSnapshot`, dan helper status yang akan dipakai registry, detector, serta UI.

- [x] **Step 1: Create Cargo manifest and failing contract tests**

  `Cargo.toml` harus mendefinisikan package `mangap`, edition `2024`, dependencies `ratatui = "0.30.2"` dan `crossterm = "0.29"`, serta library target default dari `src/lib.rs`. Test awal di `tests/model_contract.rs` harus menguji bahwa status dapat dibedakan:

  ```rust
  use mangap::model::{SourceStatus, status_label};

  #[test]
  fn status_labels_distinguish_available_disabled_and_error() {
      assert_eq!(status_label(&SourceStatus::Available { executable: "brew".into() }), "AVAILABLE");
      assert_eq!(status_label(&SourceStatus::Disabled { candidates: vec!["brew".into()] }), "DISABLED");
      assert_eq!(status_label(&SourceStatus::Error { message: "probe failed".into() }), "ERROR");
  }
  ```

- [x] **Step 2: Run the focused test and verify the expected RED failure**

  Run: `cargo test --test model_contract`

  Expected: compilation fails because `mangap::model` and its types/functions do not exist yet.

- [x] **Step 3: Implement the minimal model**

  Add `src/model.rs` with:

  ```rust
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct SourceDefinition {
      pub id: &'static str,
      pub name: &'static str,
      pub candidates: &'static [&'static str],
      pub category: &'static str,
  }

  #[derive(Clone, Debug, PartialEq, Eq)]
  pub enum SourceStatus {
      Available { executable: std::path::PathBuf },
      Disabled { candidates: Vec<String> },
      Error { message: String },
  }

  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct SourceSnapshot {
      pub definition: &'static SourceDefinition,
      pub status: SourceStatus,
  }
  ```

  Export the module and `status_label` from `src/lib.rs`. Make `status_label` return exactly `AVAILABLE`, `DISABLED`, or `ERROR`.

- [x] **Step 4: Run the focused test and verify GREEN**

  Run: `cargo test --test model_contract`

  Expected: PASS.

- [x] **Step 5: Commit the scaffold and model contract**

  ```bash
  git add Cargo.toml Cargo.lock src/lib.rs src/model.rs tests/model_contract.rs
  git commit -m "feat: scaffold mangap source model"
  ```

### Task 2: Add the alphabetic source registry — Implemented

**Files:**
- Create: `src/registry.rs`
- Modify: `src/lib.rs`
- Create: `tests/registry_contract.rs`

**Interfaces:**
- Consumes: `SourceDefinition` from `model`.
- Produces: `pub fn source_registry() -> &'static [SourceDefinition]`.

- [x] **Step 1: Write failing registry tests**

  Test that the registry contains exactly the 17 catalog entries from the spec and that the names are sorted case-insensitively:

  ```rust
  use mangap::registry::source_registry;

  #[test]
  fn registry_contains_supported_sources_in_alphabetic_order() {
      let sources = source_registry();
      assert_eq!(sources.len(), 17);
      assert!(sources.windows(2).all(|pair| pair[0].name.to_lowercase() <= pair[1].name.to_lowercase()));
      assert!(sources.iter().any(|source| source.name == "Homebrew"));
      assert!(sources.iter().any(|source| source.name == "Mac App Store"));
      assert!(sources.iter().any(|source| source.name == "Composer"));
  }

  #[test]
  fn every_registry_entry_has_complete_metadata() {
      assert!(source_registry().iter().all(|source| {
          !source.id.is_empty() && !source.name.is_empty() && !source.candidates.is_empty() && !source.category.is_empty()
      }));
  }
  ```

- [x] **Step 2: Run tests and verify RED**

  Run: `cargo test --test registry_contract`

  Expected: compilation fails because `source_registry` does not exist.

- [x] **Step 3: Implement the registry**

  Define all entries exactly once in `src/registry.rs`, in this order: Cargo, Composer, Conda, Dart, Flutter, Go, Homebrew, Mac App Store, MacPorts, Nix, npm, pipx, pnpm, Python pip, RubyGems, uv, Yarn. Use static definitions and candidate aliases such as `pip3`, `pip`, `python3` for Python pip and `conda`, `mamba` for Conda. Export `source_registry` from `src/lib.rs`.

- [x] **Step 4: Run tests and verify GREEN**

  Run: `cargo test --test registry_contract`

  Expected: PASS.

- [x] **Step 5: Commit the registry**

  ```bash
  git add src/lib.rs src/registry.rs tests/registry_contract.rs
  git commit -m "feat: add alphabetic mangap source registry"
  ```

### Task 3: Implement a testable command detector — Implemented

**Files:**
- Create: `src/detect.rs`
- Modify: `src/lib.rs`
- Create: `tests/detect_contract.rs`

**Interfaces:**
- Consumes: `SourceDefinition` and `SourceSnapshot` model types.
- Produces: `pub trait CommandResolver`, `pub struct PathResolver`, and `pub fn detect_source`.

- [x] **Step 1: Write failing detector tests**

  Use a fake resolver so tests never depend on the host machine:

  ```rust
  struct FakeResolver(Result<Option<std::path::PathBuf>, String>);

  impl CommandResolver for FakeResolver {
      fn resolve(&self, _candidate: &str) -> Result<Option<std::path::PathBuf>, String> {
          self.0.clone()
      }
  }
  ```

  Add tests for available, disabled, and error statuses, including the selected executable path and error message.

- [x] **Step 2: Run tests and verify RED**

  Run: `cargo test --test detect_contract`

  Expected: compilation fails because the detector interfaces do not exist.

- [x] **Step 3: Implement the detector**

  `CommandResolver::resolve` returns `Ok(Some(path))` for an available candidate, `Ok(None)` when absent, or `Err(message)` for a probe failure. `detect_source` must try candidates in definition order, return `Available` on the first match, return `Error` on a resolver error, and return `Disabled` only after every candidate is absent. `PathResolver` searches each component of a supplied `PATH` string and accepts executable files.

- [x] **Step 4: Run tests and verify GREEN**

  Run: `cargo test --test detect_contract`

  Expected: PASS.

- [x] **Step 5: Commit the detector**

  ```bash
  git add src/lib.rs src/detect.rs tests/detect_contract.rs
  git commit -m "feat: detect installed source commands"
  ```

### Task 4: Build the Sources page and UI state — Implemented

**Files:**
- Create: `src/ui.rs`
- Modify: `src/lib.rs`
- Create: `tests/ui_contract.rs`

**Interfaces:**
- Consumes: `SourceSnapshot`, `SourceStatus`, and registry output.
- Produces: `pub struct App`, `pub enum AppAction`, `App::handle_key`, and `ui::render`.

- [x] **Step 1: Write failing state and rendering tests**

  Test that `App::handle_key` supports Down, Up, PageDown, PageUp, Enter, `r`, `q`, and Esc. Test rendering with `ratatui::backend::TestBackend` and assert the first row contains `Cargo`, the header contains `Status`, `Sumber`, `Command`, `Kategori`, and `Keterangan`, and a disabled source contains `DISABLED`.

- [x] **Step 2: Run tests and verify RED**

  Run: `cargo test --test ui_contract`

  Expected: compilation fails because `App`, `AppAction`, and `render` do not exist.

- [x] **Step 3: Implement the UI**

  Render the title `MangApp — Sources`, an alphabetic table, a detail panel, and a footer with keyboard help. Use green for available rows, dark gray for disabled rows, and yellow/red for error rows. Keep disabled rows visible but skip them when moving the active selection. Render missing data as `—` and status-specific explanatory text. Keep layout bounds safe for small terminals by calculating widths from `Rect` and never writing outside the frame.

- [x] **Step 4: Run tests and verify GREEN**

  Run: `cargo test --test ui_contract`

  Expected: PASS.

- [x] **Step 5: Commit the UI**

  ```bash
  git add src/lib.rs src/ui.rs tests/ui_contract.rs
  git commit -m "feat: render mangap sources page"
  ```

### Task 5: Add the binary lifecycle and manual terminal smoke test — Implemented

**Files:**
- Create: `src/main.rs`
- Create: `tests/cli_contract.rs`
- Modify: `Cargo.toml` only if explicit binary metadata is needed.

**Interfaces:**
- Consumes: registry, detector, `App`, and renderer.
- Produces: executable `mangap` and clean terminal lifecycle.

- [x] **Step 1: Write failing CLI contract test**

  Add a test that verifies the Cargo package exposes a binary named `mangap`; add a unit-level lifecycle test around the terminal guard where practical. Keep all terminal I/O behind a small function so the rest of the behavior remains testable.

- [x] **Step 2: Run tests and verify RED**

  Run: `cargo test --test cli_contract`

  Expected: failure because the binary and lifecycle entry points do not exist.

- [x] **Step 3: Implement `main`**

  Probe all registry entries before opening the alternate screen. Configure raw mode, alternate screen, mouse-free keyboard events, and a cleanup guard. On `r`, re-run all probes and replace the snapshot. On `q`/Esc, restore raw mode and the previous screen before returning. Return errors from `main` with a concise message and non-zero exit status.

- [x] **Step 4: Run automated tests and manual smoke test**

  Run:

  ```bash
  cargo fmt --all -- --check
  cargo test --all-targets
  cargo clippy --all-targets --all-features -- -D warnings
  cargo run
  ```

  In the terminal, confirm the first row is `Cargo`, unavailable tools remain visible as disabled, `r` refreshes, and `q` exits without leaving raw/alternate mode.

- [x] **Step 5: Commit the executable**

  ```bash
  git add Cargo.toml Cargo.lock src/main.rs tests/cli_contract.rs
  git commit -m "feat: add mangap sources executable"
  ```

### Task 6: Reach and verify 80% coverage — Implemented

**Files:**
- Modify: `tests/model_contract.rs`
- Modify: `tests/registry_contract.rs`
- Modify: `tests/detect_contract.rs`
- Modify: `tests/ui_contract.rs`
- Modify: `tests/cli_contract.rs`

- [x] **Step 1: Install the coverage tool if missing**

  Run: `cargo install cargo-llvm-cov --locked`.

- [x] **Step 2: Measure the current coverage**

  Run: `cargo llvm-cov --all-targets --summary-only`.

  Record the reported line percentage and identify uncovered branches in registry, detector, UI navigation, small-terminal layout, error handling, and cleanup.

- [x] **Step 3: Add tests for uncovered behavior before changing production code**

  Add focused tests for every uncovered branch, including empty registry rendering, all-disabled selection, resolver errors, refresh action, quit actions, page bounds, and narrow frame rendering. Do not lower coverage thresholds or exclude application code to make the number pass.

- [x] **Step 4: Verify the 80% target**

  Run:

  ```bash
  cargo fmt --all -- --check
  cargo test --all-targets
  cargo llvm-cov --all-targets --summary-only
  ```

  Expected: all tests pass and total line coverage is at least 80%.

- [x] **Step 5: Commit coverage improvements**

  ```bash
  git add tests
  git commit -m "test: raise mangap coverage above eighty percent"
  ```

### Task 7: Remove obsolete list scripts and tests — Implemented

**Files:**
- Delete: `list_homebrew.sh`
- Delete: `list_mas.sh`
- Delete: `list_npm.sh`
- Delete: `list_pip.sh`
- Delete: `list_uv.sh`
- Delete: `list_gem.sh`
- Delete: `list_cargo.sh`
- Delete: `test/test_list_homebrew_curses.zsh`
- Delete: `test/test_list_mas_ui.zsh`

- [x] **Step 1: Verify the replacement before deletion**

  Run: `cargo test --all-targets`, `cargo llvm-cov --all-targets --summary-only`, and `cargo run` smoke test. Expected: tests pass, coverage is at least 80%, and Sources page is usable.

- [x] **Step 2: Delete only obsolete list scripts and their tests**

  Remove the nine files listed above. Do not remove `cleanup.sh`, `update.sh`, `.gitignore`, or any Rust source/spec/plan files.

- [x] **Step 3: Verify deletion scope and the full repository**

  Run:

  ```bash
  test -z "$(find . -maxdepth 1 -type f -name 'list_*' -print)"
  test -z "$(find test -maxdepth 1 -type f -name 'test_list_*' -print 2>/dev/null)"
  cargo fmt --all -- --check
  cargo test --all-targets
  cargo clippy --all-targets --all-features -- -D warnings
  cargo llvm-cov --all-targets --summary-only
  git diff --check
  git status --short
  ```

- [x] **Step 4: Commit the removal separately**

  ```bash
  git add -u list_homebrew.sh list_mas.sh list_npm.sh list_pip.sh list_uv.sh list_gem.sh list_cargo.sh test/test_list_homebrew_curses.zsh test/test_list_mas_ui.zsh
  git commit -m "chore: remove legacy list scripts"
  ```

## Forward Plan

Tahap berikutnya baru dimulai setelah Sources page dipertahankan sebagai baseline yang stabil.

### Task 8: Application inventory per source — Planned

- Define model aplikasi bersama: nama, identifier, version, source, install location, dan availability setiap field.
- Tambahkan adapter collector untuk source yang tersedia tanpa menghilangkan source yang `DISABLED`.
- Tambahkan halaman Applications yang dapat dibuka dari Sources dan memiliki empty/loading/error state yang jelas.
- Tambahkan test kontrak per adapter serta test UI untuk source dengan field yang tidak tersedia.

### Task 9: Unified application table — Planned

- Tetapkan kolom tetap lintas source.
- Pertahankan kolom ketika data tidak tersedia; tampilkan flag atau warna pada cell terkait.
- Pastikan sorting, filtering, focus, dan scrolling konsisten dengan Sources page.
- Tambahkan acceptance test untuk alignment kolom pada terminal lebar dan sempit.

### Task 10: Storage inspection — Planned

- Tentukan definisi storage: binary, package files, cache, data user, dan total yang dapat diukur.
- Implementasikan collector storage yang aman dan tidak menghapus atau mengubah file.
- Tampilkan status `Unavailable` ketika pengukuran tidak didukung atau gagal.
- Tambahkan test untuk path hilang, permission error, symlink, dan ukuran directory besar.

### Task 11: Release readiness — Planned

- Perbarui README publik dengan fitur yang benar-benar sudah tersedia.
- Tambahkan test strategy atau release checklist ketika halaman Applications mulai diimplementasikan.
- Jalankan format, test, lint, coverage, dan interactive smoke test sebelum merge ke `main`.

## Plan self-review

- Alphabetical ordering is specified in the registry task and asserted in `registry_contract`.
- Disabled support is represented in the model, detector, renderer, and UI tests.
- The application-list phase is explicitly excluded from every implementation task.
- Coverage is measured after implementation and before deletion; tests are added for uncovered branches rather than hiding them.
- Deletion is isolated in its own commit and leaves unrelated maintenance scripts intact.
