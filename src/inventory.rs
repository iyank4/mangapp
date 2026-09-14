use std::{
    collections::HashMap,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;

use crate::model::{
    ApplicationRecord, CellState, CellValue, RecordStatus, SourceSnapshot, SourceStatus,
};

pub trait CommandRunner {
    fn run(&self, executable: &Path, args: &[String]) -> Result<String, String>;

    fn run_update_check(&self, executable: &Path, args: &[String]) -> Result<String, String> {
        self.run(executable, args)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ProcessRunner;

impl CommandRunner for ProcessRunner {
    fn run(&self, executable: &Path, args: &[String]) -> Result<String, String> {
        let output = Command::new(executable)
            .args(args)
            .output()
            .map_err(|error| format!("gagal menjalankan {}: {error}", executable.display()))?;

        if !output.status.success() {
            let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            return Err(if detail.is_empty() {
                format!("command keluar dengan status {}", output.status)
            } else {
                detail
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    fn run_update_check(&self, executable: &Path, args: &[String]) -> Result<String, String> {
        let output = Command::new(executable)
            .args(args)
            .output()
            .map_err(|error| format!("gagal menjalankan {}: {error}", executable.display()))?;

        if output.status.success() || output.status.code() == Some(1) {
            return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
        }

        let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        Err(if detail.is_empty() {
            format!("command keluar dengan status {}", output.status)
        } else {
            detail
        })
    }
}

pub fn collect_inventory(sources: &[SourceSnapshot]) -> Vec<ApplicationRecord> {
    collect_inventory_with_runner(sources, &ProcessRunner)
}

pub fn collect_inventory_with_runner(
    sources: &[SourceSnapshot],
    runner: &dyn CommandRunner,
) -> Vec<ApplicationRecord> {
    sources
        .iter()
        .flat_map(|snapshot| collect_source(snapshot, runner))
        .collect()
}

pub fn upgrade_inventory(
    source: &SourceSnapshot,
    records: &[ApplicationRecord],
) -> Result<(), String> {
    let mut reporter = |_message: String| {};
    upgrade_inventory_with_reporter(source, records, &mut reporter)
}

pub fn upgrade_inventory_with_runner(
    source: &SourceSnapshot,
    records: &[ApplicationRecord],
    runner: &dyn CommandRunner,
) -> Result<(), String> {
    let mut reporter = |_message: String| {};
    upgrade_inventory_with_runner_and_reporter(source, records, runner, &mut reporter)
}

pub fn upgrade_inventory_with_reporter(
    source: &SourceSnapshot,
    records: &[ApplicationRecord],
    reporter: &mut dyn FnMut(String),
) -> Result<(), String> {
    upgrade_inventory_with_runner_and_reporter(source, records, &ProcessRunner, reporter)
}

pub fn upgrade_inventory_with_runner_and_reporter(
    source: &SourceSnapshot,
    records: &[ApplicationRecord],
    runner: &dyn CommandRunner,
    reporter: &mut dyn FnMut(String),
) -> Result<(), String> {
    let SourceStatus::Available { executable } = &source.status else {
        return Err("source tidak tersedia untuk upgrade".into());
    };
    let identifiers = upgrade_identifiers(records);
    if identifiers.is_empty() {
        return Ok(());
    }

    let commands = upgrade_commands(source.definition.id, executable, identifiers)?;

    run_upgrade_commands(runner, executable, commands, reporter)
}

pub fn upgrade_inventory_interactive(
    source: &SourceSnapshot,
    records: &[ApplicationRecord],
) -> Result<(), String> {
    let SourceStatus::Available { executable } = &source.status else {
        return Err("source tidak tersedia untuk upgrade".into());
    };
    let identifiers = upgrade_identifiers(records);
    if identifiers.is_empty() {
        return Ok(());
    }

    let commands = upgrade_commands(source.definition.id, executable, identifiers)?;
    run_interactive_upgrade_commands(executable, commands)
}

fn upgrade_identifiers(records: &[ApplicationRecord]) -> Vec<String> {
    records
        .iter()
        .filter_map(|record| match &record.identifier.state {
            CellState::Value(identifier) if !identifier.is_empty() => Some(identifier.clone()),
            _ => None,
        })
        .collect()
}

fn upgrade_commands(
    source: &str,
    executable: &Path,
    identifiers: Vec<String>,
) -> Result<Vec<Vec<String>>, String> {
    Ok(match source {
        "cargo" => identifiers
            .iter()
            .map(|identifier| vec!["install".into(), "--force".into(), identifier.clone()])
            .collect(),
        "homebrew" => vec![vec!["upgrade".into()]],
        "mas" => identifiers
            .iter()
            .map(|identifier| vec!["upgrade".into(), identifier.clone()])
            .collect(),
        "python-pip" => vec![pip_upgrade_command(executable, identifiers)],
        "rubygems" => vec![upgrade_command("update", identifiers)],
        "uv" => vec![vec!["tool".into(), "upgrade".into(), "--all".into()]],
        "pipx" => vec![vec!["upgrade-all".into()]],
        "npm" => vec![vec!["update".into(), "--global".into()]],
        "pnpm" => vec![vec!["update".into(), "--global".into()]],
        "yarn" => vec![vec!["global".into(), "upgrade".into()]],
        "composer" => vec![vec!["global".into(), "update".into()]],
        "conda" => vec![vec!["update".into(), "--all".into(), "--yes".into()]],
        "macports" => vec![vec!["upgrade".into(), "outdated".into()]],
        "nix" => vec![vec!["profile".into(), "upgrade".into(), "--all".into()]],
        "dart" | "flutter" => identifiers
            .iter()
            .map(|identifier| {
                vec![
                    "pub".into(),
                    "global".into(),
                    "activate".into(),
                    identifier.clone(),
                ]
            })
            .collect(),
        "go" => return Err("source Go tidak memiliki aturan upgrade universal".into()),
        _ => return Err("aturan upgrade belum didukung source".into()),
    })
}

fn pip_upgrade_command(executable: &Path, identifiers: Vec<String>) -> Vec<String> {
    let mut command = if executable.file_name().and_then(|name| name.to_str()) == Some("python3") {
        vec![
            "-m".into(),
            "pip".into(),
            "install".into(),
            "--upgrade".into(),
        ]
    } else {
        vec!["install".into(), "--upgrade".into()]
    };
    command.extend(identifiers);
    command
}

fn upgrade_command(action: &str, identifiers: Vec<String>) -> Vec<String> {
    let mut command = vec![action.to_owned()];
    command.extend(identifiers);
    command
}

fn run_upgrade_commands(
    runner: &dyn CommandRunner,
    executable: &Path,
    commands: Vec<Vec<String>>,
    reporter: &mut dyn FnMut(String),
) -> Result<(), String> {
    let mut errors = Vec::new();
    for command in commands {
        let label = format!("{} {}", executable.display(), command.join(" "));
        reporter(format!("Menjalankan: {label}"));
        match runner.run(executable, &command) {
            Err(error) => {
                reporter(format!("Gagal: {error}"));
                errors.push(error);
            }
            Ok(output) => {
                let output_lines = output
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .map(str::trim)
                    .collect::<Vec<_>>();
                let start = output_lines.len().saturating_sub(8);
                for line in &output_lines[start..] {
                    reporter(format!("  {line}"));
                }
                reporter(format!("Selesai: {label}"));
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

fn run_interactive_upgrade_commands(
    executable: &Path,
    commands: Vec<Vec<String>>,
) -> Result<(), String> {
    let mut errors = Vec::new();
    for command in commands {
        let label = format!("{} {}", executable.display(), command.join(" "));
        println!("Menjalankan: {label}");
        io::stdout()
            .flush()
            .map_err(|error| format!("gagal menampilkan console upgrade: {error}"))?;

        match Command::new(executable).args(&command).status() {
            Ok(status) if status.success() => println!("Selesai: {label}"),
            Ok(status) => {
                let error = format!("command keluar dengan status {status}");
                println!("Gagal: {error}");
                errors.push(error);
            }
            Err(error) => {
                let error = format!("gagal menjalankan {}: {error}", executable.display());
                println!("Gagal: {error}");
                errors.push(error);
            }
        }
        io::stdout()
            .flush()
            .map_err(|error| format!("gagal menampilkan console upgrade: {error}"))?;
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

fn collect_source(snapshot: &SourceSnapshot, runner: &dyn CommandRunner) -> Vec<ApplicationRecord> {
    let source = snapshot.definition.id;
    match &snapshot.status {
        SourceStatus::Disabled { candidates } => vec![unavailable_record(
            source,
            &format!(
                "inventory tidak tersedia: {} tidak ditemukan",
                candidates.join(" / ")
            ),
        )],
        SourceStatus::Error { message } => vec![error_record(source, message)],
        SourceStatus::Available { executable } => {
            let result = match source {
                "cargo" => run_and_parse(
                    runner,
                    executable,
                    source,
                    &["install", "--list"],
                    parse_cargo,
                ),
                "homebrew" => collect_homebrew(runner, executable),
                "mas" => run_and_parse(runner, executable, source, &["list"], parse_mas),
                "python-pip" => collect_python_pip(runner, executable),
                "rubygems" => {
                    run_and_parse(runner, executable, source, &["list", "--local"], parse_gem)
                }
                "uv" => run_and_parse(runner, executable, source, &["tool", "list"], parse_uv),
                "pipx" => run_and_parse(runner, executable, source, &["list"], parse_pipx),
                "npm" | "pnpm" => run_and_parse(
                    runner,
                    executable,
                    source,
                    &["list", "--global", "--depth=0"],
                    parse_node,
                ),
                "yarn" => run_and_parse(
                    runner,
                    executable,
                    source,
                    &["global", "list", "--depth=0"],
                    parse_yarn,
                ),
                "composer" => run_and_parse(
                    runner,
                    executable,
                    source,
                    &["global", "show"],
                    parse_composer,
                ),
                "conda" => run_and_parse(runner, executable, source, &["list"], parse_conda),
                "macports" => {
                    run_and_parse(runner, executable, source, &["installed"], parse_macports)
                }
                "nix" => run_and_parse(runner, executable, source, &["profile", "list"], parse_nix),
                "dart" => run_and_parse(
                    runner,
                    executable,
                    source,
                    &["pub", "global", "list"],
                    parse_generic,
                ),
                "flutter" => run_and_parse(
                    runner,
                    executable,
                    source,
                    &["pub", "global", "list"],
                    parse_generic,
                ),
                "go" => collect_go(runner, executable),
                _ => Ok(vec![unavailable_record(
                    source,
                    "collector source belum tersedia",
                )]),
            };

            match result {
                Ok(mut records) => {
                    populate_available_versions(source, executable, &mut records, runner);
                    records
                }
                Err(message) => vec![error_record(source, &message)],
            }
        }
    }
}

fn populate_available_versions(
    source: &str,
    executable: &Path,
    records: &mut [ApplicationRecord],
    runner: &dyn CommandRunner,
) {
    let result = match source {
        "cargo" => check_cargo_updates(runner, executable, records),
        "homebrew" => check_homebrew_updates(runner, executable),
        "mas" => check_text_updates(runner, executable, &["outdated"], parse_mas_updates),
        "python-pip" => check_python_pip_updates(runner, executable),
        "rubygems" => check_text_updates(runner, executable, &["outdated"], parse_gem_updates),
        "uv" => check_uv_updates(runner, executable, records),
        "pipx" => check_text_updates(
            runner,
            executable,
            &["list", "--outdated"],
            parse_package_updates,
        ),
        "npm" => check_json_updates(
            runner,
            executable,
            &["outdated", "--global", "--depth=0", "--json"],
            parse_registry_updates,
        ),
        "pnpm" => check_json_updates(
            runner,
            executable,
            &["outdated", "--global", "--format", "json"],
            parse_registry_updates,
        ),
        "yarn" => check_yarn_updates(runner, executable),
        "composer" => check_json_updates(
            runner,
            executable,
            &["global", "outdated", "--format=json"],
            parse_composer_updates,
        ),
        "conda" => check_json_updates(
            runner,
            executable,
            &["update", "--all", "--dry-run", "--json"],
            parse_conda_updates,
        ),
        "macports" => check_text_updates(runner, executable, &["outdated"], parse_macports_updates),
        "nix" => check_text_updates(
            runner,
            executable,
            &["profile", "upgrade", "--all", "--dry-run"],
            parse_nix_updates,
        ),
        "dart" | "flutter" => check_pub_updates(runner, records),
        "go" => check_go_updates(runner, executable, records),
        _ => Err("pengecekan versi terbaru belum didukung source".into()),
    };

    match result {
        Ok(updates) => {
            for record in records {
                if let Some(version) = updates.get(&record.identifier.display().to_lowercase()) {
                    record.available_version = CellValue::value(version);
                }
            }
        }
        Err(_) => {
            for record in records {
                record.available_version = CellValue::unavailable();
            }
        }
    }
}

fn check_cargo_updates(
    runner: &dyn CommandRunner,
    executable: &Path,
    records: &[ApplicationRecord],
) -> Result<HashMap<String, String>, String> {
    let mut updates = HashMap::new();
    for record in records {
        let identifier = record.identifier.display();
        let args = vec![
            "search".to_owned(),
            identifier.to_owned(),
            "--limit".to_owned(),
            "1".to_owned(),
        ];
        let Ok(output) = runner.run_update_check(executable, &args) else {
            continue;
        };
        if let Some(version) = parse_cargo_update(identifier, &output)
            && version != record.version.display()
        {
            updates.insert(identifier.to_lowercase(), version);
        }
    }
    Ok(updates)
}

fn check_homebrew_updates(
    runner: &dyn CommandRunner,
    executable: &Path,
) -> Result<HashMap<String, String>, String> {
    let mut updates = HashMap::new();
    for kind in ["formula", "cask"] {
        let args = vec!["outdated".into(), format!("--{kind}"), "--json=v2".into()];
        let output = runner.run_update_check(executable, &args)?;
        updates.extend(parse_homebrew_updates(&output));
    }
    Ok(updates)
}

fn check_text_updates(
    runner: &dyn CommandRunner,
    executable: &Path,
    args: &[&str],
    parser: fn(&str) -> HashMap<String, String>,
) -> Result<HashMap<String, String>, String> {
    let args = args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>();
    let output = runner.run_update_check(executable, &args)?;
    Ok(parser(&output))
}

fn check_json_updates(
    runner: &dyn CommandRunner,
    executable: &Path,
    args: &[&str],
    parser: fn(&Value) -> HashMap<String, String>,
) -> Result<HashMap<String, String>, String> {
    let args = args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>();
    let output = runner.run_update_check(executable, &args)?;
    let value = serde_json::from_str(&output)
        .map_err(|error| format!("format update tidak valid: {error}"))?;
    Ok(parser(&value))
}

fn check_python_pip_updates(
    runner: &dyn CommandRunner,
    executable: &Path,
) -> Result<HashMap<String, String>, String> {
    let args = if executable.file_name().and_then(|name| name.to_str()) == Some("python3") {
        vec![
            "-m".into(),
            "pip".into(),
            "list".into(),
            "--outdated".into(),
            "--format=json".into(),
        ]
    } else {
        vec!["list".into(), "--outdated".into(), "--format=json".into()]
    };
    let output = runner.run_update_check(executable, &args)?;
    let value: Value = serde_json::from_str(&output)
        .map_err(|error| format!("format update pip tidak valid: {error}"))?;
    Ok(parse_registry_updates(&value))
}

fn check_pub_updates(
    runner: &dyn CommandRunner,
    records: &[ApplicationRecord],
) -> Result<HashMap<String, String>, String> {
    let mut updates = HashMap::new();
    for record in records {
        let identifier = record.identifier.display();
        let args = vec![
            "-fsSL".to_owned(),
            format!("https://pub.dev/api/packages/{identifier}"),
        ];
        let Ok(output) = runner.run_update_check(Path::new("curl"), &args) else {
            continue;
        };
        if let Some(version) = parse_pub_update(&output)
            && version != record.version.display()
        {
            updates.insert(identifier.to_lowercase(), version);
        }
    }
    Ok(updates)
}

fn check_uv_updates(
    runner: &dyn CommandRunner,
    executable: &Path,
    records: &[ApplicationRecord],
) -> Result<HashMap<String, String>, String> {
    let mut updates = HashMap::new();
    for record in records {
        let identifier = record.identifier.display();
        let args = vec![
            "pip".to_owned(),
            "install".to_owned(),
            "--dry-run".to_owned(),
            "--system".to_owned(),
            "--break-system-packages".to_owned(),
            "--no-deps".to_owned(),
            identifier.to_owned(),
        ];
        let Ok(output) = runner.run_update_check(executable, &args) else {
            continue;
        };
        if let Some(version) = parse_uv_update(identifier, &output) {
            updates.insert(identifier.to_lowercase(), version);
        }
    }
    Ok(updates)
}

fn check_yarn_updates(
    runner: &dyn CommandRunner,
    executable: &Path,
) -> Result<HashMap<String, String>, String> {
    let directory_args = vec!["global".to_owned(), "dir".to_owned()];
    let directory = runner.run_update_check(executable, &directory_args)?;
    let directory = directory.trim();
    if directory.is_empty() {
        return Ok(HashMap::new());
    }
    let args = vec![
        "outdated".to_owned(),
        "--cwd".to_owned(),
        directory.to_owned(),
        "--json".to_owned(),
    ];
    let output = runner.run_update_check(executable, &args)?;
    let value: Value = serde_json::from_str(&output)
        .map_err(|error| format!("format update Yarn tidak valid: {error}"))?;
    Ok(parse_registry_updates(&value))
}

fn check_go_updates(
    runner: &dyn CommandRunner,
    executable: &Path,
    records: &[ApplicationRecord],
) -> Result<HashMap<String, String>, String> {
    let mut updates = HashMap::new();
    for record in records {
        let location = record.location.display();
        if location == "N/A" || location == "UNAVAILABLE" || location == "ERROR" {
            continue;
        }
        let inspect_args = vec!["version".to_owned(), "-m".to_owned(), location.to_owned()];
        let Ok(inspect_output) = runner.run_update_check(executable, &inspect_args) else {
            continue;
        };
        let Some(module) = inspect_output.lines().find_map(|line| {
            line.trim()
                .strip_prefix("path ")
                .map(str::trim)
                .filter(|value| !value.is_empty())
        }) else {
            continue;
        };
        let query = format!("{module}@latest");
        let query_args = vec![
            "list".to_owned(),
            "-m".to_owned(),
            "-json".to_owned(),
            query,
        ];
        let Ok(query_output) = runner.run_update_check(executable, &query_args) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&query_output) else {
            continue;
        };
        if let Some(version) = value.get("Version").and_then(Value::as_str) {
            updates.insert(
                record.identifier.display().to_lowercase(),
                version.to_owned(),
            );
        }
    }
    Ok(updates)
}

fn parse_cargo_update(identifier: &str, output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let (name, remainder) = line.split_once(" = \"")?;
        if name.trim() != identifier {
            return None;
        }
        remainder.split('"').next().map(str::to_owned)
    })
}

fn parse_homebrew_updates(output: &str) -> HashMap<String, String> {
    let Ok(value) = serde_json::from_str::<Value>(output) else {
        return HashMap::new();
    };
    let mut updates = HashMap::new();
    for key in ["formulae", "casks"] {
        if let Some(items) = value.get(key).and_then(Value::as_array) {
            for item in items {
                let Some(name) = item.get("name").and_then(Value::as_str) else {
                    continue;
                };
                let version = item
                    .get("current_version")
                    .and_then(Value::as_str)
                    .or_else(|| {
                        item.get("versions")
                            .and_then(|versions| versions.get("stable"))
                            .and_then(Value::as_str)
                    });
                if let Some(version) = version {
                    updates.insert(name.to_lowercase(), version.to_owned());
                }
            }
        }
    }
    updates
}

fn parse_mas_updates(output: &str) -> HashMap<String, String> {
    output
        .lines()
        .filter_map(|line| {
            let identifier = line.split_whitespace().next()?;
            if !identifier
                .chars()
                .all(|character| character.is_ascii_digit())
            {
                return None;
            }
            let (_, version) = split_parenthesized_version(line);
            (version != "UNKNOWN").then(|| (identifier.to_lowercase(), version.to_owned()))
        })
        .collect()
}

fn parse_gem_updates(output: &str) -> HashMap<String, String> {
    output
        .lines()
        .filter_map(|line| {
            let (identifier, detail) = line.split_once(" (")?;
            let version = detail
                .split_once("<")
                .map(|(_, version)| version)
                .or_else(|| detail.split_once("newest ").map(|(_, version)| version))?
                .trim_end_matches(')')
                .trim();
            (!identifier.is_empty() && !version.is_empty())
                .then(|| (identifier.to_lowercase(), version.to_owned()))
        })
        .collect()
}

fn parse_package_updates(output: &str) -> HashMap<String, String> {
    output
        .lines()
        .filter_map(|line| {
            let latest = line
                .split_once("latest:")
                .map(|(_, value)| value)
                .or_else(|| line.split_once(" -> ").map(|(_, value)| value))?
                .split_whitespace()
                .next()?
                .trim_matches(['v', ',', ')'])
                .to_owned();
            let before = line
                .split_once("latest:")
                .map(|(value, _)| value)
                .or_else(|| line.split_once(" -> ").map(|(value, _)| value))?;
            let identifier = before
                .split_whitespace()
                .find(|token| !token.contains(':') && !token.starts_with('('))?;
            Some((
                identifier.trim_matches(['├', '└', '─']).to_lowercase(),
                latest,
            ))
        })
        .collect()
}

fn parse_uv_update(identifier: &str, output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        line.split_whitespace().find_map(|token| {
            let token = token.trim_matches(['+', ',', ':']);
            let (name, version) = token.split_once("==")?;
            (name.eq_ignore_ascii_case(identifier) && !version.is_empty())
                .then(|| version.to_owned())
        })
    })
}

fn parse_registry_updates(value: &Value) -> HashMap<String, String> {
    let mut updates = HashMap::new();
    match value {
        Value::Object(object) => {
            for (key, item) in object {
                if let Some(latest) = item
                    .get("latest")
                    .and_then(Value::as_str)
                    .or_else(|| item.get("latest_version").and_then(Value::as_str))
                {
                    updates.insert(key.to_lowercase(), latest.to_owned());
                } else {
                    updates.extend(parse_registry_updates(item));
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                if let Some(row) = item.as_array() {
                    if row.len() >= 4 {
                        let identifier = row.first().and_then(Value::as_str);
                        let latest = row.get(3).and_then(Value::as_str);
                        if let (Some(identifier), Some(latest)) = (identifier, latest)
                            && identifier != "Package"
                        {
                            updates.insert(identifier.to_lowercase(), latest.to_owned());
                        }
                    }
                    continue;
                }
                let Some(object) = item.as_object() else {
                    continue;
                };
                let identifier = object
                    .get("name")
                    .or_else(|| object.get("package"))
                    .or_else(|| object.get("id"))
                    .and_then(Value::as_str);
                let latest = object
                    .get("latest")
                    .or_else(|| object.get("latest_version"))
                    .and_then(Value::as_str);
                if let (Some(identifier), Some(latest)) = (identifier, latest) {
                    updates.insert(identifier.to_lowercase(), latest.to_owned());
                }
            }
        }
        _ => {}
    }
    updates
}

fn parse_composer_updates(value: &Value) -> HashMap<String, String> {
    value
        .get("installed")
        .map(parse_registry_updates)
        .unwrap_or_default()
}

fn parse_conda_updates(value: &Value) -> HashMap<String, String> {
    let mut updates = HashMap::new();
    if let Some(items) = value
        .get("actions")
        .and_then(|actions| actions.get("LINK"))
        .and_then(Value::as_array)
    {
        for item in items {
            let Some(name) = item.get("name").and_then(Value::as_str) else {
                continue;
            };
            let Some(version) = item.get("version").and_then(Value::as_str) else {
                continue;
            };
            updates.insert(name.to_lowercase(), version.to_owned());
        }
    }
    updates
}

fn parse_macports_updates(output: &str) -> HashMap<String, String> {
    output
        .lines()
        .filter_map(|line| {
            let (identifier, versions) = line.trim().split_once(" @")?;
            let (_, latest) = versions.split_once(" < ")?;
            let latest = latest.split_whitespace().next()?;
            Some((identifier.trim().to_lowercase(), latest.to_owned()))
        })
        .collect()
}

fn parse_nix_updates(output: &str) -> HashMap<String, String> {
    output
        .lines()
        .filter_map(|line| {
            let from_index = line.find(" from ")?;
            let to_index = line[from_index + 6..].find(" to ")? + from_index + 6;
            let identifier = line[..from_index]
                .split_whitespace()
                .last()?
                .trim_matches(['\'', '"']);
            let version = line[to_index + 4..]
                .split_whitespace()
                .next()?
                .trim_matches(['\'', '"', ',']);
            Some((identifier.to_lowercase(), version.to_owned()))
        })
        .collect()
}

fn parse_pub_update(output: &str) -> Option<String> {
    if let Ok(value) = serde_json::from_str::<Value>(output)
        && let Some(version) = value
            .get("latest")
            .and_then(|latest| latest.get("version"))
            .and_then(Value::as_str)
    {
        return Some(version.to_owned());
    }

    output.lines().find_map(|line| {
        line.split_once("latest:")
            .or_else(|| line.split_once("latest version"))
            .and_then(|(_, value)| value.split_whitespace().next())
            .map(|version| version.trim_matches(['v', ',', ')']).to_owned())
    })
}

fn run_and_parse(
    runner: &dyn CommandRunner,
    executable: &Path,
    source: &str,
    args: &[&str],
    parser: fn(&str, &str) -> Vec<ApplicationRecord>,
) -> Result<Vec<ApplicationRecord>, String> {
    let args = args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>();
    let output = runner.run(executable, &args)?;
    Ok(parser(source, &output))
}

fn collect_homebrew(
    runner: &dyn CommandRunner,
    executable: &Path,
) -> Result<Vec<ApplicationRecord>, String> {
    let mut records = Vec::new();
    for kind in ["formula", "cask"] {
        let args = vec![
            "list".to_owned(),
            format!("--{kind}"),
            "--versions".to_owned(),
        ];
        let output = runner.run(executable, &args)?;
        records.extend(parse_homebrew(output.as_str(), kind));
    }
    Ok(records)
}

fn collect_python_pip(
    runner: &dyn CommandRunner,
    executable: &Path,
) -> Result<Vec<ApplicationRecord>, String> {
    let args = if executable.file_name().and_then(|name| name.to_str()) == Some("python3") {
        vec![
            "-m".into(),
            "pip".into(),
            "list".into(),
            "--format=freeze".into(),
        ]
    } else {
        vec!["list".into(), "--format=freeze".into()]
    };
    let output = runner.run(executable, &args)?;
    Ok(parse_freeze("python-pip", &output))
}

fn collect_go(
    runner: &dyn CommandRunner,
    executable: &Path,
) -> Result<Vec<ApplicationRecord>, String> {
    let args = ["env", "GOBIN", "GOPATH"]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let output = runner.run(executable, &args)?;
    let values = output.lines().map(str::trim).collect::<Vec<_>>();
    let bin_dir = values
        .first()
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            values
                .get(1)
                .filter(|value| !value.is_empty())
                .map(|value| PathBuf::from(value).join("bin"))
        });
    let Some(bin_dir) = bin_dir else {
        return Ok(Vec::new());
    };

    let entries = std::fs::read_dir(bin_dir)
        .map_err(|error| format!("folder binary Go tidak dapat dibaca: {error}"))?;
    let mut records = Vec::new();
    for entry in entries.flatten() {
        if entry.path().is_file() {
            let identifier = entry.file_name().to_string_lossy().into_owned();
            let mut record =
                available_record("go", &identifier, &identifier, CellValue::unavailable());
            record.location = CellValue::value(entry.path().display().to_string());
            record.available_version = CellValue::unavailable();
            records.push(record);
        }
    }
    records.sort_by(|left, right| left.identifier.display().cmp(right.identifier.display()));
    Ok(records)
}

fn parse_cargo(_executable: &str, output: &str) -> Vec<ApplicationRecord> {
    let mut records = Vec::new();
    for line in output
        .lines()
        .filter(|line| !line.starts_with(char::is_whitespace))
    {
        let header = line.trim_end_matches(':');
        let Some(version_start) = header.rfind(" v") else {
            continue;
        };
        let name = header[..version_start].trim();
        let version = header[version_start + 2..].trim();
        if !name.is_empty() && !version.is_empty() {
            records.push(available_record(
                "cargo",
                name,
                name,
                CellValue::value(version),
            ));
        }
    }
    records
}

fn parse_homebrew(output: &str, kind: &str) -> Vec<ApplicationRecord> {
    output
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let identifier = fields.next()?;
            let version = fields.collect::<Vec<_>>().join(" ");
            let display_name = if kind == "cask" {
                format!("{identifier} (cask)")
            } else {
                identifier.to_owned()
            };
            Some(available_record(
                "homebrew",
                &display_name,
                identifier,
                if version.is_empty() {
                    CellValue::empty()
                } else {
                    CellValue::value(version)
                },
            ))
        })
        .collect()
}

fn parse_mas(_executable: &str, output: &str) -> Vec<ApplicationRecord> {
    output
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let identifier = fields.next()?;
            if !identifier
                .chars()
                .all(|character| character.is_ascii_digit())
            {
                return None;
            }
            let rest = fields.collect::<Vec<_>>().join(" ");
            let (name, version) = split_parenthesized_version(&rest);
            Some(available_record(
                "mas",
                name,
                identifier,
                if version == "UNKNOWN" {
                    CellValue::empty()
                } else {
                    CellValue::value(version)
                },
            ))
        })
        .collect()
}

fn parse_gem(_executable: &str, output: &str) -> Vec<ApplicationRecord> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let (identifier, versions) = line.split_once(" (")?;
            let version = versions.trim_end_matches(')');
            Some(available_record(
                "rubygems",
                identifier,
                identifier,
                CellValue::value(version),
            ))
        })
        .collect()
}

