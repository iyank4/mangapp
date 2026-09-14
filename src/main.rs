use std::{
    error::Error,
    io,
    sync::mpsc::{self, Receiver},
    time::Duration,
};

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use mangap::{
    detect::{PathResolver, detect_source, path_entries_from_environment},
    inventory::{collect_inventory, upgrade_inventory_interactive},
    model::{ApplicationRecord, CellState, CellValue, RecordStatus, SourceSnapshot, SourceStatus},
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

enum InventoryMessage {
    Complete(Vec<ApplicationRecord>),
}

enum SourceCheckMessage {
    Started {
        current: usize,
        total: usize,
        source_name: String,
    },
    Complete {
        source_id: String,
        applications: CellValue,
        updates: CellValue,
    },
    Finished,
}

fn collect_inventory_async(source: SourceSnapshot) -> Receiver<InventoryMessage> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let inventory = collect_inventory(std::slice::from_ref(&source));
        let _ = sender.send(InventoryMessage::Complete(inventory));
    });
    receiver
}

fn check_sources_async(sources: Vec<SourceSnapshot>) -> Receiver<SourceCheckMessage> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let total = sources.len();
        for (index, source) in sources.into_iter().enumerate() {
            let _ = sender.send(SourceCheckMessage::Started {
                current: index + 1,
                total,
                source_name: source.definition.name.to_owned(),
            });
            let records = collect_inventory(std::slice::from_ref(&source));
            let (applications, updates) = summarize_source_inventory(&source, &records);
            let _ = sender.send(SourceCheckMessage::Complete {
                source_id: source.definition.id.to_owned(),
                applications,
                updates,
            });
        }
        let _ = sender.send(SourceCheckMessage::Finished);
    });
    receiver
}

fn summarize_source_inventory(
    source: &SourceSnapshot,
    records: &[ApplicationRecord],
) -> (CellValue, CellValue) {
    match &source.status {
        SourceStatus::Disabled { .. } => (CellValue::not_applicable(), CellValue::not_applicable()),
        SourceStatus::Error { .. } => (CellValue::error(), CellValue::error()),
        SourceStatus::Available { .. } => {
            if records
                .iter()
                .any(|record| record.status == RecordStatus::Error)
            {
                return (CellValue::error(), CellValue::error());
            }
            let applications = records.len();
            let updates = records
                .iter()
                .filter(|record| {
                    matches!(&record.available_version.state, CellState::Value(value) if !value.is_empty())
                })
                .count();
            (
                CellValue::value(applications.to_string()),
                CellValue::value(updates.to_string()),
            )
        }
    }
}

fn selected_inventory_source(
    app: &App,
    sources: &[mangap::model::SourceSnapshot],
) -> Option<mangap::model::SourceSnapshot> {
    let source_id = app.inventory_source()?;
    sources
        .iter()
        .find(|snapshot| snapshot.definition.id == source_id)
        .cloned()
}

fn run_blocking_upgrade(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    source: &mangap::model::SourceSnapshot,
    records: &[mangap::model::ApplicationRecord],
) -> Result<(), Box<dyn Error>> {
    terminal.show_cursor()?;
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    println!(
        "MangApp — Upgrade aplikasi dari source: {}",
        source.definition.name
    );
    println!("Console upgrade aktif. Input dari user diteruskan ke command.");
    let result = upgrade_inventory_interactive(source, records);
    match &result {
        Ok(()) => println!("Upgrade selesai."),
        Err(error) => println!("Upgrade selesai dengan error: {error}"),
    }
    println!("Tekan Enter untuk kembali ke MangApp...");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal.clear()?;
    terminal.hide_cursor()?;
    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let _guard = TerminalGuard;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut sources = collect_sources();
    let mut app = App::with_path_entries_and_inventory(
        sources.clone(),
        Vec::new(),
        path_entries_from_environment(),
    );
    let mut inventory_receiver: Option<Receiver<InventoryMessage>> = None;
    let mut source_check_receiver: Option<Receiver<SourceCheckMessage>> = None;

    loop {
        let mut completed_inventory = None;
        if let Some(receiver) = inventory_receiver.as_ref()
            && let Ok(InventoryMessage::Complete(inventory)) = receiver.try_recv()
        {
            completed_inventory = Some(inventory);
        }
        if let Some(inventory) = completed_inventory {
            app.replace_data(sources.clone(), inventory);
            inventory_receiver = None;
        }
        let mut source_check_finished = false;
        if let Some(receiver) = source_check_receiver.as_ref() {
            match receiver.try_recv() {
                Ok(SourceCheckMessage::Started {
                    current,
                    total,
                    source_name,
                }) => app.source_check_started(current, total, source_name),
                Ok(SourceCheckMessage::Complete {
                    source_id,
                    applications,
                    updates,
                }) => app.apply_source_check_result(&source_id, applications, updates),
                Ok(SourceCheckMessage::Finished) => {
                    app.finish_source_check();
                    source_check_finished = true;
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    app.finish_source_check();
                    source_check_finished = true;
                }
            }
        }
        if source_check_finished {
            source_check_receiver = None;
        }
        terminal.draw(|frame| render(frame, &app))?;
        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        let previous_page = app.page();
        match app.handle_key(key) {
            AppAction::None => {}
            AppAction::Refresh => {
                if app.page() == mangap::ui::AppPage::Inventory {
                    if let Some(source_id) = app.inventory_source().map(str::to_owned) {
                        let resolver = PathResolver::from_environment();
                        if let Some(index) = sources
                            .iter()
                            .position(|snapshot| snapshot.definition.id == source_id)
                        {
                            sources[index] = detect_source(sources[index].definition, &resolver);
                            app.replace_data(sources.clone(), Vec::new());
                            app.begin_inventory_loading();
                            inventory_receiver = selected_inventory_source(&app, &sources)
                                .map(collect_inventory_async);
                        }
                    }
                } else {
                    source_check_receiver = None;
                    sources = collect_sources();
                    app.replace_data(sources.clone(), Vec::new());
                    app.reset_source_check();
                    inventory_receiver = None;
                }
            }
            AppAction::CheckSources => {
                inventory_receiver = None;
                let sources_to_check = app
                    .sources()
                    .iter()
                    .filter(|snapshot| matches!(snapshot.status, SourceStatus::Available { .. }))
                    .cloned()
                    .collect::<Vec<_>>();
                if sources_to_check.is_empty() {
                    app.reset_source_check();
                } else {
                    app.begin_source_check(sources_to_check.len());
                    source_check_receiver = Some(check_sources_async(sources_to_check));
                }
            }
            AppAction::Upgrade => {
                if let Some(source) = selected_inventory_source(&app, &sources) {
                    app.begin_inventory_loading();
                    run_blocking_upgrade(&mut terminal, &source, app.inventory())?;
                    inventory_receiver = Some(collect_inventory_async(source));
                }
            }
            AppAction::Quit => break,
        }
        if previous_page == mangap::ui::AppPage::Sources
            && app.page() == mangap::ui::AppPage::Inventory
        {
            app.begin_inventory_loading();
            inventory_receiver =
                selected_inventory_source(&app, &sources).map(collect_inventory_async);
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
