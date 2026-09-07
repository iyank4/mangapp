use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};

use crate::detect::{path_entries_from_environment, path_source_hints};
use crate::model::{SourceSnapshot, SourceStatus, status_label};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppAction {
    None,
    Refresh,
    Quit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionFocus {
    Sources,
    Path,
}

pub struct App {
    sources: Vec<SourceSnapshot>,
    path_entries: Vec<PathBuf>,
    path_source_hints: Vec<Option<String>>,
    selectable_indices: Vec<usize>,
    selected_position: Option<usize>,
    detail_visible: bool,
    path_visible: bool,
    path_scroll: usize,
    focused_section: SectionFocus,
}

impl App {
    pub fn new(sources: Vec<SourceSnapshot>) -> Self {
        Self::with_path_entries(sources, path_entries_from_environment())
    }

    pub fn with_path_entries(sources: Vec<SourceSnapshot>, path_entries: Vec<PathBuf>) -> Self {
        let path_source_hints = path_source_hints(&path_entries);
        Self::with_path_entries_and_hints(sources, path_entries, path_source_hints)
    }

    pub fn with_path_entries_and_hints(
        sources: Vec<SourceSnapshot>,
        path_entries: Vec<PathBuf>,
        mut path_source_hints: Vec<Option<String>>,
    ) -> Self {
        path_source_hints.resize(path_entries.len(), None);
        path_source_hints.truncate(path_entries.len());
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
            path_source_hints,
            selectable_indices,
            selected_position,
            detail_visible: false,
            path_visible: false,
            path_scroll: 0,
            focused_section: SectionFocus::Sources,
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

    pub fn path_source_hints(&self) -> &[Option<String>] {
        &self.path_source_hints
    }

    pub fn path_visible(&self) -> bool {
        self.path_visible
    }

    pub fn path_scroll(&self) -> usize {
        self.path_scroll
    }

    pub fn focused_section(&self) -> SectionFocus {
        self.focused_section
    }

    pub fn replace_sources(&mut self, sources: Vec<SourceSnapshot>) {
        *self = Self::new(sources);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> AppAction {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => AppAction::Quit,
            KeyCode::Char('r') | KeyCode::Char('R') => AppAction::Refresh,
            KeyCode::Tab => {
                self.focused_section = match self.focused_section {
                    SectionFocus::Sources => {
                        self.path_visible = true;
                        SectionFocus::Path
                    }
                    SectionFocus::Path => SectionFocus::Sources,
                };
                AppAction::None
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.path_visible = !self.path_visible;
                if self.path_visible {
                    self.focused_section = SectionFocus::Path;
                } else {
                    self.path_scroll = 0;
                    self.focused_section = SectionFocus::Sources;
                }
                AppAction::None
            }
            KeyCode::Down | KeyCode::Char('j')
                if self.path_visible && self.focused_section == SectionFocus::Path =>
            {
                self.path_scroll = self
                    .path_scroll
                    .saturating_add(1)
                    .min(self.path_entries.len().saturating_sub(1));
                AppAction::None
            }
            KeyCode::Up | KeyCode::Char('k')
                if self.path_visible && self.focused_section == SectionFocus::Path =>
            {
                self.path_scroll = self.path_scroll.saturating_sub(1);
                AppAction::None
            }
            KeyCode::Enter if self.focused_section == SectionFocus::Sources => {
                self.detail_visible = !self.detail_visible;
                AppAction::None
            }
            KeyCode::Down | KeyCode::Char('j') if self.focused_section == SectionFocus::Sources => {
                self.move_selection(1)
            }
            KeyCode::Up | KeyCode::Char('k') if self.focused_section == SectionFocus::Sources => {
                self.move_selection(-1)
            }
            KeyCode::PageDown | KeyCode::Char(' ')
                if self.focused_section == SectionFocus::Sources =>
            {
                self.move_selection(10)
            }
            KeyCode::PageUp if self.focused_section == SectionFocus::Sources => {
                self.move_selection(-10)
            }
            KeyCode::Home if self.focused_section == SectionFocus::Sources => self
                .selectable_indices
                .first()
                .map_or(AppAction::None, |_| {
                    self.selected_position = Some(0);
                    AppAction::None
                }),
            KeyCode::End if self.focused_section == SectionFocus::Sources => self
                .selectable_indices
                .len()
                .checked_sub(1)
                .map_or(AppAction::None, |last| {
                    self.selected_position = Some(last);
                    AppAction::None
                }),
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
    let path_height = path_panel_height(
        frame.area().height,
        detail_height,
        app.path_entries().len(),
        app.path_visible(),
    );
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
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(section_border_style(
                app.focused_section() == SectionFocus::Sources,
            ))
            .title(if app.focused_section() == SectionFocus::Sources {
                " Sources [FOCUS] "
            } else {
                " Sources "
            }),
    )
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
        let rows = if app.path_entries().is_empty() {
            vec![Row::new([
                Cell::from("(PATH kosong)"),
                Cell::from(""),
                Cell::from(""),
            ])]
        } else {
            app.path_entries()
                .iter()
                .enumerate()
                .map(|(index, path)| {
                    let hint = app
                        .path_source_hints()
                        .get(index)
                        .and_then(Option::as_deref)
                        .unwrap_or_default();
                    Row::new([
                        Cell::from(format!("{:02}. {}", index + 1, display_path(path))),
                        Cell::from(hint),
                        Cell::from(path_note(path, app.path_entries()))
                            .style(path_note_style(path, app.path_entries())),
                    ])
                })
                .collect()
        };
        let mut path_state = TableState::default();
        if app.focused_section() == SectionFocus::Path && !app.path_entries().is_empty() {
            path_state.select(Some(
                app.path_scroll()
                    .min(app.path_entries().len().saturating_sub(1)),
            ));
        }
        let path_table = Table::new(
            rows,
            [
                Constraint::Min(30),
                Constraint::Length(28),
                Constraint::Length(26),
            ],
        )
        .header(
            Row::new([
                Cell::from("PATH"),
                Cell::from("SOURCE HINT"),
                Cell::from("NOTE / CATATAN"),
            ])
            .style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(section_border_style(
                    app.focused_section() == SectionFocus::Path,
                ))
                .title(if app.focused_section() == SectionFocus::Path {
                    " PATH directories [FOCUS] "
                } else {
                    " PATH directories "
                }),
        )
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("› ");
        frame.render_stateful_widget(path_table, areas[3], &mut path_state);
    } else {
        frame.render_widget(
            Paragraph::new("PATH directories: hidden (p untuk tampilkan)")
                .style(Style::default().fg(Color::DarkGray)),
            areas[3],
        );
    }

    frame.render_widget(
        Paragraph::new(
            "↑/↓ j/k navigasi  PgUp/PgDn halaman  Enter detail  Tab fokus  p PATH  r refresh  q/Esc keluar",
        ),
        areas[4],
    );
}

fn section_border_style(active: bool) -> Style {
    if active {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    }
}

fn path_panel_height(
    screen_height: u16,
    detail_height: u16,
    path_count: usize,
    visible: bool,
) -> u16 {
    if !visible {
        return 1;
    }

    let fixed_height = 2 + detail_height + 1 + 1;
    let available = screen_height.saturating_sub(fixed_height);
    let desired = path_count.saturating_add(3).max(4) as u16;
    let table_minimum = available.saturating_sub(5);
    desired.min(screen_height / 2).min(table_minimum).max(1)
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

fn path_note(path: &Path, paths: &[PathBuf]) -> &'static str {
    if !path.is_dir() {
        "Error: folder tidak ada"
    } else if path_is_duplicate(path, paths) {
        "Warning: path duplikat"
    } else {
        ""
    }
}

fn path_note_style(path: &Path, paths: &[PathBuf]) -> Style {
    if !path.is_dir() {
        Style::default().fg(Color::Red)
    } else if path_is_duplicate(path, paths) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
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
