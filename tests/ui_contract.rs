use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use manapp::model::{SourceSnapshot, SourceStatus};
use manapp::registry::source_registry;
use manapp::ui::{App, AppAction, render};
use ratatui::{Terminal, backend::TestBackend, style::Color};

fn snapshots() -> Vec<SourceSnapshot> {
    source_registry()
        .iter()
        .enumerate()
        .map(|(index, definition)| SourceSnapshot {
            definition,
            status: match index {
                0 | 6 | 16 => SourceStatus::Available {
                    executable: PathBuf::from(format!(
                        "/usr/local/bin/{}",
                        definition.candidates[0]
                    )),
                },
                _ => SourceStatus::Disabled {
                    candidates: definition
                        .candidates
                        .iter()
                        .map(|candidate| (*candidate).to_string())
                        .collect(),
                },
            },
        })
        .collect()
}

fn path_entries() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/first/bin"),
        PathBuf::from("/second/bin"),
        PathBuf::from("/first/bin"),
    ]
}

fn numbered_path_entries() -> Vec<PathBuf> {
    (1..=40)
        .map(|index| PathBuf::from(format!("/path/{index:02}")))
        .collect()
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn app_navigates_enabled_rows_and_handles_actions() {
    let mut app = App::new(snapshots());
    assert_eq!(app.selected_index(), Some(0));

    app.handle_key(key(KeyCode::Down));
    assert_eq!(app.selected_index(), Some(6));
    app.handle_key(key(KeyCode::Up));
    assert_eq!(app.selected_index(), Some(0));
    app.handle_key(key(KeyCode::PageDown));
    assert_eq!(app.selected_index(), Some(16));
    app.handle_key(key(KeyCode::PageUp));
    assert_eq!(app.selected_index(), Some(0));

    assert_eq!(app.handle_key(key(KeyCode::Enter)), AppAction::None);
    assert!(app.detail_visible());
    assert_eq!(app.handle_key(key(KeyCode::Char('r'))), AppAction::Refresh);
    assert_eq!(app.handle_key(key(KeyCode::Char('q'))), AppAction::Quit);
    assert_eq!(app.handle_key(key(KeyCode::Esc)), AppAction::Quit);
}

#[test]
fn app_has_no_selection_when_all_sources_are_disabled() {
    let disabled = source_registry()
        .iter()
        .map(|definition| SourceSnapshot {
            definition,
            status: SourceStatus::Disabled {
                candidates: definition
                    .candidates
                    .iter()
                    .map(|candidate| (*candidate).to_string())
                    .collect(),
            },
        })
        .collect();
    let mut app = App::new(disabled);

    assert_eq!(app.selected_index(), None);
    app.handle_key(key(KeyCode::Down));
    assert_eq!(app.selected_index(), None);
}

#[test]
fn sources_page_renders_headers_rows_and_disabled_status() {
    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    let app = App::new(snapshots());

    terminal
        .draw(|frame| render(frame, &app))
        .expect("render sources page");

    let content: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(content.contains("manapp — Sources"));
    assert!(content.contains("Status"));
    assert!(content.contains("Sumber"));
    assert!(content.contains("Command"));
    assert!(content.contains("Kategori"));
    assert!(content.contains("Keterangan"));
    assert!(content.contains("Cargo"));
    assert!(content.contains("DISABLED"));
}

#[test]
fn app_supports_home_end_replace_and_ignores_unknown_keys() {
    let mut app = App::new(snapshots());

    assert_eq!(app.handle_key(key(KeyCode::Char('x'))), AppAction::None);
    app.handle_key(key(KeyCode::End));
    assert_eq!(app.selected_index(), Some(16));
    app.handle_key(key(KeyCode::Home));
    assert_eq!(app.selected_index(), Some(0));

    app.replace_sources(Vec::new());
    assert_eq!(app.selected_index(), None);
    assert_eq!(app.handle_key(key(KeyCode::End)), AppAction::None);
}

#[test]
fn sources_page_renders_detail_and_error_styles() {
    let mut sources = snapshots();
    sources[1].status = SourceStatus::Error {
        message: "probe gagal".into(),
    };
    let mut app = App::new(sources);
    app.handle_key(key(KeyCode::Enter));

    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    terminal
        .draw(|frame| render(frame, &app))
        .expect("render detail");
    let content: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(content.contains("executable: /usr/local/bin/cargo"));

    app.handle_key(key(KeyCode::Down));
    terminal
        .draw(|frame| render(frame, &app))
        .expect("render error detail");
    let error_content: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(error_content.contains("ERROR"));
    assert!(error_content.contains("probe gagal"));
}

#[test]
fn sources_page_renders_disabled_detail_message_without_selection() {
    let sources = source_registry()
        .iter()
        .map(|definition| SourceSnapshot {
            definition,
            status: SourceStatus::Disabled {
                candidates: definition
                    .candidates
                    .iter()
                    .map(|candidate| (*candidate).to_string())
                    .collect(),
            },
        })
        .collect();
    let app = App::new(sources);
    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    terminal
        .draw(|frame| render(frame, &app))
        .expect("render disabled detail");
    let content: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(content.contains("Sumber disabled tetap ditampilkan"));
}

#[test]
fn path_section_is_hidden_by_default_below_detail() {
    let app = App::with_path_entries(snapshots(), path_entries());
    assert!(!app.path_visible());

    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    terminal
        .draw(|frame| render(frame, &app))
        .expect("render hidden path section");
    let content: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();

    assert!(content.contains("PATH directories: hidden"));
    assert!(!content.contains("/first/bin"));
}

#[test]
fn path_section_toggles_below_detail_and_preserves_path_order() {
    let mut app = App::with_path_entries(snapshots(), path_entries());
    app.handle_key(key(KeyCode::Char('p')));
    assert!(app.path_visible());

    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    terminal
        .draw(|frame| render(frame, &app))
        .expect("render visible path section");
    let content: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();

    let first = content.find("01. /first/bin").expect("first path entry");
    let second = content.find("02. /second/bin").expect("second path entry");
    let duplicate = content
        .find("03. /first/bin")
        .expect("duplicate path entry");
    assert!(first < second && second < duplicate);
    assert!(content.contains("PATH directories"));
}

#[test]
fn path_section_marks_duplicates_as_warning_and_missing_folders_as_error() {
    let mut app = App::with_path_entries(
        snapshots(),
        vec![
            PathBuf::from("/tmp"),
            PathBuf::from("/tmp"),
            PathBuf::from("/path/that/does/not/exist"),
        ],
    );
    app.handle_key(key(KeyCode::Char('p')));

    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    terminal
        .draw(|frame| render(frame, &app))
        .expect("render path statuses");
    let buffer = terminal.backend().buffer();

    let find_cell_style = |needle: &str| {
        for y in 0..buffer.area().height {
            let row: String = (0..buffer.area().width)
                .map(|x| buffer.cell((x, y)).expect("buffer cell").symbol())
                .collect();
            if let Some(x) = row.find(needle) {
                return buffer.cell((x as u16, y)).expect("path cell").style().fg;
            }
        }
        panic!("missing rendered path: {needle}");
    };

    assert_eq!(find_cell_style("01. /tmp"), Some(Color::Yellow));
    assert_eq!(find_cell_style("02. /tmp"), Some(Color::Yellow));
    assert_eq!(
        find_cell_style("03. /path/that/does/not/exist"),
        Some(Color::Red)
    );
}

#[test]
fn path_section_is_capped_at_half_height_and_scrollable() {
    let mut app = App::with_path_entries(snapshots(), numbered_path_entries());
    app.handle_key(key(KeyCode::Char('p')));

    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    terminal
        .draw(|frame| render(frame, &app))
        .expect("render initial path page");
    let buffer = terminal.backend().buffer();
    let path_title_row = row_containing(buffer, "PATH directories");
    let footer_row = row_containing(buffer, "↑/↓ j/k navigasi");
    assert_eq!(footer_row - path_title_row, 15);
    assert!(!buffer_text(buffer).contains("20. /path/20"));

    for _ in 0..20 {
        app.handle_key(key(KeyCode::Char(']')));
    }
    assert_eq!(app.path_scroll(), 20);
    app.handle_key(key(KeyCode::Char('[')));
    assert_eq!(app.path_scroll(), 19);
    app.handle_key(key(KeyCode::Char(']')));

    terminal
        .draw(|frame| render(frame, &app))
        .expect("render scrolled path page");
    assert!(buffer_text(terminal.backend().buffer()).contains("21. /path/21"));
}

fn buffer_text(buffer: &ratatui::buffer::Buffer) -> String {
    buffer.content().iter().map(|cell| cell.symbol()).collect()
}

fn row_containing(buffer: &ratatui::buffer::Buffer, needle: &str) -> u16 {
    for y in 0..buffer.area().height {
        let row: String = (0..buffer.area().width)
            .map(|x| buffer.cell((x, y)).expect("buffer cell").symbol())
            .collect();
        if row.contains(needle) {
            return y;
        }
    }
    panic!("missing rendered text: {needle}");
}
