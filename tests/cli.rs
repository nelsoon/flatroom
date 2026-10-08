//! End-to-end tests for `flatroom inspect`. All fixtures are synthetic
//! JSON authored here; no vendor files are used.

use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_flatroom"))
}

/// Fresh scratch dir per test; removed afterwards on a best-effort basis.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "flatroom-{}-{}-{}",
        name,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

fn write(dir: &std::path::Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, bytes).expect("fixture");
    path
}

#[test]
fn human_output_lists_sorted_keys() {
    let dir = scratch("human");
    let file = write(&dir, "a.json", br#"{"b": 1, "a": {"x": 1}}"#);
    let out = Command::new(bin())
        .args(["inspect", &file.to_string_lossy()])
        .output()
        .expect("run");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    assert!(stdout.contains("keys (2): a, b"), "{stdout}");
    assert!(stdout.contains("size: "), "{stdout}");
    assert!(stdout.contains("  a: 1 keys"), "{stdout}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn json_flag_emits_parseable_summary() {
    let dir = scratch("json");
    let file = write(&dir, "a.json", br#"{"b": 1, "a": 2}"#);
    let out = Command::new(bin())
        .args(["inspect", "--json", &file.to_string_lossy()])
        .output()
        .expect("run");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("summary JSON");
    assert_eq!(v["shape"], "object");
    assert_eq!(v["keys"], serde_json::json!(["a", "b"]));
    assert!(v.get("size_bytes").is_some(), "{v}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn non_json_fails_with_reason() {
    let dir = scratch("text");
    let file = write(&dir, "a.txt", b"just some text");
    let out = Command::new(bin())
        .args(["inspect", &file.to_string_lossy()])
        .output()
        .expect("run");
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).expect("utf8");
    assert!(stderr.starts_with("not JSON:"), "{stderr}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn missing_file_reports_no_such_file() {
    let out = Command::new(bin())
        .args(["inspect", "/no/such/file-xyz.json"])
        .output()
        .expect("run");
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).expect("utf8");
    assert!(stderr.contains("no such file"), "{stderr}");
}
