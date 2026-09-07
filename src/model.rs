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

pub fn status_label(status: &SourceStatus) -> &'static str {
    match status {
        SourceStatus::Available { .. } => "AVAILABLE",
        SourceStatus::Disabled { .. } => "DISABLED",
        SourceStatus::Error { .. } => "ERROR",
    }
}
