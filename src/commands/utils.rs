use std::path::Path;

/// Default patterns for unnecessary repository artefacts that should be excluded.
///
/// Items that are typically in `.gitignore` (like `target/`) should still be listed here,
/// as the commands will check `.gitignore` and skip them if already present there.
pub const DEFAULT_EXCLUDE_PATTERNS: &[&str] = &[
    ".gitignore",
    ".gitattributes",
    ".github/",
    ".idea/",
    ".vscode/",
    ".cargo/",
    ".config/",
    "tests/",
    "test/",
    "benches/",
    "examples/",
    "docs/",
    "doc/",
    "target/",
    "CHANGELOG.md",
    "changelog.md",
    "CONTRIBUTING.md",
    "contributing.md",
    "Justfile",
    "justfile",
    "Makefile",
    "makefile",
    ".editorconfig",
    ".pre-commit-config.yaml",
    ".markdownlint.yaml",
    ".yamllint",
    ".dockerignore",
    "Dockerfile",
    "docker-compose.yml",
    "docker-compose.yaml",
];

/// Normalize a pattern by stripping trailing slashes.
///
/// Users may specify directory patterns with a trailing slash (e.g., `-p target/`),
/// so we normalize them to match the format used in Cargo.toml exclude lists.
pub fn normalize_pattern(pattern: &str) -> String {
    pattern.trim_end_matches('/').to_string()
}

/// Check if a pattern corresponds to an existing file or directory in the repo.
///
/// This handles both file patterns (e.g., `CHANGELOG.md`) and directory patterns
/// (e.g., `tests/`, `benches/`). Trailing slashes are stripped before checking
/// because `Path::exists()` may not handle them consistently.
pub fn exists_in_repo(manifest_dir: &Path, pattern: &str) -> bool {
    let normalized = pattern.trim_end_matches('/');
    let path = manifest_dir.join(normalized);
    path.exists()
}

/// Get the current exclude list from Cargo.toml
pub fn get_current_excludes(cargo_toml: &toml::Value) -> Vec<String> {
    let mut excludes = Vec::new();

    if let Some(package) = cargo_toml.get("package")
        && let Some(exclude) = package.get("exclude")
    {
        if let Some(arr) = exclude.as_array() {
            for item in arr {
                if let Some(s) = item.as_str() {
                    excludes.push(s.to_string());
                }
            }
        } else if let Some(s) = exclude.as_str() {
            excludes.push(s.to_string());
        }
    }

    excludes
}

/// Check if a pattern is already excluded in the current Cargo.toml
/// Uses case-insensitive comparison to handle filesystems where casing differs
/// between the exclude list and actual file names.
pub fn is_excluded(pattern: &str, current_excludes: &[String]) -> bool {
    current_excludes.iter().any(|excluded| {
        // Case-insensitive direct match
        excluded.eq_ignore_ascii_case(pattern)
            // If the excluded pattern ends with /, it matches directories
            || (excluded.ends_with('/') && pattern.eq_ignore_ascii_case(excluded.trim_end_matches('/')))
            // If our pattern ends with / (directory), match against exclude without slash
            || (pattern.ends_with('/') && pattern.trim_end_matches('/').eq_ignore_ascii_case(excluded))
            // Check if the pattern matches a glob pattern in excludes
            || matches_glob(pattern, excluded)
    })
}

/// Simple glob matching for common patterns
pub fn matches_glob(pattern: &str, glob_pattern: &str) -> bool {
    if let Ok(g) = glob::Pattern::new(glob_pattern) {
        g.matches(pattern)
    } else {
        false
    }
}

/// Update the exclude list in the Cargo.toml content
pub fn update_cargo_toml_excludes(content: &str, excludes: &[String]) -> anyhow::Result<String> {
    let lines: Vec<&str> = content.lines().collect();

    // Find the exclude key range in [package] section
    let mut exclude_start: Option<usize> = None;
    let mut exclude_end: Option<usize> = None;
    let mut in_package = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == "[package]" {
            in_package = true;
        } else if trimmed.starts_with('[') && in_package {
            in_package = false;
        }

        if in_package && trimmed.starts_with("exclude") && trimmed.contains('=') {
            exclude_start = Some(i);
            if trimmed.contains('[') && trimmed.contains(']') {
                exclude_end = Some(i);
            }
        }

        if exclude_start.is_some() && exclude_end.is_none() && trimmed == "]" {
            exclude_end = Some(i);
        }
    }

    // Build the new content
    let mut result = String::new();
    let mut exclude_replaced = false;

    for (i, line) in lines.iter().enumerate() {
        // Skip old exclude lines
        if let (Some(start), Some(end)) = (exclude_start, exclude_end)
            && i >= start
            && i <= end
        {
            if !exclude_replaced {
                // Insert new exclude at this position
                insert_excludes(&mut result, excludes);
                exclude_replaced = true;
            }
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    // If no exclude existed, insert at the end of [package] section
    if !exclude_replaced && let Some(pkg_pos) = lines.iter().position(|l| l.trim() == "[package]") {
        let mut end_pos = pkg_pos + 1;
        while end_pos < lines.len() && !lines[end_pos].trim().starts_with('[') {
            end_pos += 1;
        }
        // end_pos now points to the next section header or lines.len()
        // Insert the exclude before that position
        let mut new_result = String::new();
        for (i, line) in lines.iter().enumerate() {
            if i == end_pos {
                insert_excludes(&mut new_result, excludes);
            }
            new_result.push_str(line);
            new_result.push('\n');
        }
        // If end_pos == lines.len(), append at the end
        if end_pos == lines.len() {
            insert_excludes(&mut new_result, excludes);
        }
        return Ok(new_result);
    }

    Ok(result)
}

fn insert_excludes(result: &mut String, excludes: &[String]) {
    if excludes.is_empty() {
        return;
    }
    if excludes.len() == 1 {
        result.push_str("exclude = \"");
        result.push_str(&excludes[0]);
        result.push_str("\"\n");
    } else {
        result.push_str("exclude = [\n");
        for exclude in excludes {
            result.push_str("    \"");
            result.push_str(exclude);
            result.push_str("\",\n");
        }
        result.push_str("]\n");
    }
}
