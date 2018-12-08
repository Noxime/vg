use entity::*;
use graphics::*;

pub struct Scene {
    entities: Vec<Entity>,
}

impl Scene {
    pub fn empty() -> Scene {
        debug!("new empty scene");
        Scene { entities: vec![] }
    }

    pub fn update(&mut self) -> usize {
        for e in self.entities.iter_mut() {
            e.update();
        }
        0
    }

    pub fn render_init(&mut self, renderer: &mut Renderer) {
        for e in self.entities.iter_mut() {
            e.render_init(renderer);
        }
    }

    pub fn render(&mut self, renderer: &mut Renderer) {
        for e in self.entities.iter_mut() {
            e.render(renderer);
        }
    }

    pub fn render_destroy(&mut self, renderer: &mut Renderer) {
        for e in self.entities.iter_mut() {
            e.render_destroy(renderer);
        }
    }

    pub fn add_entity(&mut self, entity: Entity) {
        debug!("added entity to scene");
        self.entities.push(entity);
    }

    pub fn with_entity(mut self, entity: Entity) -> Self {
        self.add_entity(entity);
        self
    }
}
