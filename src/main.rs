use clap::Parser;
use contract_guard::cli::{Cli, Commands};
use contract_guard::diff::DiffEngine;
use contract_guard::reporter::{render_report, OutputFormat};
use contract_guard::snapshot::{create_snapshot, load_snapshot, save_snapshot};
use std::fs;
use std::process;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Snapshot { spec, out } => {
            println!("📸 Capturing API contract snapshot from: {}", spec.display());
            match create_snapshot(&spec) {
                Ok(contract) => {
                    if let Err(e) = save_snapshot(&contract, &out) {
                        eprintln!("❌ Error saving snapshot: {e}");
                        process::exit(2);
                    }
                    println!(
                        "✅ Saved canonical snapshot to: {} (Endpoints: {}, Entities: {})",
                        out.display(),
                        contract.endpoints.len(),
                        contract.entities.len()
                    );
                }
                Err(e) => {
                    eprintln!("❌ Failed to parse spec for snapshot: {e}");
                    process::exit(2);
                }
            }
        }
        Commands::Check {
            spec,
            baseline,
            fail_on_breaking,
            format,
            out,
        } => {
            let base_contract = match load_snapshot(&baseline) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("❌ Failed to load baseline contract: {e}");
                    process::exit(2);
                }
            };

            let curr_contract = match create_snapshot(&spec) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("❌ Failed to parse current spec: {e}");
                    process::exit(2);
                }
            };

            let diff_result = DiffEngine::diff(&base_contract, &curr_contract);
            let output_format = OutputFormat::from_str_loose(&format);
            let rendered = render_report(&diff_result, output_format);

            if let Some(out_path) = out {
                if let Err(e) = fs::write(&out_path, &rendered) {
                    eprintln!("❌ Failed to write diff report to {}: {e}", out_path.display());
                    process::exit(2);
                }
                println!("📄 Diff report saved to: {}", out_path.display());
            } else {
                println!("{rendered}");
            }

            if fail_on_breaking && diff_result.is_breaking {
                eprintln!(
                    "🚨 Pipeline failed: {} breaking change(s) detected with --fail-on-breaking enabled.",
                    diff_result.breaking_count
                );
                process::exit(1);
            }
        }
    }
}
