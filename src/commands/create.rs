use anyhow::{Context, Result};
use clap::Args;
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::commands::{OutputFormat, gitignore, utils};

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Path to the Cargo.toml file
    #[arg(short, long, default_value = "Cargo.toml")]
    pub manifest_path: PathBuf,

    /// Output format (text or json)
    #[arg(long, default_value = "text")]
    pub format: OutputFormat,

    /// Custom exclude patterns (can be specified multiple times)
    #[arg(short = 'p', long)]
    pub patterns: Vec<String>,

    /// Dry run: show what would be added without modifying the file
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Serialize, Debug)]
struct CreateReport {
    /// Whether the operation was successful
    success: bool,
    /// Artefacts that were added
    added: Vec<String>,
    /// Artefacts that were already excluded
    already_excluded: Vec<String>,
    /// Artefacts already covered by .gitignore (skipped)
    gitignored: Vec<String>,
    /// Whether this was a dry run
    dry_run: bool,
}

impl CreateArgs {
    pub fn run(&self) -> Result<()> {
        let manifest_path = &self.manifest_path;

        // Read and parse Cargo.toml
        let cargo_toml_content = std::fs::read_to_string(manifest_path)
            .with_context(|| format!("Failed to read {}", manifest_path.display()))?;

        let cargo_toml: toml::Value = toml::from_str(&cargo_toml_content)
            .with_context(|| format!("Failed to parse {}", manifest_path.display()))?;

        // Get current exclude list
        let current_excludes = utils::get_current_excludes(&cargo_toml);

        // Read .gitignore entries
        let manifest_dir = manifest_path.parent().unwrap_or(Path::new("."));
        let gitignore_entries = gitignore::read_gitignore(manifest_dir);

        // Determine patterns to check (normalize by stripping trailing slashes)
        let patterns: Vec<String> = if !self.patterns.is_empty() {
            self.patterns
                .iter()
                .map(|p| utils::normalize_pattern(p))
                .collect()
        } else {
            utils::DEFAULT_EXCLUDE_PATTERNS
                .iter()
                .map(|s| s.to_string())
                .collect()
        };

        // Find patterns that exist in the repo but are not excluded and not gitignored
        let to_add: Vec<String> = patterns
            .iter()
            .filter(|pattern| {
                utils::exists_in_repo(manifest_dir, pattern.as_str())
                    && !utils::is_excluded(pattern, &current_excludes)
                    && !gitignore::is_gitignored(pattern, &gitignore_entries)
            })
            .map(|s| s.to_string())
            .collect();

        let already_excluded: Vec<String> = patterns
            .iter()
            .filter(|pattern| utils::is_excluded(pattern, &current_excludes))
            .map(|s| s.to_string())
            .collect();

        let gitignored: Vec<String> = patterns
            .iter()
            .filter(|pattern| {
                !utils::is_excluded(pattern, &current_excludes)
                    && gitignore::is_gitignored(pattern, &gitignore_entries)
            })
            .map(|s| s.to_string())
            .collect();

        if self.dry_run {
            let report = CreateReport {
                success: true,
                added: to_add.clone(),
                already_excluded,
                gitignored,
                dry_run: true,
            };
            self.output_report(&report, &to_add);
            return Ok(());
        }

        // Modify the Cargo.toml to add the new excludes
        let new_excludes: Vec<String> = current_excludes
            .iter()
            .chain(to_add.iter())
            .cloned()
            .collect();
        let new_content = utils::update_cargo_toml_excludes(&cargo_toml_content, &new_excludes)
            .with_context(|| "Failed to update Cargo.toml")?;

        std::fs::write(manifest_path, &new_content)
            .with_context(|| format!("Failed to write {}", manifest_path.display()))?;

        let report = CreateReport {
            success: true,
            added: to_add.clone(),
            already_excluded,
            gitignored,
            dry_run: false,
        };
        self.output_report(&report, &to_add);

        Ok(())
    }

    fn output_report(&self, report: &CreateReport, to_add: &[String]) {
        match self.format {
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(report).unwrap();
                println!("{}", json);
            }
            OutputFormat::Text => {
                if report.dry_run {
                    println!(
                        "Dry run — would add {} artefact(s) to exclude list:",
                        to_add.len()
                    );
                } else {
                    println!(
                        "Added {} artefact(s) to Cargo.toml exclude list:",
                        to_add.len()
                    );
                }
                for item in to_add {
                    println!("  + {}", item);
                }
                if report.already_excluded.is_empty() && to_add.is_empty() {
                    println!("No changes needed — all artefacts are already excluded.");
                }
            }
        }
    }
}
