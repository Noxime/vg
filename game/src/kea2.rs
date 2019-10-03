/// Design scratch for kea 2.0

struct Example;

impl Game for Example {
    // Called on start up
    fn new(&mut config: Config, storage: &mut Storage) -> Self {
        // Set various settings
        config.title("My game");
        config.resolution(800, 600);

        // Load a setting from last run
        let player = storage.get("player")
            .unwrap_or("New Player".into());

        Example {
            player
        }
    }

    // Called repeatedly
    fn update(&mut self, api: &mut Api) {
        // Set screen to blue if touching the screen or mouse on screen, red otherwise
        if api.input().pointers().len() >= 1 {
            api.window().set(Color::BLUE);
        } {
            api.window().set(Color::RED);
        }
    }

    fn resume(&mut self, api: &mut Api) { api.run_update(true); }
    fn suspend(&mut self, api: &mut Api) { api.run_update(false); }
}


pub fn new() -> Box<Box<dyn Game>> {
    Box::new(Box::new(Example))
}