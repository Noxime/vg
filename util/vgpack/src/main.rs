use std::env::args;

fn pack(folder: &str, target: &str) -> Result<(), Box<std::error::Error>> {
    let filename = if target.contains('.') {
        target.to_string()
    } else {
        format!("{}.vgpack", target)
    };

    println!("Packing `{}` into `{}`", folder, filename);

    // bootstrap vg env var
    std::env::set_var("OUT_DIR", ".");
    vg::assets::generate_asset_pack(folder, &filename);

    println!("Done ({})", b(std::fs::metadata(filename)?.len()));
    Ok(())
}

fn b(v: u64) -> String {
    let mut v = v as f64;
    let mut i = 0;
    while v > 1024.0 && i < 4 {
        v /= 1024.0;
        i += 1;
    }

    format!("{:.2} {}", v, vec!["B", "KiB", "MiB", "GiB"][i])
}

fn unpack(source: &str, target: &str) -> Result<(), Box<std::error::Error>> {
    let bytes = std::fs::read(source)?;
    use vg::assets::Assets;
    fn recurse(assets: Assets, path: &str) -> Result<(), Box<std::error::Error>> {
        let path = &format!("{}/{}", path, assets.name());
        println!("Unpacking `{}` into `{}`", assets.name(), path);
        std::fs::create_dir_all(path)?;

        for assets in assets.all_assets() {
            recurse(assets, path)?;
        }

        for (name, bin) in assets.all_binaries() {
            let path = format!("{}/{}", path, name);
            println!("Wrote {} ({})", path, b(bin.len() as _));
            std::fs::write(path, bin)?;
        }
        Ok(())
    }

    recurse(vg::assets::Assets { data: &bytes }, target)
}

fn run(op: &str, source: &str, target: &str) -> Result<(), String> {
    match op {
        "pack" => pack(source, target).map_err(|e| format!("{}", e)),
        "unpack" => unpack(source, target).map_err(|e| format!("{}", e)),
        _ => Err(format!("Unknown command `{}`", op))
    }
}

fn main() {
    let args: Vec<String> = args().collect();
    if let Err(e) = match args.len() {
        3 => run(&args[1], &args[2], "package"),
        4 => run(&args[1], &args[2], &args[3]),
        _ => {
            println!(
                "{} v{}
{}

{}

USAGE:
\t{} <COMMAND> <source> [destination]

EXAMPLE:
\t{} pack assets/

COMMANDS:
\tpack\tPack a source directory into a vgpack
\tunpack \tUnpack a vgpack to a directory
",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                env!("CARGO_PKG_DESCRIPTION"),
                env!("CARGO_PKG_AUTHORS"),
                args[0],
                args[0]
            );
            std::process::exit(1);
        }
    } {
        println!("Error: {}", e);
        std::process::exit(1);
    }
}
