use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use manapp::model::{SourceSnapshot, SourceStatus};
use manapp::registry::source_registry;
use manapp::ui::{App, AppAction, SectionFocus, render};
use ratatui::{
    Terminal,
    backend::TestBackend,
    style::{Color, Modifier},
};

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

fn selected_source_name(app: &App) -> Option<&'static str> {
    app.selected_index()
        .map(|index| app.sources()[index].definition.name)
}

#[test]
fn app_navigates_enabled_rows_and_handles_actions() {
    let mut app = App::new(snapshots());
    assert_eq!(app.selected_index(), Some(0));

    app.handle_key(key(KeyCode::Down));
    assert_eq!(selected_source_name(&app), Some("Homebrew"));
    app.handle_key(key(KeyCode::Up));
    assert_eq!(selected_source_name(&app), Some("Cargo"));
    app.handle_key(key(KeyCode::PageDown));
    assert_eq!(selected_source_name(&app), Some("Yarn"));
    app.handle_key(key(KeyCode::PageUp));
    assert_eq!(selected_source_name(&app), Some("Cargo"));

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
fn app_orders_enabled_sources_before_disabled_sources() {
    let app = App::new(snapshots());
    let mut disabled_seen = false;

    for snapshot in app.sources() {
        let is_disabled = matches!(snapshot.status, SourceStatus::Disabled { .. });
        if is_disabled {
            disabled_seen = true;
        } else {
            assert!(
                !disabled_seen,
                "enabled source appeared after disabled source"
            );
        }
    }

    let names = app
        .sources()
        .iter()
        .map(|snapshot| snapshot.definition.name)
        .collect::<Vec<_>>();
    assert_eq!(&names[..3], &["Cargo", "Homebrew", "Yarn"]);
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
    assert!(content.contains("No."));
    assert!(content.contains("01"));
    assert!(content.contains("17"));
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
    assert_eq!(selected_source_name(&app), Some("Yarn"));
    app.handle_key(key(KeyCode::Home));
    assert_eq!(selected_source_name(&app), Some("Cargo"));

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
fn path_section_renders_source_hint_column_and_leaves_unknown_hint_blank() {
    let mut app = App::with_path_entries_and_hints(
        snapshots(),
        vec![PathBuf::from("/known/bin"), PathBuf::from("/unknown/bin")],
        vec![Some("~/.profile".into()), None],
    );
    app.handle_key(key(KeyCode::Tab));

    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    terminal
        .draw(|frame| render(frame, &app))
        .expect("render path source hints");
    let buffer = terminal.backend().buffer();
    assert!(buffer_text(buffer).contains("SOURCE HINT"));
    assert!(buffer_text(buffer).contains("NOTE / CATATAN"));
    assert!(buffer_text(buffer).contains("~/.profile"));
    let path_header_row = row_containing(buffer, "SOURCE HINT");
    let path_header_text: String = (0..buffer.area().width)
        .map(|x| {
            buffer
                .cell((x, path_header_row))
                .expect("buffer cell")
                .symbol()
        })
        .collect();
    assert!(path_header_text.contains("No."));

    let unknown_row = row_containing(buffer, "/unknown/bin");
    let unknown_text: String = (0..buffer.area().width)
        .map(|x| buffer.cell((x, unknown_row)).expect("buffer cell").symbol())
        .collect();
    assert!(!unknown_text.contains("~/.profile"));
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

    let first = content.find("/first/bin").expect("first path entry");
    let second = content.find("/second/bin").expect("second path entry");
    let duplicate = content.rfind("/first/bin").expect("duplicate path entry");
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

    assert_eq!(
        find_cell_style("Warning: path duplikat"),
        Some(Color::Yellow)
    );
    assert_eq!(
        find_cell_style("Warning: path duplikat"),
        Some(Color::Yellow)
    );
    assert_eq!(find_cell_style("Error: folder tidak ada"), Some(Color::Red));
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
    assert!(!buffer_text(buffer).contains("/path/20"));

    for _ in 0..20 {
        app.handle_key(key(KeyCode::Down));
    }
    assert_eq!(app.path_scroll(), 20);
    app.handle_key(key(KeyCode::Char('k')));
    assert_eq!(app.path_scroll(), 19);
    app.handle_key(key(KeyCode::Char('j')));

    terminal
        .draw(|frame| render(frame, &app))
        .expect("render scrolled path page");
    assert!(buffer_text(terminal.backend().buffer()).contains("/path/21"));
}

#[test]
fn tab_switches_focus_between_sources_and_path() {
    let mut app = App::with_path_entries(snapshots(), numbered_path_entries());

    assert_eq!(app.focused_section(), SectionFocus::Sources);
    assert!(!app.path_visible());

    app.handle_key(key(KeyCode::Tab));
    assert_eq!(app.focused_section(), SectionFocus::Path);
    assert!(app.path_visible());

    app.handle_key(key(KeyCode::Tab));
    assert_eq!(app.focused_section(), SectionFocus::Sources);
}

#[test]
fn section_focus_routes_section_specific_controls() {
    let mut app = App::with_path_entries(snapshots(), numbered_path_entries());

    app.handle_key(key(KeyCode::Down));
    assert_eq!(app.path_scroll(), 0);
    app.handle_key(key(KeyCode::Enter));
    assert!(app.detail_visible());

    app.handle_key(key(KeyCode::Tab));
    app.handle_key(key(KeyCode::Char('j')));
    assert_eq!(app.path_scroll(), 1);
    app.handle_key(key(KeyCode::Up));
    assert_eq!(app.path_scroll(), 0);
    app.handle_key(key(KeyCode::Char('k')));
    assert_eq!(app.path_scroll(), 0);
    app.handle_key(key(KeyCode::Enter));
    assert!(app.detail_visible());
}

#[test]
fn active_section_is_visually_marked() {
    let mut app = App::with_path_entries(snapshots(), numbered_path_entries());
    app.handle_key(key(KeyCode::Tab));

    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    terminal
        .draw(|frame| render(frame, &app))
        .expect("render focused path section");
    let content = buffer_text(terminal.backend().buffer());

    assert!(content.contains("PATH directories [FOCUS]"));
    assert!(!content.contains("Sources [FOCUS]"));
}

#[test]
fn path_section_highlights_the_focused_row() {
    let mut app = App::with_path_entries(snapshots(), numbered_path_entries());
    app.handle_key(key(KeyCode::Tab));

    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    terminal
        .draw(|frame| render(frame, &app))
        .expect("render initial focused path row");
    let first_buffer = terminal.backend().buffer();
    assert!(row_has_modifier(
        first_buffer,
        "/path/01",
        Modifier::REVERSED
    ));
    assert!(!row_has_modifier(
        first_buffer,
        "/path/02",
        Modifier::REVERSED
    ));

    app.handle_key(key(KeyCode::Down));
    terminal
        .draw(|frame| render(frame, &app))
        .expect("render next focused path row");
    let second_buffer = terminal.backend().buffer();
    assert!(!row_has_modifier(
        second_buffer,
        "/path/01",
        Modifier::REVERSED
    ));
    assert!(row_has_modifier(
        second_buffer,
        "/path/02",
        Modifier::REVERSED
    ));
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

fn row_has_modifier(buffer: &ratatui::buffer::Buffer, needle: &str, modifier: Modifier) -> bool {
    let row = row_containing(buffer, needle);
    let row_text: String = (0..buffer.area().width)
        .map(|x| buffer.cell((x, row)).expect("buffer cell").symbol())
        .collect();
    let x = row_text.find(needle).expect("path row start") as u16;
    buffer
        .cell((x, row))
        .expect("path cell")
        .style()
        .has_modifier(modifier)
}
