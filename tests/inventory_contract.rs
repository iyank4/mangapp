use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use mangap::{
    inventory::{CommandRunner, collect_inventory_with_runner},
    model::{ApplicationRecord, CellValue, RecordStatus, SourceSnapshot, SourceStatus},
    registry::source_registry,
    ui::{App, render},
};
use ratatui::{Terminal, backend::TestBackend};

struct FakeRunner;

impl CommandRunner for FakeRunner {
    fn run(&self, executable: &Path, args: &[String]) -> Result<String, String> {
        match executable.file_name().and_then(|name| name.to_str()) {
            Some("cargo") => Ok("ripgrep v14.1.1:\n    rg\n\nbat v0.25.0:\n    bat\n".into()),
            Some("brew") if args.get(1).map(String::as_str) == Some("--formula") => {
                Ok("git 2.50.0\n".into())
            }
            Some("brew") => Ok("visual-studio-code 1.99.0\n".into()),
            Some("mas") => Err("mas list gagal".into()),
            _ => Err("unexpected command".into()),
        }
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
    assert_eq!(records[0].record_key, "cargo:ripgrep");
    assert_eq!(records[2].source, "homebrew");
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

    app.handle_key(key(KeyCode::Char('i')));
    assert_eq!(app.selected_inventory_index(), Some(0));
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
    app.handle_key(key(KeyCode::Char('i')));
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
        "Sumber",
        "Status",
        "Identifier",
        "Lokasi",
    ] {
        assert!(content.contains(header), "missing header: {header}");
    }
    assert!(content.contains("Tanggal Install: 2026-01-02"));
    assert!(content.contains("Tanggal Update: 2026-02-03"));
    assert!(content.contains("Note: metadata dari fixture"));
}

#[test]
fn narrow_inventory_keeps_columns_and_supports_horizontal_scroll() {
    let mut app = App::with_inventory(
        vec![available(&source_registry()[0])],
        vec![record("cargo", "Ripgrep", "ripgrep")],
    );
    app.handle_key(key(KeyCode::Char('i')));
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
    app.handle_key(key(KeyCode::Char('i')));
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
