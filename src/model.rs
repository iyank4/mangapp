use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub candidates: &'static [&'static str],
    pub category: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceStatus {
    Available { executable: PathBuf },
    Disabled { candidates: Vec<String> },
    Error { message: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceSnapshot {
    pub definition: &'static SourceDefinition,
    pub status: SourceStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CellState {
    Value(String),
    Empty,
    Unavailable,
    NotApplicable,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellValue {
    pub state: CellState,
}

impl CellValue {
    pub fn value(value: impl Into<String>) -> Self {
        Self {
            state: CellState::Value(value.into()),
        }
    }

    pub fn empty() -> Self {
        Self {
            state: CellState::Empty,
        }
    }

    pub fn unavailable() -> Self {
        Self {
            state: CellState::Unavailable,
        }
    }

    pub fn not_applicable() -> Self {
        Self {
            state: CellState::NotApplicable,
        }
    }

    pub fn error() -> Self {
        Self {
            state: CellState::Error,
        }
    }

    pub fn display(&self) -> &str {
        match &self.state {
            CellState::Value(value) => value,
            CellState::Empty => "EMPTY",
            CellState::Unavailable => "UNAVAILABLE",
            CellState::NotApplicable => "N/A",
            CellState::Error => "ERROR",
        }
    }

    pub fn searchable(&self) -> String {
        self.display().to_lowercase()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordStatus {
    Available,
    Empty,
    Unavailable,
    Error,
}

pub fn record_status_label(status: &RecordStatus) -> &'static str {
    match status {
        RecordStatus::Available => "AVAILABLE",
        RecordStatus::Empty => "EMPTY",
        RecordStatus::Unavailable => "UNAVAILABLE",
        RecordStatus::Error => "ERROR",
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplicationRecord {
    pub record_key: String,
    pub source: String,
    pub name: CellValue,
    pub version: CellValue,
    pub identifier: CellValue,
    pub status: RecordStatus,
    pub location: CellValue,
    pub installed_at: CellValue,
    pub updated_at: CellValue,
    pub note: CellValue,
}

impl ApplicationRecord {
    pub fn new(
        source: impl Into<String>,
        identifier: CellValue,
        name: CellValue,
        version: CellValue,
        status: RecordStatus,
    ) -> Self {
        let source = source.into();
        let key_identifier = identifier.display().to_lowercase();
        Self {
            record_key: format!("{source}:{key_identifier}"),
            source,
            name,
            version,
            identifier,
            status,
            location: CellValue::not_applicable(),
            installed_at: CellValue::not_applicable(),
            updated_at: CellValue::not_applicable(),
            note: CellValue::empty(),
        }
    }
}

pub fn status_label(status: &SourceStatus) -> &'static str {
    match status {
        SourceStatus::Available { .. } => "AVAILABLE",
        SourceStatus::Disabled { .. } => "DISABLED",
        SourceStatus::Error { .. } => "ERROR",
    }
}
