use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::model::{ApplicationRecord, CellValue, RecordStatus, SourceSnapshot, SourceStatus};

pub trait CommandRunner {
    fn run(&self, executable: &Path, args: &[String]) -> Result<String, String>;
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
                Ok(records) => records,
                Err(message) => vec![error_record(source, &message)],
            }
        }
    }
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
    record
}
