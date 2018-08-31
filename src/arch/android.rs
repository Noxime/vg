extern crate android_glue;
extern crate png;

use self::android_glue::{Event, SyncEventHandler};

pub fn init() {
    log!("INIT android");
    android_glue::add_sync_event_handler(Box::new(Handler));
}

use std::fmt::Arguments;
pub fn stdout(s: Arguments) {
    android_glue::write_log(&format!("{}", s));
}

pub fn load_string(path: &str) -> Option<String> {
    match android_glue::load_asset(path).map(|v| String::from_utf8(v)) {
        Ok(Ok(s)) => Some(s),
        _ => {
            log!("asset loading failed for {}", path);
            None
        }
    }
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
    let data_vec = android_glue::load_asset(path).map_err(|_| ())?;
    let decoder = png::Decoder::new(&data_vec[..]);
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

struct Handler;
impl SyncEventHandler for Handler {
    fn handle(&mut self, event: &Event) {
        log!("EVENTS: Event: {:#?}", event);
    }
}
