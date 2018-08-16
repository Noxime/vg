use std::fmt::Arguments;

pub fn init() {
    log!("INIT desktop");
}

pub fn stdout(s: Arguments) {
    print!("{}", s);
}

pub fn load_string(path: &str) -> Option<String> {
    use std::fs::read_to_string;
    use std::path::Path;
    read_to_string(Path::new("assets").join(path)).map_err(|_| log!("File {} not found", path)).ok()
}