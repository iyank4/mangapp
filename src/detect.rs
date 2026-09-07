use std::ffi::OsString;
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
