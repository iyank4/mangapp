use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use mangap::{
    inventory::{CommandRunner, collect_inventory_with_runner, upgrade_inventory_with_runner},
    model::{ApplicationRecord, CellValue, RecordStatus, SourceSnapshot, SourceStatus},
    registry::source_registry,
    ui::{App, render},
};
use ratatui::{Terminal, backend::TestBackend};

struct FakeRunner;

impl CommandRunner for FakeRunner {
    fn run(&self, executable: &Path, args: &[String]) -> Result<String, String> {
        match executable.file_name().and_then(|name| name.to_str()) {
            Some("cargo") if args.first().map(String::as_str) == Some("search") => {
                Ok("ripgrep = \"14.2.0\"    # fast search tool\n".into())
            }
            Some("cargo") => Ok("ripgrep v14.1.1:\n    rg\n\nbat v0.25.0:\n    bat\n".into()),
            Some("brew") if args.first().map(String::as_str) == Some("outdated") => {
                Ok(r#"{"formulae":[{"name":"git","current_version":"2.51.0"}],"casks":[]}"#.into())
            }
            Some("brew") if args.get(1).map(String::as_str) == Some("--formula") => {
                Ok("git 2.50.0\n".into())
            }
            Some("brew") => Ok("visual-studio-code 1.99.0\n".into()),
            Some("mas") => Err("mas list gagal".into()),
            _ => Err("unexpected command".into()),
        }
    }
}

struct AllSourcesRunner {
    calls: RefCell<Vec<(String, Vec<String>)>>,
}

impl AllSourcesRunner {
    fn new() -> Self {
        Self {
            calls: RefCell::new(Vec::new()),
        }
    }
}

impl CommandRunner for AllSourcesRunner {
    fn run(&self, executable: &Path, args: &[String]) -> Result<String, String> {
        let name = executable
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_owned();
        self.calls.borrow_mut().push((name.clone(), args.to_vec()));
        Ok(match (name.as_str(), args) {
            ("cargo", _) => "ripgrep v14.1.1:\n    rg\n".into(),
            ("brew", [_, flag, _]) if flag == "--formula" => "git 2.50.0\n".into(),
            ("brew", _) => "visual-studio-code 1.99.0\n".into(),
            ("mas", _) => "497799835 Pixelmator Pro (3.6.10)\n".into(),
            ("pip3", _) => "requests==2.32.3\n".into(),
            ("gem", _) => "rails (8.0.0)\n".into(),
            ("uv", _) => "ruff v0.8.0\n".into(),
            ("pipx", _) => "  package black 24.10.0, installed using Python 3.13.0\n".into(),
            ("npm", _) | ("pnpm", _) | ("yarn", _) => "├── typescript@5.7.2\n".into(),
            ("composer", _) => "vendor/package v1.2.3\n".into(),
            ("conda", _) => "# packages in environment\nnumpy 2.1.0 py312_0\n".into(),
            ("port", _) => "  wget @2.2.0_0 (active)\n".into(),
            ("nix", _) => "Name: ripgrep\n".into(),
            ("dart", _) | ("flutter", _) => "melos 6.3.0\n".into(),
            ("go", _) => "\n\n".into(),
            _ => return Err(format!("unexpected adapter command: {name} {args:?}")),
        })
    }
}

fn available(definition: &'static mangap::model::SourceDefinition) -> SourceSnapshot {
    SourceSnapshot {
        definition,
        status: SourceStatus::Available {
            executable: PathBuf::from(format!("/usr/local/bin/{}", definition.candidates[0])),
        },
    }
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn record(source: &str, name: &str, identifier: &str) -> ApplicationRecord {
    let mut record = ApplicationRecord::new(
        source,
        CellValue::value(identifier),
        CellValue::value(name),
        CellValue::value("1.2.3 (Build 4)"),
        RecordStatus::Available,
    );
    record.location = CellValue::value("/Applications/Example.app");
    record.available_version = CellValue::value("1.3.0");
    record.installed_at = CellValue::value("2026-01-02");
    record.updated_at = CellValue::value("2026-02-03");
    record.note = CellValue::value("metadata dari fixture");
    record
}

#[test]
fn collectors_normalize_records_and_keep_source_in_record_key() {
    let sources = vec![
        available(&source_registry()[0]),
        available(&source_registry()[6]),
    ];
    let records = collect_inventory_with_runner(&sources, &FakeRunner);

    assert_eq!(records.len(), 4);
    assert_eq!(records[0].source, "cargo");
    assert_eq!(records[0].identifier.display(), "ripgrep");
    assert_eq!(records[0].version.display(), "14.1.1");
    assert_eq!(records[0].available_version.display(), "14.2.0");
    assert_eq!(records[0].record_key, "cargo:ripgrep");
    assert_eq!(records[2].source, "homebrew");
    assert_eq!(records[2].available_version.display(), "2.51.0");
    assert_eq!(records[3].source, "homebrew");
}

#[test]
fn one_source_error_does_not_remove_records_from_other_sources() {
    let sources = vec![
        available(&source_registry()[0]),
        available(&source_registry()[7]),
    ];
    let records = collect_inventory_with_runner(&sources, &FakeRunner);

    assert!(records.iter().any(|record| record.source == "cargo"));
    let error = records
        .iter()
        .find(|record| record.source == "mas")
        .expect("error record");
    assert_eq!(error.status, RecordStatus::Error);
    assert_eq!(error.note.display(), "mas list gagal");
}

#[test]
fn inventory_sorts_filters_and_numbers_visible_rows() {
    let mut app = App::with_inventory(
        vec![
            available(&source_registry()[0]),
            available(&source_registry()[6]),
        ],
        vec![
            record("homebrew", "Zeta", "zeta"),
            record("homebrew", "Alpha", "alpha"),
            record("cargo", "Alpha", "alpha"),
        ],
    );

    app.handle_key(key(KeyCode::Down));
    app.handle_key(key(KeyCode::Enter));
    assert_eq!(app.inventory_source(), Some("homebrew"));
    assert_eq!(app.selected_inventory_index(), Some(1));
    app.handle_key(key(KeyCode::Char('f')));
    for character in "homebrew".chars() {
        app.handle_key(key(KeyCode::Char(character)));
    }
    app.handle_key(key(KeyCode::Enter));
    assert_eq!(app.selected_inventory_index(), Some(1));
    assert_eq!(app.inventory_filter(), "homebrew");
}

#[test]
fn inventory_page_renders_fixed_columns_and_detail_fields() {
    let mut app = App::with_inventory(
        vec![available(&source_registry()[0])],
        vec![record("cargo", "Ripgrep", "ripgrep")],
    );
    app.handle_key(key(KeyCode::Enter));
    app.handle_key(key(KeyCode::Enter));

    let mut terminal = Terminal::new(TestBackend::new(140, 30)).expect("test terminal");
    terminal
        .draw(|frame| render(frame, &app))
        .expect("render inventory");
    let content: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();

    for header in [
        "Aplikasi",
        "Versi",
        "Versi Baru",
        "Sumber",
        "Status",
        "Identifier",
        "Lokasi",
    ] {
        assert!(content.contains(header), "missing header: {header}");
    }
    assert!(content.contains("Source: Cargo"));
    assert!(content.contains("Aplikasi       : Ripgrep"));
    assert!(content.contains("Versi          : 1.2.3 (Build 4)"));
    assert!(content.contains("Versi Baru     : 1.3.0"));
    assert!(content.contains("Sumber         : cargo"));
    assert!(content.contains("Status         : AVAILABLE"));
    assert!(content.contains("Tanggal Install: 2026-01-02"));
    assert!(content.contains("Tanggal Update : 2026-02-03"));
    assert!(content.contains("Note           : metadata dari fixture"));
}

#[test]
fn inventory_page_does_not_render_path_section() {
    let mut app = App::with_path_entries_and_inventory(
        vec![available(&source_registry()[0])],
        vec![record("cargo", "Ripgrep", "ripgrep")],
        vec![PathBuf::from("/path/entry")],
    );
    app.handle_key(key(KeyCode::Char('p')));
    app.handle_key(key(KeyCode::Tab));
    app.handle_key(key(KeyCode::Enter));

    let mut terminal = Terminal::new(TestBackend::new(140, 30)).expect("test terminal");
    terminal
        .draw(|frame| render(frame, &app))
        .expect("render inventory without path");
    let content: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();

    assert!(content.contains("MangApp — Application Inventory"));
    assert!(!content.contains("PATH directories"));
    assert!(!content.contains("PATH"));
    assert!(!content.contains("/path/entry"));
}

#[test]
fn inventory_empty_state_spans_the_table_width() {
    let mut app = App::with_inventory(vec![available(&source_registry()[0])], Vec::new());
    app.handle_key(key(KeyCode::Enter));

    let mut terminal = Terminal::new(TestBackend::new(140, 30)).expect("test terminal");
    terminal
        .draw(|frame| render(frame, &app))
        .expect("render empty inventory");
    let content: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();

    assert!(content.contains("Belum ada aplikasi yang terdeteksi"));
}

#[test]
fn narrow_inventory_keeps_columns_and_supports_horizontal_scroll() {
    let mut app = App::with_inventory(
        vec![available(&source_registry()[0])],
        vec![record("cargo", "Ripgrep", "ripgrep")],
    );
    app.handle_key(key(KeyCode::Enter));
    for _ in 0..20 {
        app.handle_key(key(KeyCode::Right));
    }
    assert!(app.inventory_horizontal_scroll() > 0);

    let mut terminal = Terminal::new(TestBackend::new(80, 30)).expect("test terminal");
    terminal
        .draw(|frame| render(frame, &app))
        .expect("render narrow inventory");
    let content: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(content.contains("Lokasi"));
}

#[test]
fn refresh_preserves_inventory_filter_and_selected_record() {
    let sources = vec![available(&source_registry()[0])];
    let mut app = App::with_inventory(
        sources.clone(),
        vec![
            record("cargo", "Alpha", "alpha"),
            record("cargo", "Beta", "beta"),
        ],
    );
    app.handle_key(key(KeyCode::Enter));
    app.handle_key(key(KeyCode::Char('f')));
    for character in "beta".chars() {
        app.handle_key(key(KeyCode::Char(character)));
    }
    app.handle_key(key(KeyCode::Enter));
    assert_eq!(app.selected_inventory_index(), Some(1));

    app.replace_data(sources, vec![record("cargo", "Beta", "beta")]);
    assert_eq!(app.inventory_filter(), "beta");
    assert_eq!(app.selected_inventory_index(), Some(0));
}

#[test]
fn every_registry_source_has_a_working_inventory_adapter() {
    let sources = source_registry().iter().map(available).collect::<Vec<_>>();
    let runner = AllSourcesRunner::new();
    let records = collect_inventory_with_runner(&sources, &runner);

    assert!(
        records
            .iter()
            .all(|record| record.status != RecordStatus::Error)
    );
    assert!(records.iter().any(|record| record.source == "cargo"));
    assert!(records.iter().any(|record| record.source == "conda"));
    assert!(records.iter().any(|record| record.source == "macports"));
    assert!(records.iter().any(|record| record.source == "nix"));

    let called = runner
        .calls
        .borrow()
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    for definition in source_registry() {
        assert!(
            called.iter().any(|name| name == definition.candidates[0]),
            "missing adapter call for {}",
            definition.id
        );
    }
    let yarn_args = runner
        .calls
        .borrow()
        .iter()
        .find(|(name, _)| name == "yarn")
        .map(|(_, args)| args.clone())
        .expect("yarn adapter call");
    assert_eq!(yarn_args, ["global", "list", "--depth=0"]);
}

#[test]
fn upgrade_uses_batch_commands_and_upgrades_mas_one_by_one() {
    let runner = AllSourcesRunner::new();
    let cargo = available(&source_registry()[0]);
    upgrade_inventory_with_runner(&cargo, &[record("cargo", "Ripgrep", "ripgrep")], &runner)
        .expect("upgrade cargo");

    let mas = available(&source_registry()[7]);
    let mas_records = vec![
        record("mas", "Pixelmator Pro", "497799835"),
        record("mas", "Xcode", "497799835-2"),
    ];
    upgrade_inventory_with_runner(&mas, &mas_records, &runner).expect("upgrade mas");

    let calls = runner.calls.borrow();
    assert!(
        calls
            .iter()
            .any(|(name, args)| { name == "cargo" && args == &["install", "--force", "ripgrep"] })
    );
    let mas_calls = calls
        .iter()
        .filter(|(name, args)| name == "mas" && args.first().map(String::as_str) == Some("upgrade"))
        .count();
    assert_eq!(mas_calls, 2);
}

#[test]
fn disabled_sources_remain_visible_as_unavailable_inventory_status() {
    let definition = &source_registry()[7];
    let source = SourceSnapshot {
        definition,
        status: SourceStatus::Disabled {
            candidates: definition
                .candidates
                .iter()
                .map(|candidate| (*candidate).to_owned())
                .collect(),
        },
    };

    let records = collect_inventory_with_runner(&[source], &FakeRunner);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].source, "mas");
    assert_eq!(records[0].status, RecordStatus::Unavailable);
    assert_eq!(records[0].name.display(), "UNAVAILABLE");
    assert!(records[0].note.display().contains("mas"));
}

#[test]
fn inventory_renders_loading_and_unavailable_states_without_panicking() {
    let definition = &source_registry()[7];
    let unavailable = ApplicationRecord::new(
        "mas",
        CellValue::unavailable(),
        CellValue::unavailable(),
        CellValue::unavailable(),
        RecordStatus::Unavailable,
    );
    let mut app = App::with_inventory(vec![available(definition)], vec![unavailable]);
    app.handle_key(key(KeyCode::Enter));
    app.begin_inventory_loading();

    let mut terminal = Terminal::new(TestBackend::new(140, 30)).expect("test terminal");
    terminal
        .draw(|frame| render(frame, &app))
        .expect("render loading state");
    let loading: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(loading.contains("Memuat daftar aplikasi"));

    let inventory = app.inventory().to_vec();
    app.replace_data(vec![available(definition)], inventory);
    terminal
        .draw(|frame| render(frame, &app))
        .expect("render unavailable state");
    let unavailable_view: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(unavailable_view.contains("UNAVAILABLE"));
}
