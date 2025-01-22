use assert_cmd::Command;
use std::fs;

use tempfile::tempdir;

#[test]
fn test_cli_compare_signatures_identical() {
    let mut cmd = Command::cargo_bin("rrdeep").unwrap();
    cmd.args(&["compare", "ABCDEF:ABCDEF:4", "ABCDEF:ABCDEF:4"]);
    cmd.assert().stdout(predicates::str::contains("Score: 100"));
}

#[test]
fn test_cli_compare_files_identical() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("file1.bin");
    let file2 = dir.path().join("file2.bin");

    fs::write(&file1, b"Hello rrdeep").unwrap();
    fs::write(&file2, b"Hello rrdeep").unwrap();

    let mut cmd = Command::cargo_bin("rrdeep").unwrap();
    cmd.args(&["compare-files", file1.to_str().unwrap(), file2.to_str().unwrap()]);
    cmd.assert().stdout(predicates::str::contains("Score: 100"));
}

#[test]
fn test_cli_compare_files_different() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("file1.bin");
    let file2 = dir.path().join("file2.bin");

    fs::write(&file1, b"File1 data").unwrap();
    fs::write(&file2, b"Completely different").unwrap();

    let mut cmd = Command::cargo_bin("rrdeep").unwrap();
    cmd.args(&["compare-files", file1.to_str().unwrap(), file2.to_str().unwrap()]);
    cmd.assert().stdout(predicates::str::contains("Score: "));
    cmd.assert().stdout(predicates::str::contains("Result: Different")); // example check
}
