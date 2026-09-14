use std::{error::Error, io, time::Duration};

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use mangap::{
    detect::{PathResolver, detect_source, path_entries_from_environment},
    inventory::collect_inventory,
    registry::source_registry,
    ui::{App, AppAction, render},
};
use ratatui::{Terminal, backend::CrosstermBackend};

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen);
    }
}

fn collect_sources() -> Vec<mangap::model::SourceSnapshot> {
    let resolver = PathResolver::from_environment();
    source_registry()
        .iter()
        .map(|definition| detect_source(definition, &resolver))
        .collect()
}

fn collect_data() -> (
    Vec<mangap::model::SourceSnapshot>,
    Vec<mangap::model::ApplicationRecord>,
) {
    let sources = collect_sources();
    let inventory = collect_inventory(&sources);
    (sources, inventory)
}

fn run() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let _guard = TerminalGuard;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let (sources, inventory) = collect_data();
    let mut app =
        App::with_path_entries_and_inventory(sources, inventory, path_entries_from_environment());

    loop {
        terminal.draw(|frame| render(frame, &app))?;
        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        match app.handle_key(key) {
            AppAction::None => {}
            AppAction::Refresh => {
                let (sources, inventory) = collect_data();
                app.replace_data(sources, inventory);
            }
            AppAction::Quit => break,
        }
    }

    terminal.show_cursor()?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("mangap: {error}");
        std::process::exit(1);
    }
}
