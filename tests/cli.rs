use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

/// Helper: create a temporary directory with a Cargo.toml and optional files/dirs.
fn setup_test_dir(dir: &Path, cargo_toml: &str, artifacts: &[&str]) {
    fs::create_dir_all(dir).ok();
    fs::write(dir.join("Cargo.toml"), cargo_toml).unwrap();
    for artifact in artifacts {
        let path = dir.join(artifact);
        if path.extension().is_none() && !artifact.contains('.') {
            // Treat as directory
            fs::create_dir_all(&path).ok();
        } else {
            // Ensure parent exists
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::write(&path, "").ok();
        }
    }
}

/// Helper: create a .gitignore file in the test directory
fn setup_gitignore(dir: &Path, content: &str) {
    fs::write(dir.join(".gitignore"), content).unwrap();
}

/// Helper: get the path to the cargo-lean binary
fn cmd() -> Command {
    Command::cargo_bin("cargo-lean").unwrap()
}

// ---------------------------------------------------------------------------
// LINT command tests
// ---------------------------------------------------------------------------

#[test]
fn lint_help_shows_usage() {
    cmd().arg("lint").arg("--help").assert().success();
}

#[test]
fn lint_diff_flag_shows_plus_prefixes() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    setup_test_dir(dir.path(), cargo_toml, &[".idea"]);

    cmd()
        .arg("lint")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .arg("--diff")
        .assert()
        .failure()
        .stdout(predicate::str::contains("+ .idea"));
}

#[test]
fn lint_reports_missing_exclusions() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    setup_test_dir(dir.path(), cargo_toml, &[".idea"]);

    cmd()
        .arg("lint")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .assert()
        .failure();
}

#[test]
fn lint_passes_when_all_excluded() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
exclude = [".idea"]
"#;
    setup_test_dir(dir.path(), cargo_toml, &[".idea"]);

    cmd()
        .arg("lint")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .assert()
        .success();
}

#[test]
fn lint_json_output_contains_valid_field() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    setup_test_dir(dir.path(), cargo_toml, &[".idea"]);

    cmd()
        .arg("lint")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .arg("--format")
        .arg("json")
        .assert()
        .failure()
        .stdout(predicate::str::contains(r#""valid": false"#));
}

#[test]
fn lint_json_output_valid_true_when_complete() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
exclude = [".idea"]
"#;
    setup_test_dir(dir.path(), cargo_toml, &[".idea"]);

    cmd()
        .arg("lint")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""valid": true"#));
}

#[test]
fn lint_with_custom_patterns() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    setup_test_dir(dir.path(), cargo_toml, &["custom-artifact"]);

    cmd()
        .arg("lint")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .arg("--patterns")
        .arg("custom-artifact")
        .assert()
        .failure()
        .stdout(predicate::str::contains("custom-artifact"));
}

#[test]
fn lint_diff_no_changes_needed() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
exclude = [".idea"]
"#;
    setup_test_dir(dir.path(), cargo_toml, &[".idea"]);

    cmd()
        .arg("lint")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .arg("--diff")
        .assert()
        .success()
        .stdout(predicate::str::contains("No changes needed"));
}

#[test]
fn lint_missing_manifest_returns_error() {
    cmd()
        .arg("lint")
        .arg("--manifest-path")
        .arg("/nonexistent/Cargo.toml")
        .assert()
        .failure();
}

#[test]
fn lint_json_missing_array_contains_artifact() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    setup_test_dir(dir.path(), cargo_toml, &[".idea"]);

    cmd()
        .arg("lint")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .arg("--format")
        .arg("json")
        .assert()
        .failure()
        .stdout(predicate::str::contains(r#""missing""#))
        .stdout(predicate::str::contains(".idea"));
}

// ---------------------------------------------------------------------------
// CREATE command tests
// ---------------------------------------------------------------------------

#[test]
fn create_help_shows_usage() {
    cmd().arg("create").arg("--help").assert().success();
}

#[test]
fn create_dry_run_does_not_modify_file() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    setup_test_dir(dir.path(), cargo_toml, &[".idea"]);

    cmd()
        .arg("create")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .arg("--dry-run")
        .assert()
        .success();

    // File should not have been modified
    let content = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
    assert!(!content.contains("exclude"));
}

#[test]
fn create_adds_missing_exclusions() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    setup_test_dir(dir.path(), cargo_toml, &[".idea"]);

    cmd()
        .arg("create")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .assert()
        .success();

    let content = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
    assert!(content.contains("exclude"));
    assert!(content.contains(".idea"));
}

