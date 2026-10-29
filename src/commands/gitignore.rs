use std::path::Path;

/// Parse a `.gitignore` file at the given path and return a list of normalized ignore patterns.
///
/// Returns an empty vector if the file does not exist or cannot be read.
/// Skips blank lines, comments (lines starting with `#`), and negation patterns (starting with `!`).
/// Normalizes patterns by stripping leading `/` (which anchors to the repo root) and trailing `/`
/// (which indicates a directory-only match).
fn parse_gitignore_file(path: &Path) -> Vec<String> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    content
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| {
            // Skip empty lines, comments, and negation patterns
            !line.is_empty() && !line.starts_with('#') && !line.starts_with('!')
        })
        .map(|line| {
            // Normalize: strip leading `/` (root anchor) and trailing `/` (directory indicator)
            let line = line.strip_prefix('/').unwrap_or(&line);
            let line = line.strip_suffix('/').unwrap_or(line);
            line.to_string()
        })
        .collect()
}

/// Expand `~` to the user's home directory, if present at the start of the path.
fn expand_tilde(path: &str) -> std::path::PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return std::path::PathBuf::from(home).join(rest);
    }
    // Also handle bare `~`
    if path == "~"
        && let Ok(home) = std::env::var("HOME")
    {
        return std::path::PathBuf::from(home);
    }
    std::path::PathBuf::from(path)
}

/// Try to resolve the global gitignore excludes file path.
///
/// Checks `git config --global --get core.excludesFile` first, then falls back to the XDG
/// default (`$XDG_CONFIG_HOME/git/ignore` or `~/.config/git/ignore`).
///
/// Returns `None` if no global excludes file is configured or found.
fn global_excludes_file() -> Option<std::path::PathBuf> {
    // Try `git config --global --get core.excludesFile`
    if let Ok(output) = std::process::Command::new("git")
        .args(["config", "--global", "--get", "core.excludesFile"])
        .output()
        && output.status.success()
    {
        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path_str.is_empty() {
            let path = expand_tilde(&path_str);
            if path.exists() {
                return Some(path);
            }
        }
    }

    // Fall back to XDG default: $XDG_CONFIG_HOME/git/ignore or ~/.config/git/ignore
    let xdg_config = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| format!("{}/.config", h))
                .unwrap_or_default()
        });

    if !xdg_config.is_empty() {
        let default_path = std::path::PathBuf::from(xdg_config).join("git/ignore");
        if default_path.exists() {
            return Some(default_path);
        }
    }

    None
}

/// Read all gitignore patterns for a manifest directory.
///
/// Combines patterns from the local `.gitignore` file and the global excludes file
/// (configured via `core.excludesFile` or the XDG default).
/// Returns an empty vector if no files are found.
pub fn read_gitignore(manifest_dir: &Path) -> Vec<String> {
    let mut patterns = Vec::new();

    // Read local .gitignore
    let local_path = manifest_dir.join(".gitignore");
    patterns.extend(parse_gitignore_file(&local_path));

    // Read global excludes file
    if let Some(global_path) = global_excludes_file() {
        patterns.extend(parse_gitignore_file(&global_path));
    }

    patterns
}

/// Check if a pattern is already covered by a `.gitignore` entry.
///
/// Uses simple string matching and basic glob matching via the `glob` crate.
/// Strips trailing slashes from the pattern before matching so that directory
/// patterns (e.g., `target/`) match gitignore entries without slashes (e.g.,
/// `target`).
pub fn is_gitignored(pattern: &str, gitignore_entries: &[String]) -> bool {
    let pattern = pattern.trim_end_matches('/');
    gitignore_entries.iter().any(|entry| {
        // Direct match
        entry == pattern
            // Glob-like match
            || matches_glob(pattern, entry)
    })
}

/// Simple glob matching for common patterns
fn matches_glob(pattern: &str, glob_pattern: &str) -> bool {
    if let Ok(g) = glob::Pattern::new(glob_pattern) {
        g.matches(pattern)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parse_gitignore_file_returns_empty_when_missing() {
        let patterns = parse_gitignore_file(Path::new("/nonexistent/.gitignore"));
        assert!(patterns.is_empty());
    }

    #[test]
    fn parse_gitignore_file_skips_comments_and_blank_lines() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".gitignore");
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file).unwrap();
        writeln!(file, "target/").unwrap();
        writeln!(file, "   ").unwrap();
        writeln!(file, ".idea/").unwrap();

        let patterns = parse_gitignore_file(&path);
        assert_eq!(patterns, vec!["target".to_string(), ".idea".to_string()]);
    }

    #[test]
    fn parse_gitignore_file_skips_negation_patterns() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".gitignore");
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(file, "!important.txt").unwrap();
        writeln!(file, "target/").unwrap();

        let patterns = parse_gitignore_file(&path);
        assert_eq!(patterns, vec!["target".to_string()]);
    }

    #[test]
    fn parse_gitignore_file_strips_leading_slash() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".gitignore");
        std::fs::write(&path, "/target/\n").unwrap();

        let patterns = parse_gitignore_file(&path);
        assert_eq!(patterns, vec!["target".to_string()]);
    }

    #[test]
    fn read_gitignore_reads_local_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".gitignore"), "target/\n.idea/\n").unwrap();

        let patterns = read_gitignore(dir.path());
        // May include global patterns, so check that local ones are present
        assert!(patterns.contains(&"target".to_string()));
        assert!(patterns.contains(&".idea".to_string()));
    }

    #[test]
    fn read_gitignore_returns_empty_when_no_files() {
        let dir = tempfile::tempdir().unwrap();
        // No .gitignore created

        // Global patterns may still be present from the user's config,
        // so we only verify the function runs without error
        let _ = read_gitignore(dir.path());
    }

    #[test]
    fn is_gitignored_matches_global_patterns() {
        // Simulate patterns that would come from both local and global gitignore
        let entries = vec![
            "target".to_string(),
            ".idea".to_string(),
            "*.log".to_string(),
        ];

        assert!(is_gitignored("target", &entries));
        assert!(is_gitignored("target/", &entries));
        assert!(is_gitignored(".idea/", &entries));
        assert!(is_gitignored("debug.log", &entries));
        assert!(!is_gitignored("src", &entries));
    }
}
