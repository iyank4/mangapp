use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};

use crate::detect::path_entries_from_environment;
use crate::model::{SourceSnapshot, SourceStatus, status_label};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppAction {
    None,
    Refresh,
    Quit,
}

pub struct App {
    sources: Vec<SourceSnapshot>,
    path_entries: Vec<PathBuf>,
    selectable_indices: Vec<usize>,
    selected_position: Option<usize>,
    detail_visible: bool,
    path_visible: bool,
}

impl App {
    pub fn new(sources: Vec<SourceSnapshot>) -> Self {
        Self::with_path_entries(sources, path_entries_from_environment())
    }

    pub fn with_path_entries(sources: Vec<SourceSnapshot>, path_entries: Vec<PathBuf>) -> Self {
        let selectable_indices = sources
            .iter()
            .enumerate()
            .filter_map(|(index, snapshot)| match snapshot.status {
                SourceStatus::Disabled { .. } => None,
                SourceStatus::Available { .. } | SourceStatus::Error { .. } => Some(index),
            })
            .collect::<Vec<_>>();
        let selected_position = (!selectable_indices.is_empty()).then_some(0);

        Self {
            sources,
            path_entries,
            selectable_indices,
            selected_position,
            detail_visible: false,
            path_visible: false,
        }
    }

    pub fn sources(&self) -> &[SourceSnapshot] {
        &self.sources
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_position
            .and_then(|position| self.selectable_indices.get(position).copied())
    }

    pub fn detail_visible(&self) -> bool {
        self.detail_visible
    }

    pub fn path_entries(&self) -> &[PathBuf] {
        &self.path_entries
    }

    pub fn path_visible(&self) -> bool {
        self.path_visible
    }

    pub fn replace_sources(&mut self, sources: Vec<SourceSnapshot>) {
        *self = Self::new(sources);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> AppAction {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => AppAction::Quit,
            KeyCode::Char('r') | KeyCode::Char('R') => AppAction::Refresh,
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.path_visible = !self.path_visible;
                AppAction::None
            }
            KeyCode::Enter => {
                self.detail_visible = !self.detail_visible;
                AppAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::PageDown | KeyCode::Char(' ') => self.move_selection(10),
            KeyCode::PageUp => self.move_selection(-10),
            KeyCode::Home => self
                .selectable_indices
                .first()
                .map_or(AppAction::None, |_| {
                    self.selected_position = Some(0);
                    AppAction::None
                }),
            KeyCode::End => {
                self.selectable_indices
                    .len()
                    .checked_sub(1)
                    .map_or(AppAction::None, |last| {
                        self.selected_position = Some(last);
                        AppAction::None
                    })
            }
            _ => AppAction::None,
        }
    }

    fn move_selection(&mut self, amount: isize) -> AppAction {
        let Some(current) = self.selected_position else {
            return AppAction::None;
        };
        let last = self.selectable_indices.len().saturating_sub(1) as isize;
        self.selected_position = Some((current as isize + amount).clamp(0, last) as usize);
        AppAction::None
    }
}

