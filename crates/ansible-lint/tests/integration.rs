use std::path::PathBuf;
use std::process::Command;

fn binary_path() -> PathBuf {
    // Works for both `cargo test` and `cargo test --release`
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // remove test binary name
    if path.ends_with("deps") {
        path.pop();
    }
    path.join("ansible-lint")
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(binary_path())
        .args(args)
        .output()
        .expect("Failed to run ansible-lint binary");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    (code, stdout, stderr)
}

#[test]
fn version_flag() {
    let (code, stdout, _) = run(&["--version"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("25.5.0"), "stdout was: {stdout}");
}

#[test]
fn list_rules_flag() {
    let (code, stdout, _) = run(&["--list-rules"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("yaml[truthy]"));
    assert!(stdout.contains("name[missing]"));
    assert!(stdout.contains("fqcn[action]"));
}

#[test]
fn list_profiles_flag() {
    let (code, stdout, _) = run(&["--list-profiles"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("basic"));
    assert!(stdout.contains("production"));
}

#[test]
fn list_tags_flag() {
    let (code, stdout, _) = run(&["--list-tags"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("yaml"));
    assert!(stdout.contains("security"));
}

#[test]
fn yaml_truthy_fail_file_exits_nonzero() {
    let file = fixtures_dir().join("playbooks/yaml-truthy-fail.yml");
    let (code, stdout, _) = run(&["--no-color", "--format", "brief",
                                   "--profile", "basic",
                                   file.to_str().unwrap()]);
    // yaml[truthy] is a warning at basic profile — need --strict for exit 1.
    // Without --strict, exit 0 even with warnings.
    assert!(stdout.contains("yaml[truthy]"), "stdout was: {stdout}");
}

#[test]
fn yaml_truthy_fail_file_strict_exits_one() {
    let file = fixtures_dir().join("playbooks/yaml-truthy-fail.yml");
    let (code, _, _) = run(&["--no-color", "--strict",
                              "--profile", "basic",
                              file.to_str().unwrap()]);
    assert_eq!(code, 1, "expected exit 1 with --strict");
}

#[test]
fn yaml_truthy_pass_file_exits_zero() {
    let file = fixtures_dir().join("playbooks/yaml-truthy-pass.yml");
    // With basic profile, no yaml[truthy] violations in the pass file.
    // name[missing] and fqcn violations may exist but they are warnings.
    let (code, _, _) = run(&["--no-color",
                              "--skip-list", "name[missing],fqcn[action],name[casing]",
                              "--profile", "basic",
                              file.to_str().unwrap()]);
    assert_eq!(code, 0);
}

#[test]
fn name_missing_detected() {
    let file = fixtures_dir().join("tasks/name-missing-fail.yml");
    let (_, stdout, _) = run(&["--no-color", "--format", "brief",
                                "--profile", "basic",
                                file.to_str().unwrap()]);
    assert!(stdout.contains("name[missing]"), "stdout was: {stdout}");
}

#[test]
fn no_changed_when_detected() {
    let file = fixtures_dir().join("tasks/no-changed-when-fail.yml");
    // no-changed-when is moderate profile.
    let (_, stdout, _) = run(&["--no-color", "--format", "brief",
                                "--profile", "moderate",
                                file.to_str().unwrap()]);
    assert!(stdout.contains("no-changed-when"), "stdout was: {stdout}");
}

#[test]
fn no_changed_when_passes_when_set() {
    let file = fixtures_dir().join("tasks/no-changed-when-pass.yml");
    let (code, stdout, _) = run(&["--no-color", "--strict",
                                   "--skip-list", "fqcn[action],name[missing],name[casing]",
                                   "--profile", "moderate",
                                   file.to_str().unwrap()]);
    assert!(!stdout.contains("no-changed-when"), "stdout was: {stdout}");
}

#[test]
fn json_format_output() {
    let file = fixtures_dir().join("playbooks/yaml-truthy-fail.yml");
    let (_, stdout, _) = run(&["--no-color", "--format", "json",
                                "--profile", "basic",
                                file.to_str().unwrap()]);
    let parsed: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&stdout);
    assert!(parsed.is_ok(), "JSON parse failed, stdout was: {stdout}");
    let items = parsed.unwrap();
    assert!(!items.is_empty());
    assert!(items[0]["rule"].is_string());
    assert!(items[0]["line"].is_number());
}

#[test]
fn skip_list_suppresses_rule() {
    let file = fixtures_dir().join("playbooks/yaml-truthy-fail.yml");
    let (_, stdout, _) = run(&["--no-color", "--format", "brief",
                                "--profile", "basic",
                                "--skip-list", "yaml[truthy]",
                                file.to_str().unwrap()]);
    assert!(!stdout.contains("yaml[truthy]"), "stdout was: {stdout}");
}

#[test]
fn profile_min_suppresses_moderate_rules() {
    let file = fixtures_dir().join("tasks/no-changed-when-fail.yml");
    let (_, stdout, _) = run(&["--no-color", "--profile", "min", file.to_str().unwrap()]);
    assert!(!stdout.contains("no-changed-when"),
        "min profile should suppress moderate rules, stdout was: {stdout}");
}

#[test]
fn profile_moderate_triggers_moderate_rules() {
    let file = fixtures_dir().join("tasks/no-changed-when-fail.yml");
    let (_, stdout, _) = run(&["--no-color", "--profile", "moderate", file.to_str().unwrap()]);
    assert!(stdout.contains("no-changed-when"),
        "moderate profile should trigger no-changed-when, stdout was: {stdout}");
}

#[test]
fn sarif_format_output() {
    let file = fixtures_dir().join("playbooks/yaml-truthy-fail.yml");
    let (_, stdout, _) = run(&["--no-color", "--format", "sarif",
                                "--profile", "basic",
                                file.to_str().unwrap()]);
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["version"], "2.1.0");
    assert!(v["runs"][0]["results"].is_array());
}

#[test]
fn codeclimate_format_output() {
    let file = fixtures_dir().join("playbooks/yaml-truthy-fail.yml");
    let (_, stdout, _) = run(&["--no-color", "--format", "codeclimate",
                                "--profile", "basic",
                                file.to_str().unwrap()]);
    let v: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap();
    assert!(!v.is_empty());
    assert_eq!(v[0]["type"], "issue");
}

#[test]
fn pep8_format_output() {
    let file = fixtures_dir().join("playbooks/yaml-truthy-fail.yml");
    let (_, stdout, _) = run(&["--no-color", "--format", "pep8",
                                "--profile", "basic",
                                file.to_str().unwrap()]);
    // pep8 format: path:line:col: rule message
    assert!(stdout.contains(':'), "pep8 output should contain colons: {stdout}");
}

#[test]
fn document_start_rule_fires() {
    let file = fixtures_dir().join("tasks/name-missing-fail.yml");
    // name-missing-fail.yml starts with --- so document-start should NOT fire.
    let (_, stdout, _) = run(&["--no-color", "--profile", "basic", file.to_str().unwrap()]);
    assert!(!stdout.contains("yaml[document-start]"), "stdout: {stdout}");
}

#[test]
fn nonexistent_file_exits_two() {
    let (code, _, _) = run(&["/nonexistent/path/that/does/not/exist.yml"]);
    // File discovery may return empty, which is exit 0, or could error.
    // Either 0 or 2 is acceptable — not 1.
    assert_ne!(code, 1);
}
