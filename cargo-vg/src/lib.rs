use std::{
    path::{Path, PathBuf},
    rc::Rc,
    sync::mpsc::TryRecvError,
};
use structopt::StructOpt;
use vg_native::runtime::wasm::Wasm;

#[derive(Debug, StructOpt)]
#[structopt(bin_name = "cargo")]
pub enum Args {
    #[structopt(name = "vg")]
    Vg(Opts),
}

#[derive(Debug, Clone, StructOpt)]
pub struct Opts {
    #[structopt(default_value = "Cargo.toml")]
    pub manifest_path: PathBuf,
    #[structopt()]
    pub build_path: Option<PathBuf>,
    #[structopt(subcommand)]
    pub cmd: Option<Cmd>,
}

#[derive(Debug, Clone, StructOpt)]
pub enum Cmd {
    /// Build the project and launch it
    Run,
    /// Run the project on every file change
    Watch,
    /// Build the project
    Build,
    /// Clean the project build files
    Clean,
}

fn run_cargo(manifest: impl AsRef<Path>, build: Option<PathBuf>, step: impl AsRef<str>) -> bool {
    let mut cmd = std::process::Command::new("cargo");

    cmd.arg(step.as_ref())
        .arg("--manifest-path")
        .arg(manifest.as_ref())
        .arg("--target")
        .arg("wasm32-unknown-unknown");

    if let Some(path) = build {
        cmd.arg("--target-dir").arg(path);
    }

    cmd.status().unwrap().success()
}

fn read_wasm() -> Vec<u8> {
    std::fs::read("target/wasm32-unknown-unknown/debug/rust-test.wasm").unwrap()
}

pub fn run(opts: Opts) {
    // let existing = std::env::var("RUSTFLAGS").unwrap_or_default();
    // std::env::set_var("RUSTFLAGS", "-C link-arg=--import-memory");

    match opts.cmd {
        None | Some(Cmd::Run) => {
            if run_cargo(&opts.manifest_path, opts.build_path, "build") {
                println!("Running project");
                let mut wasm = Some(read_wasm());
                vg_native::Engine::run::<Wasm, _>(move || wasm.take());
            }
        }
        Some(Cmd::Watch) => {
            println!("Watching project for changes");

            use notify::{watcher, RecursiveMode, Watcher};
            use std::sync::mpsc::channel;
            use std::time::Duration;

            let (tx, rx) = channel();
            let rx = Rc::new(rx);

            let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();

            watcher
                .watch(&opts.manifest_path, RecursiveMode::Recursive)
                .unwrap();
            watcher
                .watch(
                    &opts.manifest_path.with_file_name("src"),
                    RecursiveMode::Recursive,
                )
                .unwrap();

            vg_native::Engine::run::<Wasm, _>(move || match rx.try_recv() {
                Ok(_) => {
                    assert!(run_cargo(
                        &opts.manifest_path,
                        opts.build_path.clone(),
                        "build"
                    ));
                    Some(read_wasm())
                }
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => panic!("File notification channel closed"),
            })
        }
        Some(Cmd::Build) => {
            println!("Building project");
            run_cargo(opts.manifest_path, opts.build_path, "build");
        }
        Some(Cmd::Clean) => {
            println!("Cleaning project");
            run_cargo(opts.manifest_path, opts.build_path, "clean");
        }
    }
}
