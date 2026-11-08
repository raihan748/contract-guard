use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "contract-guard",
    version,
    about = "API schema & function contract regression snapshotter and semantic breaking change differ",
    long_about = "contract-guard safeguards API backward compatibility by capturing canonical snapshots of OpenAPI/JSON Schema contracts and semantically detecting breaking changes against baselines."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Records a canonical snapshot of an API or JSON Schema contract
    Snapshot {
        /// Path to OpenAPI spec (JSON/YAML) or JSON Schema definition
        #[arg(short, long)]
        spec: PathBuf,

        /// Output path for canonical snapshot JSON
        #[arg(short, long, default_value = "contract-snapshot.json")]
        out: PathBuf,
    },

    /// Verifies current schema against a baseline snapshot, identifying breaking regressions
    Check {
        /// Path to current OpenAPI spec or JSON Schema definition
        #[arg(short, long)]
        spec: PathBuf,

        /// Path to baseline snapshot JSON (or previous schema spec)
        #[arg(short, long)]
        baseline: PathBuf,

        /// Exit with non-zero code (1) if breaking regressions are found
        #[arg(long, default_value_t = false)]
        fail_on_breaking: bool,

        /// Diff report output format: table, json, or markdown
        #[arg(short, long, default_value = "table")]
        format: String,

        /// Optional file path to write diff report output
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
}
