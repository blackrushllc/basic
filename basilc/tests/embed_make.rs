#![cfg(feature = "embed-test")]

use std::env;
use std::fs;

// These tests focus on the embedding and extraction helpers without running the VM.

fn create_unique_temp_dir(prefix: &str) -> std::path::PathBuf {
    let mut p = env::temp_dir();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let pid = std::process::id();
    p.push(format!("{}_{}_{}", prefix, pid, nanos));
    fs::create_dir_all(&p).expect("create temp dir");
    p
}

#[test]
fn list_contains_some_known_entries() {
    // The build script populates EMBEDDED_FILES; ensure at least one website file exists.
    let files: Vec<&'static str> = basic::embedded::list_all_paths().collect();
    assert!(files.iter().any(|p| p.ends_with("upgrade.bas")), "missing upgrade.bas in embedded files");
}

#[test]
fn write_single_file_to_temp_dir() {
    let tmp = create_unique_temp_dir("embed_single");
    // Avoid running after make in case we touch main's handler directly later
    env::set_var("BASIL_SKIP_RUN_AFTER_MAKE", "1");

    // Use helper directly
    let out = basic::embedded::write_single("upgrade", &tmp).expect("write single file");
    assert!(out.exists(), "output file should exist");

    // Verify bytes match embedded table
    let embedded = basic::embedded::find_file("upgrade.bas").unwrap();
    let on_disk = fs::read(&out).unwrap();
    assert_eq!(embedded.contents, on_disk.as_slice(), "bytes must match exactly");

    // Clean up
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn write_directory_to_temp_dir() {
    let tmp = create_unique_temp_dir("embed_dir");
    basic::embedded::extract_dir("website", &tmp).expect("extract dir");

    // Check a couple of files exist
    let p1 = tmp.join("website").join("css").join("site.css");
    let p2 = tmp.join("website").join("js").join("site.js");
    assert!(p1.exists(), "expected website/css/site.css");
    assert!(p2.exists(), "expected website/js/site.js");

    // Clean up
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn unsafe_targets_rejected() {
    assert!(basic::embedded::is_unsafe_target("/abs"));
    assert!(basic::embedded::is_unsafe_target("../etc"));
    assert!(basic::embedded::is_unsafe_target("..\\evil"));
}
