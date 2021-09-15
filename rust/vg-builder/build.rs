use std::env::var;
use std::path::PathBuf;

const TARGET: &str = "wasm32-wasi";

// Find Cargo.toml in parent directory
fn root_manifest() -> PathBuf {
    let mut path: PathBuf = var("OUT_DIR").unwrap().parse().unwrap();
    while !path.with_file_name("Cargo.toml").exists() && path.pop() {}
    println!("cargo:rerun-if-changed={}", path.display());
    path.with_file_name("Cargo.toml")
}

fn build_wasm() {
    let manifest_path = root_manifest();
    let build_path: PathBuf = var("OUT_DIR").unwrap().parse().unwrap();
    let build_path = build_path.join("magic-build");

    println!("cargo:warning=Summoning Beelzebub to build game files. Demonic build rituals are not well supported, please use cargo-vg instead");

    std::fs::create_dir_all(&build_path).unwrap();

    assert!(std::process::Command::new("cargo")
        .arg("rustc")
        .arg("--target")
        .arg(TARGET)
        .arg("--manifest-path")
        .arg(manifest_path)
        .arg("--target-dir")
        .arg(&build_path)
        .arg("--")
        .arg("-C")
        .arg("opt-level=3")
        // .arg("--")
        // .arg("-C")
        // .arg("link-arg=--import-memory")
        .status()
        .unwrap()
        .success());

    let mut out_file = build_path.join(TARGET).join("debug");

    for file in out_file.read_dir().unwrap() {
        let file = file.unwrap();
        if file.file_name().to_string_lossy().ends_with(".wasm") {
            out_file = file.path();
            break;
        }
    }

    println!("Produced {}", out_file.display());

    std::fs::copy(out_file, build_path.join("out.wasm")).unwrap();
}

fn main() {
    if std::env::var("TARGET").unwrap() != TARGET {
        build_wasm()
    }
}
