use crate::model::SourceDefinition;

static SOURCES: &[SourceDefinition] = &[
    SourceDefinition {
        id: "cargo",
        name: "Cargo",
        candidates: &["cargo"],
        category: "Rust package manager",
    },
    SourceDefinition {
        id: "composer",
        name: "Composer",
        candidates: &["composer"],
        category: "PHP package manager",
    },
    SourceDefinition {
        id: "conda",
        name: "Conda",
        candidates: &["conda", "mamba"],
        category: "Environment/package manager",
    },
    SourceDefinition {
        id: "dart",
        name: "Dart",
        candidates: &["dart"],
        category: "Dart/Flutter toolchain",
    },
    SourceDefinition {
        id: "flutter",
        name: "Flutter",
        candidates: &["flutter"],
        category: "Dart/Flutter toolchain",
    },
    SourceDefinition {
        id: "go",
        name: "Go",
        candidates: &["go"],
        category: "Go toolchain",
    },
    SourceDefinition {
        id: "homebrew",
        name: "Homebrew",
        candidates: &["brew"],
        category: "System package manager",
    },
    SourceDefinition {
        id: "mas",
        name: "Mac App Store",
        candidates: &["mas"],
        category: "App store",
    },
    SourceDefinition {
        id: "macports",
        name: "MacPorts",
        candidates: &["port"],
        category: "System package manager",
    },
    SourceDefinition {
        id: "nix",
        name: "Nix",
        candidates: &["nix"],
        category: "System package manager",
    },
    SourceDefinition {
        id: "npm",
        name: "npm",
        candidates: &["npm"],
        category: "JavaScript package manager",
    },
    SourceDefinition {
        id: "pipx",
        name: "pipx",
        candidates: &["pipx"],
        category: "Python application manager",
    },
    SourceDefinition {
        id: "pnpm",
        name: "pnpm",
        candidates: &["pnpm"],
        category: "JavaScript package manager",
    },
    SourceDefinition {
        id: "python-pip",
        name: "Python pip",
        candidates: &["pip3", "pip", "python3"],
        category: "Python package manager",
    },
    SourceDefinition {
        id: "rubygems",
        name: "RubyGems",
        candidates: &["gem"],
        category: "Ruby package manager",
    },
    SourceDefinition {
        id: "uv",
        name: "uv",
        candidates: &["uv"],
        category: "Python tool manager",
    },
    SourceDefinition {
        id: "yarn",
        name: "Yarn",
        candidates: &["yarn"],
        category: "JavaScript package manager",
    },
];

pub fn source_registry() -> &'static [SourceDefinition] {
    SOURCES
}
