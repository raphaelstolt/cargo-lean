use std::path::Path;

/// Parse a `.gitignore` file and return a list of normalized ignore patterns.
///
/// Returns an empty vector if the file does not exist or cannot be read.
/// Skips blank lines, comments (lines starting with `#`), and negation patterns (starting with `!`).
/// Normalizes patterns by stripping leading `/` (which anchors to the repo root) and trailing `/`
/// (which indicates a directory-only match).
pub fn read_gitignore(manifest_dir: &Path) -> Vec<String> {
    let gitignore_path = manifest_dir.join(".gitignore");
    let content = match std::fs::read_to_string(&gitignore_path) {
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
