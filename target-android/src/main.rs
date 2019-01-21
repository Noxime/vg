extern crate kea;
extern crate game;
extern crate kea_dev;
extern crate android_glue;

struct Api;
impl kea::PlatformApi for Api {
    fn print(&self, s: &str) {
        println!("{}", s);
    }
}

struct AndroidHandler {}

impl android_glue::SyncEventHandler for AndroidHandler {
    fn handle(&mut self, event: &android_glue::Event) {
        println!("{:#?}", event);
        match event {
            android_glue::Event::LostFocus => {
                println!("FOCUS LOST: Save game state");
                // TODO: ask game for its state and save it, and set reload flag
                std::process::exit(0);
            },
            _ => (),
        }
    }
}

fn main() {  
    android_glue::add_sync_event_handler(Box::new(AndroidHandler {}));
    kea::run(Api, kea_dev::Renderer::new(), &game::game);
}