fn parse_freeze(source: &str, output: &str) -> Vec<ApplicationRecord> {
    output
        .lines()
        .filter_map(|line| {
            let (identifier, version) = line.split_once("==")?;
            if identifier.is_empty() || version.is_empty() {
                return None;
            }
            Some(available_record(
                source,
                identifier,
                identifier,
                CellValue::value(version),
            ))
        })
        .collect()
}

fn parse_uv(_executable: &str, output: &str) -> Vec<ApplicationRecord> {
    parse_package_lines("uv", output)
}

fn parse_pipx(_executable: &str, output: &str) -> Vec<ApplicationRecord> {
    parse_package_lines("pipx", output)
}

fn parse_node(source: &str, output: &str) -> Vec<ApplicationRecord> {
    parse_package_lines(source, output)
}

fn parse_yarn(_executable: &str, output: &str) -> Vec<ApplicationRecord> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let token = line
                .strip_prefix("info ")
                .and_then(|value| value.split('"').nth(1))
                .or_else(|| {
                    line.strip_prefix(['├', '└'])
                        .map(|value| value.trim_start_matches(['─', ' ']))
                        .and_then(|value| value.split_whitespace().next())
                })?;
            let (identifier, version) = split_at_version(token);
            Some(available_record(
                "yarn",
                identifier,
                identifier,
                version.map_or_else(CellValue::empty, CellValue::value),
            ))
        })
        .collect()
}

