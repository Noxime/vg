extern crate png;

use std::fmt::Arguments;
use std::fs::{read_to_string, File};
use std::path::Path;


pub fn init() {
    log!("INIT desktop");
}

pub fn stdout(s: Arguments) {
    print!("{}", s);
}

#[allow(dead_code)]
pub fn load_string(path: &str) -> Option<String> {
    read_to_string(Path::new("assets").join(path))
        .map_err(|_| log!("File {} not found", path))
        .ok()
}

fn _bitdepth(v: png::BitDepth) -> &'static str {
    match v {
        png::BitDepth::One => "1",
        png::BitDepth::Two => "2",
        png::BitDepth::Four => "4",
        png::BitDepth::Eight => "8",
        png::BitDepth::Sixteen => "16",
    }
}

fn _colortype(v: png::ColorType) -> &'static str {
    match v {
        png::ColorType::Grayscale => "Grayscale",
        png::ColorType::RGB => "RGB",
        png::ColorType::Indexed => "Indexed",
        png::ColorType::GrayscaleAlpha => "GrayscaleAlpha",
        png::ColorType::RGBA => "RGBA",
    }
}

#[allow(dead_code)]
pub fn load_png(path: &str) -> Result<(usize, usize, Vec<u8>), ()> {
    log!("Loading png `{}`", path);
    //let decoder = png::Decoder::new(File::open(Path::new("assets").join(path)).map_err(|_| ())?);
    let decoder = png::Decoder::new(&include_bytes!("../../assets/textures/test.png")[..]);
    log!("Reading png info");
    let (info, mut reader) = decoder.read_info().map_err(|_| ())?;

    match (info.color_type, info.bit_depth) {
        (png::ColorType::RGBA, png::BitDepth::Eight) => (),
        v => {
            log!("Error: PNG loading only supported for files with 8 bits per channel and RGBA format! (file `{}` is {} bits and format {}",
            path, _bitdepth(v.1), _colortype(v.0));
            return Err(());
        }
    }

    log!(
        "PNG info for `{}`: Size {}x{} {}bit, color: {}",
        path,
        info.width,
        info.height,
        _bitdepth(info.bit_depth),
        _colortype(info.color_type),
    );
    // Allocate the output buffer.
    let mut buf = vec![0; info.buffer_size()];
    // Read the next frame. Currently this function should only called once.
    // The default options
    reader.next_frame(&mut buf).map_err(|_| ())?;

    Ok((info.width as usize, info.height as usize, buf))
}
