use anyhow::{Context, Result};
use clap::Args;
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::commands::{OutputFormat, gitignore, utils};

#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// Path to the Cargo.toml file
    #[arg(short, long, default_value = "Cargo.toml")]
    pub manifest_path: PathBuf,

    /// Output format (text or json)
    #[arg(long, default_value = "text")]
    pub format: OutputFormat,

    /// Custom exclude patterns (can be specified multiple times)
    #[arg(short = 'p', long)]
    pub patterns: Vec<String>,

    /// Dry run: show what would be changed without modifying the file
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Serialize, Debug)]
struct UpdateReport {
    /// Whether the operation was successful
    success: bool,
    /// Artefacts that will be in the new exclude list
    new_excludes: Vec<String>,
    /// Artefacts removed from the exclude list
    removed: Vec<String>,
    /// Artefacts added to the exclude list
    added: Vec<String>,
    /// Artefacts already covered by .gitignore (skipped)
    gitignored: Vec<String>,
    /// Whether this was a dry run
    dry_run: bool,
}

impl UpdateArgs {
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

        // Find patterns that exist in the repo and are not gitignored
        let new_excludes: Vec<String> = patterns
            .iter()
            .filter(|pattern| {
                utils::exists_in_repo(manifest_dir, pattern.as_str())
                    && !gitignore::is_gitignored(pattern, &gitignore_entries)
            })
            .map(|s| s.to_string())
            .collect();

        // Find patterns covered only by .gitignore
        let gitignored: Vec<String> = patterns
            .iter()
            .filter(|pattern| {
                utils::exists_in_repo(manifest_dir, pattern.as_str())
                    && gitignore::is_gitignored(pattern, &gitignore_entries)
            })
            .map(|s| s.to_string())
            .collect();

        // Calculate diff
        let added: Vec<String> = new_excludes
            .iter()
            .filter(|item| !current_excludes.contains(item))
            .map(|s| s.to_string())
            .collect();

        let removed: Vec<String> = current_excludes
            .iter()
            .filter(|item| !new_excludes.contains(*item))
            .map(|s| s.to_string())
            .collect();

        if self.dry_run {
            let report = UpdateReport {
                success: true,
                new_excludes: new_excludes.clone(),
                removed,
                added: added.clone(),
                gitignored: gitignored.clone(),
                dry_run: true,
            };
            self.output_report(&report);
            return Ok(());
        }

        // Update the Cargo.toml
        let new_content = utils::update_cargo_toml_excludes(&cargo_toml_content, &new_excludes)
            .with_context(|| "Failed to update Cargo.toml")?;

        std::fs::write(manifest_path, &new_content)
            .with_context(|| format!("Failed to write {}", manifest_path.display()))?;

        let report = UpdateReport {
            success: true,
            new_excludes: new_excludes.clone(),
            removed,
            added: added.clone(),
            gitignored,
            dry_run: false,
        };
        self.output_report(&report);

        Ok(())
    }

    fn output_report(&self, report: &UpdateReport) {
        match self.format {
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(report).unwrap();
                println!("{}", json);
            }
            OutputFormat::Text => {
                if report.dry_run {
                    println!("Dry run — would update exclude list:");
                } else {
                    println!("Updated Cargo.toml exclude list:");
                }
                println!("  Total excluded: {}", report.new_excludes.len());
                if !report.added.is_empty() {
                    println!("  Added ({}):", report.added.len());
                    for item in &report.added {
                        println!("    + {}", item);
                    }
                }
                if !report.removed.is_empty() {
                    println!("  Removed ({}):", report.removed.len());
                    for item in &report.removed {
                        println!("    - {}", item);
                    }
                }
                if report.added.is_empty() && report.removed.is_empty() {
                    println!("  No changes needed.");
                }
            }
        }
    }
}