fn parse_composer(_executable: &str, output: &str) -> Vec<ApplicationRecord> {
    output
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let identifier = fields.next()?;
            if !identifier.contains('/') {
                return None;
            }
            let version = fields.next().unwrap_or("UNKNOWN").trim_start_matches('v');
            Some(available_record(
                "composer",
                identifier,
                identifier,
                CellValue::value(version),
            ))
        })
        .collect()
}

fn parse_generic(source: &str, output: &str) -> Vec<ApplicationRecord> {
    parse_package_lines(source, output)
}

fn parse_conda(_executable: &str, output: &str) -> Vec<ApplicationRecord> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let mut fields = line.split_whitespace();
            let identifier = fields.next()?;
            let version = fields.next().unwrap_or_default();
            if identifier == "Name"
                || identifier == "#"
                || identifier.chars().all(|character| character == '-')
            {
                return None;
            }
            Some(available_record(
                "conda",
                identifier,
                identifier,
                if version.is_empty() {
                    CellValue::empty()
                } else {
                    CellValue::value(version)
                },
            ))
        })
        .collect()
}

fn parse_macports(_executable: &str, output: &str) -> Vec<ApplicationRecord> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let line = line.strip_prefix(" ").unwrap_or(line);
            let mut fields = line.split_whitespace();
            let identifier = fields.next()?;
            if identifier == "The" || identifier == "No" || identifier == "Currently" {
                return None;
            }
            let version = fields
                .find_map(|field| field.strip_prefix('@'))
                .unwrap_or_default();
            Some(available_record(
                "macports",
                identifier,
                identifier,
                if version.is_empty() {
                    CellValue::empty()
                } else {
                    CellValue::value(version)
                },
            ))
        })
        .collect()
}

