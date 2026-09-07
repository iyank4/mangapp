use std::path::PathBuf;

use manapp::detect::{CommandResolver, PathResolver, detect_source, path_entries};
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

#[test]
fn path_resolver_finds_only_executable_files_and_rejects_empty_names() {
    use std::fs::{self, File};
    use std::os::unix::fs::PermissionsExt;
    use std::time::{SystemTime, UNIX_EPOCH};

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before epoch")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!("manapp-detector-{suffix}"));
    fs::create_dir(&directory).expect("create detector fixture");

    let executable = directory.join("installed-tool");
    File::create(&executable).expect("create executable fixture");
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755))
        .expect("make fixture executable");
    let non_executable = directory.join("readme");
    File::create(&non_executable).expect("create non-executable fixture");

    let resolver = PathResolver::new(directory.as_os_str());
    assert_eq!(
        resolver.resolve("installed-tool").unwrap(),
        Some(executable)
    );
    assert_eq!(resolver.resolve("readme").unwrap(), None);
    assert_eq!(resolver.resolve("missing-tool").unwrap(), None);
    assert!(resolver.resolve("").is_err());
    assert_eq!(PathResolver::new(":").resolve(".").unwrap(), None);
    assert!(resolver.resolve("\0").is_err());
    let _environment_resolver = PathResolver::from_environment();

    fs::remove_dir_all(directory).expect("remove detector fixture");
}

#[test]
fn path_entries_preserve_read_order_and_duplicates() {
    let entries = path_entries(std::ffi::OsStr::new("/first:/second:/first"));

    assert_eq!(
        entries,
        vec![
            PathBuf::from("/first"),
            PathBuf::from("/second"),
            PathBuf::from("/first"),
        ]
    );
}
