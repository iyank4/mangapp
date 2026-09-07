use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use manapp::model::{SourceSnapshot, SourceStatus};
use manapp::registry::source_registry;
use manapp::ui::{App, AppAction, render};
use ratatui::{Terminal, backend::TestBackend};

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
