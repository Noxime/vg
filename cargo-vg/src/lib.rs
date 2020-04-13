use std::error::Error;
use std::process::Command;

#[derive(Copy, Clone, Debug)]
enum Platform {
    Desktop,
}

fn fetch(plat: Platform) -> Result<(), Box<dyn Error>> {
    let plat_str = match plat {
        Platform::Desktop => "target-desktop",
    };

    let url = format!(
        "https://owo.codes/noxim/vg/-/archive/current/vg-current.tar.gz?path={}",
        plat_str
    );

    println!("Downloading source for {:?}", plat);
    let mut target = std::fs::File::create("target.tar.gz").unwrap();
    let mut req = reqwest::get(&url)?;
    let count = req.copy_to(&mut target).unwrap();
    println!("\tDone ({}kB)", count / 1024);

    println!("Extracting");
    let _ = Command::new("tar")
        .arg("-xf")
        .arg("target.tar.gz")
        .output()
        .unwrap();
    let _ = Command::new("mv")
        .arg(format!("vg-current-{}/{}", plat_str, plat_str))
        .arg(".")
        .output()
        .unwrap();

    std::fs::rename(plat_str, "target-target").unwrap();
    println!("\tDone");

    Ok(())
}

fn change_dep() {

    let path = "../";
    let name = "teet";

    // game = { path = "../game", package = "game" 
    let new_toml = std::fs::read_to_string("target-target/Cargo.toml")
        .unwrap()
        .replace(
            r#"game = { path = "../game", package = "game" }"#,
            &format!(r#"game = {{ path = "{}", package = "{}" }}"#, path, name),
        );

    dbg!(&new_toml);
    std::fs::write("target-target/Cargo.toml", new_toml).unwrap();

}

pub fn build(release: bool) {
    println!("CARGO VG with release {}", release);
    fetch(Platform::Desktop).expect("failed to fetch");
    change_dep();

    std::env::set_current_dir("target-target");
    println!("{:?}", Command::new("make")
        .arg("build")
        .output());
}