pub fn render(frame: &mut Frame, app: &App) {
    let detail_height = if app.detail_visible() { 5 } else { 3 };
    let path_height = if app.path_visible() {
        let fixed_height = 2 + detail_height + 1 + 1;
        let available = frame.area().height.saturating_sub(fixed_height);
        let desired = app.path_entries().len().saturating_add(2) as u16;
        desired.min(available.saturating_sub(5)).max(1)
    } else {
        1
    };
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(5),
            Constraint::Length(detail_height),
            Constraint::Length(path_height),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let title = Paragraph::new(Line::from(vec![Span::styled(
        "manapp — Sources",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, areas[0]);

    let header = Row::new([
        Cell::from("Status"),
        Cell::from("Sumber"),
        Cell::from("Command"),
        Cell::from("Kategori"),
        Cell::from("Keterangan"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));

    let rows = app.sources().iter().map(|snapshot| {
        let status = status_label(&snapshot.status);
        let candidates = snapshot.definition.candidates.join(" / ");
        let note = status_note(&snapshot.status);
        Row::new([
            Cell::from(status),
            Cell::from(snapshot.definition.name),
            Cell::from(candidates),
            Cell::from(snapshot.definition.category),
            Cell::from(note),
        ])
        .style(status_style(&snapshot.status))
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(18),
            Constraint::Length(24),
            Constraint::Length(28),
            Constraint::Min(20),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Sources "))
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol("› ");
    let mut table_state = TableState::default();
    table_state.select(app.selected_index());
    frame.render_stateful_widget(table, areas[1], &mut table_state);

    let detail = app
        .selected_index()
        .and_then(|index| app.sources().get(index))
        .map(detail_line)
        .unwrap_or_else(|| {
            "Sumber disabled tetap ditampilkan sebagai dukungan yang tersedia di manapp.".into()
        });
    frame.render_widget(
        Paragraph::new(detail).block(Block::default().borders(Borders::ALL).title(" Detail ")),
        areas[2],
    );

    if app.path_visible() {
        let lines = if app.path_entries().is_empty() {
            vec![Line::from("(PATH kosong)")]
        } else {
            app.path_entries()
                .iter()
                .enumerate()
                .map(|(index, path)| {
                    let annotation = path_annotation(path, app.path_entries());
                    Line::from(Span::styled(
                        format!("{:02}. {}{}", index + 1, display_path(path), annotation),
                        path_style(path, app.path_entries()),
                    ))
                })
                .collect()
        };
        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" PATH directories "),
            ),
            areas[3],
        );
    } else {
        frame.render_widget(
            Paragraph::new("PATH directories: hidden (p untuk tampilkan)")
                .style(Style::default().fg(Color::DarkGray)),
            areas[3],
        );
    }

    frame.render_widget(
        Paragraph::new(
            "↑/↓ j/k navigasi  PgUp/PgDn halaman  Enter detail  r refresh  q/Esc keluar",
        ),
        areas[4],
    );
}

fn display_path(path: &Path) -> String {
    if path.as_os_str().is_empty() {
        "(current directory)".into()
    } else {
        path.to_string_lossy().into_owned()
    }
}

fn path_is_duplicate(path: &Path, paths: &[PathBuf]) -> bool {
    paths
        .iter()
        .filter(|candidate| candidate.as_path() == path)
        .count()
        > 1
}

fn path_style(path: &Path, paths: &[PathBuf]) -> Style {
    if !path.is_dir() {
        Style::default().fg(Color::Red)
    } else if path_is_duplicate(path, paths) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    }
}

fn path_annotation(path: &Path, paths: &[PathBuf]) -> &'static str {
    if !path.is_dir() {
        " [ERROR: folder tidak ada]"
    } else if path_is_duplicate(path, paths) {
        " [WARNING: duplikat]"
    } else {
        ""
    }
}

fn status_style(status: &SourceStatus) -> Style {
    match status {
        SourceStatus::Available { .. } => Style::default().fg(Color::Green),
        SourceStatus::Disabled { .. } => Style::default().fg(Color::DarkGray),
        SourceStatus::Error { .. } => Style::default().fg(Color::Yellow),
    }
}

fn status_note(status: &SourceStatus) -> String {
    match status {
        SourceStatus::Available { executable } => format!("detected: {}", executable.display()),
        SourceStatus::Disabled { candidates } => {
            format!("{} tidak ditemukan", candidates.join(" / "))
        }
        SourceStatus::Error { message } => message.clone(),
    }
}

fn detail_line(snapshot: &SourceSnapshot) -> String {
    match &snapshot.status {
        SourceStatus::Available { executable } => format!(
            "{} · {} · executable: {}",
            snapshot.definition.name,
            status_label(&snapshot.status),
            executable.display()
        ),
        SourceStatus::Disabled { candidates } => format!(
            "{} · DISABLED · command dicari: {}",
            snapshot.definition.name,
            candidates.join(" / ")
        ),
        SourceStatus::Error { message } => {
            format!("{} · ERROR · {}", snapshot.definition.name, message)
        }
    }
}
