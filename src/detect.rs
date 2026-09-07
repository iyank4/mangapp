use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use crate::model::{SourceDefinition, SourceSnapshot, SourceStatus};

pub trait CommandResolver {
    fn resolve(&self, candidate: &str) -> Result<Option<PathBuf>, String>;
}

#[derive(Clone, Debug)]
pub struct PathResolver {
    path: OsString,
}

impl PathResolver {
    pub fn new(path: impl Into<OsString>) -> Self {
        Self { path: path.into() }
    }

    pub fn from_environment() -> Self {
        Self::new(std::env::var_os("PATH").unwrap_or_default())
    }
}

pub fn path_entries(path: &OsStr) -> Vec<PathBuf> {
    std::env::split_paths(path).collect()
}

pub fn path_entries_from_environment() -> Vec<PathBuf> {
    path_entries(&std::env::var_os("PATH").unwrap_or_default())
}

pub fn path_source_hints(paths: &[PathBuf]) -> Vec<Option<String>> {
    let mut sources = Vec::new();
    let system_paths = Path::new("/etc/paths");
    if let Some(entries) = read_path_file(system_paths) {
        sources.push((system_paths.display().to_string(), entries));
    }

    if let Ok(entries) = std::fs::read_dir("/etc/paths.d") {
        let mut files = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .collect::<Vec<_>>();
        files.sort();
        for file in files {
            if let Some(entries) = read_path_file(&file) {
                sources.push((file.display().to_string(), entries));
            }
        }
    }

    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        for filename in [".profile", ".zprofile", ".zshrc", ".zlogin"] {
            let file = home.join(filename);
            let Ok(contents) = std::fs::read_to_string(&file) else {
                continue;
            };
            let entries = shell_path_literals(&contents, &home);
            if !entries.is_empty() {
                sources.push((format!("~/{filename}"), entries));
            }
        }
    }

    paths
        .iter()
        .map(|path| {
            sources.iter().find_map(|(source, entries)| {
                entries
                    .iter()
                    .any(|candidate| candidate == path)
                    .then(|| source.clone())
            })
        })
        .collect()
}

fn read_path_file(path: &Path) -> Option<Vec<PathBuf>> {
    let contents = std::fs::read_to_string(path).ok()?;
    Some(
        contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(PathBuf::from)
            .collect(),
    )
}

fn shell_path_literals(contents: &str, home: &Path) -> Vec<PathBuf> {
    let home = home.to_string_lossy();
    contents
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || !line.contains("PATH") {
                return None;
            }
            let expanded = line
                .replace("${HOME}", home.as_ref())
                .replace("$HOME", home.as_ref());
            Some(
                expanded
                    .split(|character: char| {
                        matches!(
                            character,
                            ':' | '"' | '\'' | '=' | '`' | ' ' | '(' | ')' | ';'
                        )
                    })
                    .filter(|token| token.starts_with('/') && token.len() > 1)
                    .map(PathBuf::from)
                    .collect::<Vec<_>>(),
            )
        })
        .flatten()
        .collect()
}

impl CommandResolver for PathResolver {
    fn resolve(&self, candidate: &str) -> Result<Option<PathBuf>, String> {
        if candidate.is_empty() {
            return Err("nama command kosong".into());
        }

        for directory in std::env::split_paths(&self.path) {
            let directory = if directory.as_os_str().is_empty() {
                Path::new(".")
            } else {
                directory.as_path()
            };
            let path = directory.join(candidate);
            if is_executable(&path)? {
                return Ok(Some(path));
            }
        }

        Ok(None)
    }
}

#[cfg(unix)]
fn is_executable(path: &Path) -> Result<bool, String> {
    use std::os::unix::fs::PermissionsExt;

    match std::fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_file() && metadata.permissions().mode() & 0o111 != 0),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!("gagal memeriksa {}: {error}", path.display())),
    }
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> Result<bool, String> {
    match std::fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_file()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!("gagal memeriksa {}: {error}", path.display())),
    }
}

pub fn detect_source(
    definition: &'static SourceDefinition,
    resolver: &impl CommandResolver,
) -> SourceSnapshot {
    for candidate in definition.candidates {
        match resolver.resolve(candidate) {
            Ok(Some(executable)) => {
                return SourceSnapshot {
                    definition,
                    status: SourceStatus::Available { executable },
                };
            }
            Ok(None) => {}
            Err(message) => {
                return SourceSnapshot {
                    definition,
                    status: SourceStatus::Error { message },
                };
            }
        }
    }

    SourceSnapshot {
        definition,
        status: SourceStatus::Disabled {
            candidates: definition
                .candidates
                .iter()
                .map(|candidate| (*candidate).to_string())
                .collect(),
        },
    }
}
