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

    pub fn render_init(&mut self, data: &mut APIData) {
        for e in self.entities.iter_mut() {
            e.render_init(data);
        }
    }

    pub fn render(&mut self, data: &mut APIData) {
        for e in self.entities.iter_mut() {
            e.render(data);
        }
    }

    pub fn render_destroy(&mut self, data: &mut APIData) {
        for e in self.entities.iter_mut() {
            e.render_destroy(data);
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
