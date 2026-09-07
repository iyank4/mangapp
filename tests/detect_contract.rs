use std::path::PathBuf;

use manapp::detect::{CommandResolver, detect_source};
use manapp::model::SourceStatus;
use manapp::registry::source_registry;

#[derive(Clone)]
struct FakeResolver(Result<Option<PathBuf>, String>);

impl CommandResolver for FakeResolver {
    fn resolve(&self, _candidate: &str) -> Result<Option<PathBuf>, String> {
        self.0.clone()
    }
}

#[test]
fn detector_returns_available_with_the_first_resolved_executable() {
    let source = &source_registry()[0];
    let resolver = FakeResolver(Ok(Some(PathBuf::from("/usr/bin/cargo"))));

    let snapshot = detect_source(source, &resolver);

    assert_eq!(
        snapshot.status,
        SourceStatus::Available {
            executable: PathBuf::from("/usr/bin/cargo")
        }
    );
}

#[test]
fn detector_returns_disabled_when_all_candidates_are_absent() {
    let source = &source_registry()[2];
    let resolver = FakeResolver(Ok(None));

    let snapshot = detect_source(source, &resolver);

    assert_eq!(
        snapshot.status,
        SourceStatus::Disabled {
            candidates: vec!["conda".into(), "mamba".into()]
        }
    );
}

#[test]
fn detector_returns_error_when_resolver_fails() {
    let source = &source_registry()[6];
    let resolver = FakeResolver(Err("PATH tidak dapat dibaca".into()));

    let snapshot = detect_source(source, &resolver);

    assert_eq!(
        snapshot.status,
        SourceStatus::Error {
            message: "PATH tidak dapat dibaca".into()
        }
    );
}
