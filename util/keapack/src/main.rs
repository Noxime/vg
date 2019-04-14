use std::env::args;

fn run(folder: &str, target: &str) -> Result<(), Box<std::error::Error>> {

    let filename = if target.contains('.') {
        target.to_string()
    } else {
        format!("{}.keapack", target)
    };

    // bootstrap kea env var
    std::env::set_var("OUT_DIR", ".");
    kea::assets::generate_asset_pack(folder, &filename);

    println!("Created `{}` successfully", filename);
    Ok(())
}

fn main() {
    let args: Vec<String> = args().collect();
    if let Err(e) = match args.len() {
        2 => {
            run(&args[1], "package")
        },
        3 => {
            run(&args[1], &args[2])
        },
        _ => {
            println!(r#"Usage: {} <source path> [keapack name]

Example: {} assets/
"#, args[0], args[0]);
            std::process::exit(1);
        }
    } {
        println!("Error: {}", e);
        std::process::exit(1);
    }

}
