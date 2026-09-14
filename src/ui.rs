use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, StatefulWidget, Table, TableState},
};

use crate::detect::{path_entries_from_environment, path_source_hints};
use crate::model::{
    ApplicationRecord, CellState, CellValue, RecordStatus, SourceSnapshot, SourceStatus,
    record_status_label, status_label,
};
use crate::registry::source_registry;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppAction {
    None,
    Refresh,
    Quit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionFocus {
    Sources,
    Inventory,
    Path,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppPage {
    Sources,
    Inventory,
}

pub struct App {
    sources: Vec<SourceSnapshot>,
    inventory: Vec<ApplicationRecord>,
    path_entries: Vec<PathBuf>,
    path_source_hints: Vec<Option<String>>,
    selectable_indices: Vec<usize>,
    selected_position: Option<usize>,
    inventory_selected_position: Option<usize>,
    inventory_filter: String,
    inventory_filter_active: bool,
    inventory_horizontal_scroll: u16,
    detail_visible: bool,
    path_visible: bool,
    path_scroll: usize,
    focused_section: SectionFocus,
    page: AppPage,
}

impl App {
    pub fn new(sources: Vec<SourceSnapshot>) -> Self {
        Self::with_path_entries_and_inventory(sources, Vec::new(), path_entries_from_environment())
    }

    pub fn with_path_entries(sources: Vec<SourceSnapshot>, path_entries: Vec<PathBuf>) -> Self {
        let path_source_hints = path_source_hints(&path_entries);
        Self::with_path_entries_and_inventory_and_hints(
            sources,
            Vec::new(),
            path_entries,
            path_source_hints,
        )
    }

    pub fn with_inventory(sources: Vec<SourceSnapshot>, inventory: Vec<ApplicationRecord>) -> Self {
        Self::with_path_entries_and_inventory(sources, inventory, path_entries_from_environment())
    }

    pub fn with_path_entries_and_inventory(
        sources: Vec<SourceSnapshot>,
        inventory: Vec<ApplicationRecord>,
        path_entries: Vec<PathBuf>,
    ) -> Self {
        let path_source_hints = path_source_hints(&path_entries);
        Self::with_path_entries_and_inventory_and_hints(
            sources,
            inventory,
            path_entries,
            path_source_hints,
        )
    }

    pub fn with_path_entries_and_hints(
        sources: Vec<SourceSnapshot>,
        path_entries: Vec<PathBuf>,
        path_source_hints: Vec<Option<String>>,
    ) -> Self {
        Self::with_path_entries_and_inventory_and_hints(
            sources,
            Vec::new(),
            path_entries,
            path_source_hints,
        )
    }

    pub fn with_path_entries_and_inventory_and_hints(
        sources: Vec<SourceSnapshot>,
        mut inventory: Vec<ApplicationRecord>,
        path_entries: Vec<PathBuf>,
        mut path_source_hints: Vec<Option<String>>,
    ) -> Self {
        path_source_hints.resize(path_entries.len(), None);
        path_source_hints.truncate(path_entries.len());
        let mut sources = sources;
        sources.sort_by(|left, right| {
            let left_disabled = matches!(left.status, SourceStatus::Disabled { .. });
            let right_disabled = matches!(right.status, SourceStatus::Disabled { .. });
            left_disabled
                .cmp(&right_disabled)
                .then_with(|| left.definition.name.cmp(right.definition.name))
        });
        sort_inventory(&mut inventory);
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
            inventory,
            path_entries,
            path_source_hints,
            selectable_indices,
            selected_position,
            inventory_selected_position: None,
            inventory_filter: String::new(),
            inventory_filter_active: false,
            inventory_horizontal_scroll: 0,
            detail_visible: false,
            path_visible: false,
            path_scroll: 0,
            focused_section: SectionFocus::Sources,
            page: AppPage::Sources,
        }
    }

    pub fn sources(&self) -> &[SourceSnapshot] {
        &self.sources
    }

    pub fn inventory(&self) -> &[ApplicationRecord] {
        &self.inventory
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_position
            .and_then(|position| self.selectable_indices.get(position).copied())
    }

    pub fn selected_inventory_index(&self) -> Option<usize> {
        self.inventory_selected_position
            .and_then(|position| self.filtered_inventory_indices().get(position).copied())
    }

    pub fn inventory_filter(&self) -> &str {
        &self.inventory_filter
    }

    pub fn inventory_filter_active(&self) -> bool {
        self.inventory_filter_active
    }

    pub fn inventory_horizontal_scroll(&self) -> u16 {
        self.inventory_horizontal_scroll
    }

    pub fn page(&self) -> AppPage {
        self.page
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

    pub fn replace_data(
        &mut self,
        sources: Vec<SourceSnapshot>,
        inventory: Vec<ApplicationRecord>,
    ) {
        let path_entries = self.path_entries.clone();
        let path_hints = self.path_source_hints.clone();
        let page = self.page;
        let focused_section = self.focused_section;
        let detail_visible = self.detail_visible;
        let inventory_filter = self.inventory_filter.clone();
        let inventory_horizontal_scroll = self.inventory_horizontal_scroll;
        let selected_record_key = self
            .selected_inventory_index()
            .and_then(|index| self.inventory.get(index))
            .map(|record| record.record_key.clone());
        *self = Self::with_path_entries_and_inventory_and_hints(
            sources,
            inventory,
            path_entries,
            path_hints,
        );
        self.page = page;
        self.focused_section = focused_section;
        self.detail_visible = detail_visible;
        self.inventory_filter = inventory_filter;
        self.inventory_horizontal_scroll = inventory_horizontal_scroll;
        self.inventory_selected_position = selected_record_key
            .and_then(|key| {
                self.filtered_inventory_indices()
                    .iter()
                    .position(|index| self.inventory[*index].record_key == key)
            })
            .or_else(|| (!self.filtered_inventory_indices().is_empty()).then_some(0));
        self.clamp_inventory_selection();
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> AppAction {
        if self.inventory_filter_active {
            return self.handle_filter_key(key);
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => AppAction::Quit,
            KeyCode::Char('r') | KeyCode::Char('R') => AppAction::Refresh,
            KeyCode::Char('i') | KeyCode::Char('I') => {
                self.page = AppPage::Inventory;
                self.focused_section = SectionFocus::Inventory;
                self.detail_visible = false;
                if self.inventory_selected_position.is_none()
                    && !self.filtered_inventory_indices().is_empty()
                {
                    self.inventory_selected_position = Some(0);
                }
                AppAction::None
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.page = AppPage::Sources;
                self.focused_section = SectionFocus::Sources;
                self.detail_visible = false;
                AppAction::None
            }
            KeyCode::Char('f') | KeyCode::Char('F') if self.page == AppPage::Inventory => {
                self.inventory_filter_active = true;
                AppAction::None
            }
            KeyCode::Tab => {
                self.focused_section = match self.focused_section {
                    SectionFocus::Sources => {
                        self.path_visible = true;
                        SectionFocus::Path
                    }
                    SectionFocus::Inventory => {
                        self.path_visible = true;
                        SectionFocus::Path
                    }
                    SectionFocus::Path => SectionFocus::Sources,
                };
                if self.focused_section == SectionFocus::Sources {
                    self.page = AppPage::Sources;
                }
                AppAction::None
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.path_visible = !self.path_visible;
                if self.path_visible {
                    self.focused_section = SectionFocus::Path;
                } else {
                    self.path_scroll = 0;
                    self.focused_section = SectionFocus::Sources;
                    self.page = AppPage::Sources;
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
            KeyCode::Enter
                if self.focused_section == SectionFocus::Sources
                    || self.focused_section == SectionFocus::Inventory =>
            {
                self.detail_visible = !self.detail_visible;
                AppAction::None
            }
            KeyCode::Down | KeyCode::Char('j') if self.focused_section == SectionFocus::Sources => {
                self.move_selection(1)
            }
            KeyCode::Up | KeyCode::Char('k') if self.focused_section == SectionFocus::Sources => {
                self.move_selection(-1)
            }
            KeyCode::Down | KeyCode::Char('j')
                if self.focused_section == SectionFocus::Inventory =>
            {
                self.move_inventory_selection(1)
            }
            KeyCode::Up | KeyCode::Char('k') if self.focused_section == SectionFocus::Inventory => {
                self.move_inventory_selection(-1)
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
            KeyCode::PageDown | KeyCode::Char(' ')
                if self.focused_section == SectionFocus::Inventory =>
            {
                self.move_inventory_selection(10)
            }
            KeyCode::PageUp if self.focused_section == SectionFocus::Inventory => {
                self.move_inventory_selection(-10)
            }
            KeyCode::Home if self.focused_section == SectionFocus::Inventory => {
                if self.filtered_inventory_indices().is_empty() {
                    AppAction::None
                } else {
                    self.inventory_selected_position = Some(0);
                    AppAction::None
                }
            }
            KeyCode::End if self.focused_section == SectionFocus::Inventory => {
                self.inventory_selected_position =
                    self.filtered_inventory_indices().len().checked_sub(1);
                AppAction::None
            }
            KeyCode::Left | KeyCode::Char('h')
                if self.page == AppPage::Inventory
                    && self.focused_section == SectionFocus::Inventory =>
            {
                self.inventory_horizontal_scroll =
                    self.inventory_horizontal_scroll.saturating_sub(8);
                AppAction::None
            }
            KeyCode::Right | KeyCode::Char('l')
                if self.page == AppPage::Inventory
                    && self.focused_section == SectionFocus::Inventory =>
            {
                self.inventory_horizontal_scroll =
                    self.inventory_horizontal_scroll.saturating_add(8);
                AppAction::None
            }
            _ => AppAction::None,
        }
    }

    fn handle_filter_key(&mut self, key: KeyEvent) -> AppAction {
        match key.code {
            KeyCode::Enter => {
                self.inventory_filter_active = false;
                AppAction::None
            }
            KeyCode::Esc => {
                self.inventory_filter.clear();
                self.inventory_filter_active = false;
                self.inventory_selected_position = None;
                AppAction::None
            }
            KeyCode::Backspace => {
                self.inventory_filter.pop();
                self.clamp_inventory_selection();
                AppAction::None
            }
            KeyCode::Char(character) => {
                self.inventory_filter.push(character);
                if self.inventory_selected_position.is_none()
                    && !self.filtered_inventory_indices().is_empty()
                {
                    self.inventory_selected_position = Some(0);
                }
                self.clamp_inventory_selection();
                AppAction::None
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

    fn move_inventory_selection(&mut self, amount: isize) -> AppAction {
        let indices = self.filtered_inventory_indices();
        let Some(current) = self.inventory_selected_position else {
            if !indices.is_empty() {
                self.inventory_selected_position = Some(0);
            }
            return AppAction::None;
        };
        let last = indices.len().saturating_sub(1) as isize;
        self.inventory_selected_position =
            Some((current as isize + amount).clamp(0, last) as usize);
        AppAction::None
    }

    fn filtered_inventory_indices(&self) -> Vec<usize> {
        let query = self.inventory_filter.to_lowercase();
        self.inventory
            .iter()
            .enumerate()
            .filter(|(_, record)| {
                query.is_empty()
                    || [
                        record.name.searchable(),
                        record.version.searchable(),
                        record.source.to_lowercase(),
                        record_status_label(&record.status).to_lowercase(),
                        record.identifier.searchable(),
                        record.location.searchable(),
                    ]
                    .iter()
                    .any(|value| value.contains(&query))
            })
            .map(|(index, _)| index)
            .collect()
    }

    fn clamp_inventory_selection(&mut self) {
        let max = self.filtered_inventory_indices().len().checked_sub(1);
        self.inventory_selected_position = self
            .inventory_selected_position
            .and_then(|position| max.map(|last| position.min(last)));
    }
}

pub fn render(frame: &mut Frame, app: &App) {
    let detail_height = if app.detail_visible() {
        if app.page == AppPage::Inventory { 7 } else { 5 }
    } else {
        3
    };
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

    match app.page {
        AppPage::Sources => render_sources(frame, app, areas[0], areas[1], areas[2]),
        AppPage::Inventory => render_inventory(frame, app, areas[0], areas[1], areas[2]),
    }

    if app.path_visible() {
        let rows = if app.path_entries().is_empty() {
            vec![Row::new([
                Cell::from(""),
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
                        Cell::from(format!("{:02}", index + 1)),
                        Cell::from(display_path(path)),
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
                Constraint::Length(4),
                Constraint::Min(30),
                Constraint::Length(28),
                Constraint::Length(26),
            ],
        )
        .header(
            Row::new([
                Cell::from("No."),
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
            if app.page == AppPage::Inventory {
                if app.inventory_filter_active {
                    "Ketik filter  Enter selesai  Esc batal  Backspace hapus"
                } else {
                    "↑/↓ j/k navigasi  PgUp/PgDn halaman  Enter detail  f filter  s Sources  Tab PATH  r refresh  q/Esc keluar"
                }
            } else {
                "↑/↓ j/k navigasi  PgUp/PgDn halaman  Enter detail  i Inventory  Tab PATH  p PATH  r refresh  q/Esc keluar"
            },
        ),
        areas[4],
    );
}

fn render_sources(
    frame: &mut Frame,
    app: &App,
    title_area: ratatui::layout::Rect,
    table_area: ratatui::layout::Rect,
    detail_area: ratatui::layout::Rect,
) {
    let title = Paragraph::new(Line::from(vec![Span::styled(
        "MangApp — Sources",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, title_area);

    let header = Row::new([
        Cell::from("No."),
        Cell::from("Status"),
        Cell::from("Sumber"),
        Cell::from("Command"),
        Cell::from("Kategori"),
        Cell::from("Keterangan"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));

    let rows = app.sources().iter().enumerate().map(|(index, snapshot)| {
        let status = status_label(&snapshot.status);
        let candidates = snapshot.definition.candidates.join(" / ");
        let note = status_note(&snapshot.status);
        Row::new([
            Cell::from(format!("{:02}", index + 1)),
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
            Constraint::Length(4),
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
    frame.render_stateful_widget(table, table_area, &mut table_state);

    let detail = app
        .selected_index()
        .and_then(|index| app.sources().get(index))
        .map(detail_line)
        .unwrap_or_else(|| {
            "Sumber disabled tetap ditampilkan sebagai dukungan yang tersedia di MangApp.".into()
        });
    frame.render_widget(
        Paragraph::new(detail).block(Block::default().borders(Borders::ALL).title(" Detail ")),
        detail_area,
    );
}

fn render_inventory(
    frame: &mut Frame,
    app: &App,
    title_area: ratatui::layout::Rect,
    table_area: ratatui::layout::Rect,
    detail_area: ratatui::layout::Rect,
) {
    let title_text = if app.inventory_filter().is_empty() {
        "MangApp — Application Inventory".to_owned()
    } else if app.inventory_filter_active() {
        format!(
            "MangApp — Application Inventory  |  Filter: {}_",
            app.inventory_filter()
        )
    } else {
        format!(
            "MangApp — Application Inventory  |  Filter: {}",
            app.inventory_filter()
        )
    };
    let title = Paragraph::new(Line::from(vec![Span::styled(
        title_text,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, title_area);

    let header = Row::new([
        Cell::from("No."),
        Cell::from("Aplikasi"),
        Cell::from("Versi"),
        Cell::from("Sumber"),
        Cell::from("Status"),
        Cell::from("Identifier"),
        Cell::from("Lokasi"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));

    let indices = app.filtered_inventory_indices();
    let rows = if indices.is_empty() {
        vec![Row::new([
            Cell::from(""),
            Cell::from(if app.inventory().is_empty() {
                "Belum ada aplikasi yang terdeteksi"
            } else {
                "Tidak ada aplikasi yang cocok dengan filter"
            }),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])]
    } else {
        indices
            .iter()
            .enumerate()
            .filter_map(|(position, index)| {
                let record = app.inventory().get(*index)?;
                Some(
                    Row::new([
                        Cell::from(format!("{:02}", position + 1)),
                        value_cell(&record.name),
                        value_cell(&record.version),
                        Cell::from(record.source.clone()),
                        Cell::from(record_status_label(&record.status))
                            .style(record_status_style(&record.status)),
                        value_cell(&record.identifier),
                        value_cell(&record.location),
                    ])
                    .style(Style::default()),
                )
            })
            .collect::<Vec<_>>()
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(22),
            Constraint::Length(16),
            Constraint::Length(14),
            Constraint::Length(12),
            Constraint::Length(24),
            Constraint::Min(18),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(section_border_style(
                app.focused_section() == SectionFocus::Inventory,
            ))
            .title(if app.focused_section() == SectionFocus::Inventory {
                " Inventory [FOCUS] "
            } else {
                " Inventory "
            }),
    )
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol("› ");
    let mut table_state = TableState::default();
    if !indices.is_empty() {
        table_state.select(app.inventory_selected_position);
    }
    render_inventory_table(
        frame,
        table,
        table_area,
        &mut table_state,
        app.inventory_horizontal_scroll(),
    );

    let detail = app
        .selected_inventory_index()
        .and_then(|index| app.inventory().get(index))
        .map(application_detail_line)
        .unwrap_or_else(|| "Pilih aplikasi untuk melihat detail.".into());
    frame.render_widget(
        Paragraph::new(detail).block(Block::default().borders(Borders::ALL).title(" Detail ")),
        detail_area,
    );
}

const INVENTORY_SCROLL_WIDTH: u16 = 120;

fn render_inventory_table(
    frame: &mut Frame,
    table: Table<'_>,
    area: ratatui::layout::Rect,
    state: &mut TableState,
    requested_offset: u16,
) {
    if area.width >= INVENTORY_SCROLL_WIDTH {
        frame.render_stateful_widget(table, area, state);
        return;
    }

    let mut buffer = Buffer::empty(ratatui::layout::Rect::new(
        0,
        0,
        INVENTORY_SCROLL_WIDTH,
        area.height,
    ));
    table.render(buffer.area, &mut buffer, state);
    let offset = requested_offset.min(INVENTORY_SCROLL_WIDTH.saturating_sub(area.width));
    let target = frame.buffer_mut();
    for y in 0..area.height {
        for x in 0..area.width {
            let Some(source) = buffer.cell((x.saturating_add(offset), y)) else {
                continue;
            };
            let Some(destination) = target.cell_mut((area.x.saturating_add(x), area.y + y)) else {
                continue;
            };
            *destination = source.clone();
        }
    }
}

fn value_cell(value: &CellValue) -> Cell<'static> {
    Cell::from(value.display().to_owned()).style(cell_style(value))
}

fn cell_style(value: &CellValue) -> Style {
    match value.state {
        CellState::Value(_) => Style::default(),
        CellState::Empty => Style::default().fg(Color::Yellow),
        CellState::Unavailable => Style::default().fg(Color::DarkGray),
        CellState::NotApplicable => Style::default().fg(Color::DarkGray),
        CellState::Error => Style::default().fg(Color::Red),
    }
}

fn record_status_style(status: &RecordStatus) -> Style {
    match status {
        RecordStatus::Available => Style::default().fg(Color::Green),
        RecordStatus::Empty => Style::default().fg(Color::Yellow),
        RecordStatus::Unavailable => Style::default().fg(Color::DarkGray),
        RecordStatus::Error => Style::default().fg(Color::Red),
    }
}

fn sort_inventory(inventory: &mut [ApplicationRecord]) {
    inventory.sort_by(|left, right| {
        compare_cells_ascending(&left.name, &right.name)
            .then_with(|| source_rank(&left.source).cmp(&source_rank(&right.source)))
            .then_with(|| compare_versions(&left.version, &right.version))
            .then_with(|| compare_cells_ascending(&left.identifier, &right.identifier))
    });
}

fn source_rank(source: &str) -> usize {
    source_registry()
        .iter()
        .position(|definition| definition.id == source)
        .unwrap_or(usize::MAX)
}

fn compare_cells_ascending(left: &CellValue, right: &CellValue) -> Ordering {
    match (is_valid_cell(left), is_valid_cell(right)) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => left
            .display()
            .to_lowercase()
            .cmp(&right.display().to_lowercase()),
    }
}

fn compare_versions(left: &CellValue, right: &CellValue) -> Ordering {
    match (is_valid_cell(left), is_valid_cell(right)) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => right
            .display()
            .to_lowercase()
            .cmp(&left.display().to_lowercase()),
    }
}

fn is_valid_cell(value: &CellValue) -> bool {
    matches!(value.state, CellState::Value(_))
}

fn application_detail_line(record: &ApplicationRecord) -> String {
    format!(
        "{} · {} · {}\nIdentifier: {}  |  Lokasi: {}\nTanggal Install: {}  |  Tanggal Update: {}\nNote: {}",
        record.name.display(),
        record.source,
        record_status_label(&record.status),
        record.identifier.display(),
        record.location.display(),
        record.installed_at.display(),
        record.updated_at.display(),
        record.note.display(),
    )
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