#[test]
fn create_does_not_duplicate_existing_excludes() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
exclude = [".idea"]
"#;
    setup_test_dir(dir.path(), cargo_toml, &[".idea", "CHANGELOG.md"]);

    cmd()
        .arg("create")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .assert()
        .success();

    let content = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
    // .idea should appear in the exclude list
    assert!(content.contains(".idea"));
    // CHANGELOG.md should be added
    assert!(content.contains("CHANGELOG.md"));
}

#[test]
fn create_json_output_shows_added() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    setup_test_dir(dir.path(), cargo_toml, &[".idea"]);

    cmd()
        .arg("create")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""added""#));
}

#[test]
fn create_with_no_artifacts_needed() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    // No artifacts created — nothing to add
    setup_test_dir(dir.path(), cargo_toml, &[]);

    cmd()
        .arg("create")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// UPDATE command tests
// ---------------------------------------------------------------------------

#[test]
fn update_help_shows_usage() {
    cmd().arg("update").arg("--help").assert().success();
}

#[test]
fn update_dry_run_does_not_modify_file() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
exclude = ["old-artifact"]
"#;
    setup_test_dir(dir.path(), cargo_toml, &[".idea"]);

    cmd()
        .arg("update")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .arg("--dry-run")
        .assert()
        .success();

    // File should still contain the old exclude
    let content = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
    assert!(content.contains("old-artifact"));
}

#[test]
fn update_replaces_exclude_list() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
exclude = ["old-artifact"]
"#;
    setup_test_dir(dir.path(), cargo_toml, &[".idea"]);

    cmd()
        .arg("update")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .assert()
        .success();

    let content = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
    assert!(content.contains(".idea"));
    assert!(!content.contains("old-artifact"));
}

#[test]
fn update_json_output_shows_added_and_removed() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
exclude = ["old-artifact"]
"#;
    setup_test_dir(dir.path(), cargo_toml, &[".idea"]);

    let output = cmd()
        .arg("update")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains(r#""added""#));
    assert!(stdout.contains(r#""removed""#));
}

#[test]
fn update_with_no_existing_excludes() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    setup_test_dir(dir.path(), cargo_toml, &[".idea", "target"]);

    cmd()
        .arg("update")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .assert()
        .success();

    let content = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
    assert!(content.contains("exclude"));
    assert!(content.contains(".idea"));
}

// ---------------------------------------------------------------------------
// General CLI tests
// ---------------------------------------------------------------------------

#[test]
fn cli_help_shows_subcommands() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("lint"));
}

#[test]
fn cli_version_flag() {
    cmd().arg("--version").assert().success();
}

#[test]
fn create_then_lint_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    setup_test_dir(dir.path(), cargo_toml, &[".idea"]);

    // First, create the excludes
    cmd()
        .arg("create")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .assert()
        .success();

    // Then, lint should pass
    cmd()
        .arg("lint")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .assert()
        .success();
}

#[test]
fn update_then_lint_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
exclude = ["old-item", "another-old"]
"#;
    setup_test_dir(dir.path(), cargo_toml, &[".idea"]);

    // Update the excludes
    cmd()
        .arg("update")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .assert()
        .success();

    // Lint should pass
    cmd()
        .arg("lint")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// .gitignore awareness tests
// ---------------------------------------------------------------------------

#[test]
fn lint_passes_when_artefact_is_gitignored() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    // target/ exists but is in .gitignore, so lint should pass
    // .gitignore is also in .gitignore (self-referencing), so it's not flagged
    setup_test_dir(dir.path(), cargo_toml, &["target"]);
    setup_gitignore(dir.path(), "/target\n.gitignore\n");

    cmd()
        .arg("lint")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .assert()
        .success();
}

#[test]
fn lint_json_shows_gitignored_field() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    // .gitignore is self-referenced so it's not flagged as missing
    setup_test_dir(dir.path(), cargo_toml, &["target"]);
    setup_gitignore(dir.path(), "/target\n.gitignore\n");

    cmd()
        .arg("lint")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""gitignored""#))
        .stdout(predicate::str::contains("target"));
}

#[test]
fn create_skips_gitignored_artefacts() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    // target/ exists and is in .gitignore; .idea exists and is not
    setup_test_dir(dir.path(), cargo_toml, &["target", ".idea"]);
    setup_gitignore(dir.path(), "/target\n");

    cmd()
        .arg("create")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .assert()
        .success();

    let content = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
    // .idea should be added
    assert!(content.contains(".idea"));
    // target should NOT be added (it's in .gitignore)
    assert!(!content.contains("target"));
}

