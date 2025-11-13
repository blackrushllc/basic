use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let includes_root = manifest_dir.join("includes");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest = out_dir.join("embedded_includes.rs");

    // If there is no includes/ folder, emit an empty table (no error).
    let mut entries: Vec<String> = Vec::new();
    if includes_root.exists() {
        collect_files(&includes_root, &includes_root, &mut entries).unwrap();
    }

    let mut f = File::create(&dest).unwrap();
    writeln!(
        f,
        r#"#[derive(Debug, Clone, Copy)]
pub struct EmbeddedFile {{ pub path: &'static str, pub contents: &'static [u8] }}

pub static EMBEDDED_FILES: &[EmbeddedFile] = &["#
    )
    .unwrap();

    // Keep a stable order for deterministic builds
    entries.sort();
    for logical in entries {
        // We generate lines like:
        // EmbeddedFile { path: "examples/hello.bas", contents: include_bytes!("includes/examples/hello.bas") },
        let include_path = format!("includes/{}", logical);
        writeln!(
            f,
            "    EmbeddedFile {{ path: {lp:?}, contents: include_bytes!({ip:?}) }},",
            lp = logical,
            ip = include_path
        )
        .unwrap();
    }

    writeln!(f, "];\n").unwrap();
    println!("cargo:rerun-if-changed=includes");
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<String>) -> std::io::Result<()> {
    for ent in fs::read_dir(dir)? {
        let ent = ent?;
        let p = ent.path();
        let meta = ent.metadata()?;
        if meta.is_dir() {
            collect_files(root, &p, out)?;
        } else if meta.is_file() {
            let rel = p.strip_prefix(root).unwrap();
            // Normalize to forward slashes for a stable logical path
            let logical = rel.to_string_lossy().replace('\\', "/");
            out.push(logical);
        }
    }
    Ok(())
}
