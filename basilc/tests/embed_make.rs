#![cfg(feature = "embed-test")]

use std::env;
use std::fs;
use std::path::PathBuf;

// These tests focus on the embedding and extraction helpers without running the VM.

#[test]
fn list_contains_some_known_entries() {
    // The build script populates EMBEDDED_FILES; ensure at least one website file exists.
    let files: Vec<&'static str> = basic::embedded::list_all_paths().collect();
    assert!(files.iter().any(|p| p.ends_with("upgrade.bas")), "missing upgrade.bas in embedded files");
}

#[test]
fn write_single_file_to_temp_dir() {
    let tmp = tempfile::tempdir().unwrap();
    // Avoid running after make in case we touch main's handler directly later
    env::set_var("BASIL_SKIP_RUN_AFTER_MAKE", "1");

    // Use helper directly
    let out = basic::embedded::write_single("upgrade", tmp.path()).expect("write single file");
    assert!(out.exists(), "output file should exist");

    // Verify bytes match embedded table
    let embedded = basic::embedded::find_file("upgrade.bas").unwrap();
    let on_disk = fs::read(&out).unwrap();
    assert_eq!(embedded.contents, on_disk.as_slice(), "bytes must match exactly");
}

#[test]
fn write_directory_to_temp_dir() {
    let tmp = tempfile::tempdir().unwrap();
    basic::embedded::extract_dir("website", tmp.path()).expect("extract dir");

    // Check a couple of files exist
    let p1 = tmp.path().join("website").join("css").join("site.css");
    let p2 = tmp.path().join("website").join("js").join("site.js");
    assert!(p1.exists(), "expected website/css/site.css");
    assert!(p2.exists(), "expected website/js/site.js");
}

#[test]
fn unsafe_targets_rejected() {
    assert!(basic::embedded::is_unsafe_target("/abs"));
    assert!(basic::embedded::is_unsafe_target("../etc"));
    assert!(basic::embedded::is_unsafe_target("..\\evil"));
}