fn parse_nix(_executable: &str, output: &str) -> Vec<ApplicationRecord> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let name = line
                .strip_prefix("Name:")
                .map(str::trim)
                .or_else(|| line.strip_prefix("- ").map(str::trim))?;
            if name.is_empty() {
                return None;
            }
            let version = line
                .split_once("Version:")
                .map(|(_, version)| version.trim())
                .unwrap_or_default();
            Some(available_record(
                "nix",
                name,
                name,
                if version.is_empty() {
                    CellValue::empty()
                } else {
                    CellValue::value(version)
                },
            ))
        })
        .collect()
}

fn parse_package_lines(source: &str, output: &str) -> Vec<ApplicationRecord> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim().trim_start_matches(['├', '└', '─', '│', ' ']);
            let mut fields = line.split_whitespace();
            let first = fields.next()?;
            if first.is_empty()
                || first.starts_with('/')
                || matches!(first, "#" | "info" | "Package")
            {
                return None;
            }
            let (identifier, inline_version) = if first == "package" {
                let identifier = fields.next()?;
                split_at_version(identifier)
            } else {
                split_at_version(first)
            };
            let version = inline_version
                .or_else(|| fields.clone().find_map(|field| normalized_version(field)));
            Some(available_record(
                source,
                identifier,
                identifier,
                version.map_or_else(CellValue::empty, CellValue::value),
            ))
        })
        .collect()
}

