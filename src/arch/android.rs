extern crate android_glue;

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
        },
    }
}

struct Handler;
impl SyncEventHandler for Handler {
    fn handle(&mut self, event: &Event) {
        log!("Event: {:#?}", event);
    }
}