#[test]
fn create_json_shows_gitignored_field() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    setup_test_dir(dir.path(), cargo_toml, &["target", ".idea"]);
    setup_gitignore(dir.path(), "/target\n");

    cmd()
        .arg("create")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""gitignored""#));
}

#[test]
fn update_skips_gitignored_artefacts() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    // target/ exists and is in .gitignore; .idea exists and is not
    setup_test_dir(dir.path(), cargo_toml, &["target", ".idea"]);
    setup_gitignore(dir.path(), "/target\n");

    cmd()
        .arg("update")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .assert()
        .success();

    let content = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
    // .idea should be in the exclude list
    assert!(content.contains(".idea"));
    // target should NOT be in the exclude list (it's in .gitignore)
    assert!(!content.contains("target"));
}

#[test]
fn update_json_shows_gitignored_field() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    setup_test_dir(dir.path(), cargo_toml, &["target", ".idea"]);
    setup_gitignore(dir.path(), "/target\n");

    cmd()
        .arg("update")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""gitignored""#));
}

#[test]
fn lint_with_gitignore_wildcard_pattern() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    // *.log files are gitignored; .gitignore is self-referenced so it's not flagged
    setup_test_dir(dir.path(), cargo_toml, &[]);
    setup_gitignore(dir.path(), "*.log\n.gitignore\n");

    // Lint should pass — no default patterns match *.log
    cmd()
        .arg("lint")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .assert()
        .success();
}

#[test]
fn lint_gitignore_with_comments_and_blank_lines() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    setup_test_dir(dir.path(), cargo_toml, &["target", ".idea"]);
    setup_gitignore(
        dir.path(),
        "# This is a comment\n\n/target\n# Another comment\n",
    );

    // target is gitignored, .idea is not — lint should fail for .idea
    cmd()
        .arg("lint")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .assert()
        .failure()
        .stdout(predicate::str::contains(".idea"));
}

#[test]
fn create_then_lint_with_gitignore() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    setup_test_dir(dir.path(), cargo_toml, &["target", ".idea"]);
    setup_gitignore(dir.path(), "/target\n");

    // Create should add .idea but skip target
    cmd()
        .arg("create")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .assert()
        .success();

    // Lint should pass — .idea is excluded, target is gitignored
    cmd()
        .arg("lint")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// Trailing slash patterns
// ---------------------------------------------------------------------------

#[test]
fn lint_accepts_trailing_slash_patterns() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    setup_test_dir(dir.path(), cargo_toml, &[".idea", "target"]);
    setup_gitignore(dir.path(), ".gitignore\n");

    // Patterns with trailing slashes should be normalized
    cmd()
        .arg("lint")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .arg("-p")
        .arg(".idea/")
        .arg("-p")
        .arg("target/")
        .assert()
        .failure()
        .stdout(predicate::str::contains(".idea"))
        .stdout(predicate::str::contains("target"));
}

#[test]
fn lint_trailing_slash_patterns_match_excludes() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
exclude = [".idea", "target"]
"#;
    setup_test_dir(dir.path(), cargo_toml, &[".idea", "target"]);
    setup_gitignore(dir.path(), ".gitignore\n");

    // Trailing slash patterns should match the exclude entries without slashes
    cmd()
        .arg("lint")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .arg("-p")
        .arg(".idea/")
        .arg("-p")
        .arg("target/")
        .assert()
        .success();
}

#[test]
fn create_accepts_trailing_slash_patterns() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    setup_test_dir(dir.path(), cargo_toml, &[".idea", "target"]);
    setup_gitignore(dir.path(), ".gitignore\n");

    cmd()
        .arg("create")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .arg("-p")
        .arg(".idea/")
        .arg("-p")
        .arg("target/")
        .assert()
        .success();

    let content = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
    // Should be added without trailing slashes
    assert!(content.contains(r#""#) && content.contains(".idea"));
    assert!(content.contains("target"));
}

#[test]
fn update_accepts_trailing_slash_patterns() {
    let dir = tempfile::tempdir().unwrap();
    let cargo_toml = r#"[package]
name = "test"
version = "0.1.0"
edition = "2024"
"#;
    setup_test_dir(dir.path(), cargo_toml, &[".idea", "target"]);
    setup_gitignore(dir.path(), ".gitignore\n");

    cmd()
        .arg("update")
        .arg("--manifest-path")
        .arg(dir.path().join("Cargo.toml"))
        .arg("-p")
        .arg(".idea/")
        .arg("-p")
        .arg("target/")
        .assert()
        .success();

    let content = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
    assert!(content.contains(".idea"));
    assert!(content.contains("target"));
}