fn normalized_version(value: &str) -> Option<&str> {
    let value = value.trim_matches([',', '(', ')']);
    let value = value.strip_prefix('v').unwrap_or(value);
    (!value.is_empty()
        && value
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_digit()))
    .then_some(value)
}

fn split_at_version(value: &str) -> (&str, Option<&str>) {
    let Some(index) = value.rfind('@') else {
        return (value, None);
    };
    if index == 0 {
        let Some(second) = value[1..].rfind('@') else {
            return (value, None);
        };
        let split = second + 1;
        return (&value[..split], Some(&value[split + 1..]));
    }
    (&value[..index], Some(&value[index + 1..]))
}

fn split_parenthesized_version(value: &str) -> (&str, &str) {
    let Some(start) = value.rfind(" (") else {
        return (value.trim(), "UNKNOWN");
    };
    (
        value[..start].trim(),
        value[start + 2..].trim_end_matches(')').trim(),
    )
}

fn available_record(
    source: &str,
    name: &str,
    identifier: &str,
    version: CellValue,
) -> ApplicationRecord {
    let mut record = ApplicationRecord::new(
        source,
        CellValue::value(identifier),
        CellValue::value(name),
        version,
        RecordStatus::Available,
    );
    record.location = CellValue::not_applicable();
    record.available_version = CellValue::not_applicable();
    record.installed_at = CellValue::not_applicable();
    record.updated_at = CellValue::not_applicable();
    record
}

fn unavailable_record(source: &str, note: &str) -> ApplicationRecord {
    let mut record = ApplicationRecord::new(
        source,
        CellValue::unavailable(),
        CellValue::unavailable(),
        CellValue::unavailable(),
        RecordStatus::Unavailable,
    );
    record.note = CellValue::value(note);
    record.available_version = CellValue::unavailable();
    record
}

fn error_record(source: &str, message: &str) -> ApplicationRecord {
    let mut record = ApplicationRecord::new(
        source,
        CellValue::error(),
        CellValue::unavailable(),
        CellValue::error(),
        RecordStatus::Error,
    );
    record.note = CellValue::value(message);
    record.available_version = CellValue::error();
    record
}
