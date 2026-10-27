use anyhow::{Context, Result};
use clap::Args;
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::commands::{OutputFormat, gitignore, utils};

#[derive(Args, Debug)]
pub struct LintArgs {
    /// Path to the Cargo.toml file
    #[arg(short, long, default_value = "Cargo.toml")]
    pub manifest_path: PathBuf,

    /// Output format (text or json)
    #[arg(long, default_value = "text")]
    pub format: OutputFormat,

    /// Custom exclude patterns (can be specified multiple times)
    #[arg(short = 'p', long)]
    pub patterns: Vec<String>,

    /// Show a diff of what would change (like `create` would do)
    #[arg(long)]
    pub diff: bool,
}

#[derive(Serialize, Debug)]
struct LintReport {
    /// Whether all recommended artefacts are excluded
    valid: bool,
    /// Artefacts that should be excluded but are not
    missing: Vec<String>,
    /// Artefacts that are excluded
    excluded: Vec<String>,
    /// Artefacts already covered by .gitignore
    gitignored: Vec<String>,
    /// Total number of recommended patterns checked
    total_patterns: usize,
}

impl LintArgs {
    pub fn run(&self) -> Result<()> {
        let manifest_path = &self.manifest_path;

        // Read and parse Cargo.toml
        let cargo_toml_content = std::fs::read_to_string(manifest_path)
            .with_context(|| format!("Failed to read {}", manifest_path.display()))?;

        let cargo_toml: toml::Value = toml::from_str(&cargo_toml_content)
            .with_context(|| format!("Failed to parse {}", manifest_path.display()))?;

        // Get current exclude list from Cargo.toml
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
        let existing_unexcluded: Vec<String> = patterns
            .iter()
            .filter(|pattern| {
                utils::exists_in_repo(manifest_dir, pattern.as_str())
                    && !utils::is_excluded(pattern, &current_excludes)
                    && !gitignore::is_gitignored(pattern, &gitignore_entries)
            })
            .map(|s| s.to_string())
            .collect();

        // Find patterns that are covered (excluded or gitignored)
        let matched_patterns: Vec<String> = patterns
            .iter()
            .filter(|pattern| {
                utils::is_excluded(pattern, &current_excludes)
                    || gitignore::is_gitignored(pattern, &gitignore_entries)
            })
            .map(|s| s.to_string())
            .collect();

        // Find patterns covered only by .gitignore
        let gitignored_only: Vec<String> = patterns
            .iter()
            .filter(|pattern| {
                !utils::is_excluded(pattern, &current_excludes)
                    && gitignore::is_gitignored(pattern, &gitignore_entries)
            })
            .map(|s| s.to_string())
            .collect();

        let valid = existing_unexcluded.is_empty();

        let report = LintReport {
            valid,
            missing: existing_unexcluded.clone(),
            excluded: matched_patterns,
            gitignored: gitignored_only,
            total_patterns: patterns.len(),
        };

        match self.format {
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(&report)?;
                println!("{}", json);
            }
            OutputFormat::Text => {
                if self.diff {
                    // Diff mode: show what would be added
                    if valid {
                        println!("No changes needed — Cargo.toml exclude list is complete.");
                    } else {
                        println!(
                            "--- Cargo.toml exclude list is missing {} artefact(s):",
                            existing_unexcluded.len()
                        );
                        for item in &existing_unexcluded {
                            println!("+ {}", item);
                        }
                    }
                } else {
                    if valid {
                        println!("✓ Cargo.toml exclude list is complete.");
                    } else {
                        println!(
                            "✗ Found {} artefact(s) that should be excluded but are not:",
                            existing_unexcluded.len()
                        );
                        for item in &existing_unexcluded {
                            println!("  - {}", item);
                        }
                    }
                }
            }
        }

        if !valid {
            std::process::exit(1);
        }

        Ok(())
    }
}
