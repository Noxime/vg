use entity::*;

pub struct Scene {
    entities: Vec<Entity>,
}

impl Scene {
    pub fn empty() -> Scene {
        debug!("new empty scene");
        Scene {
            entities: vec![]
        }
    }

    pub fn prepare_render(&mut self) {
        for mut entity in self.entities.iter_mut() {
            entity.prepare_render();
        }
    }

    pub fn render(&mut self) {
        for mut entity in self.entities.iter_mut() {
            entity.render();
        }
    }

    pub fn destroy_render(&mut self) {
        for mut entity in self.entities.iter_mut() {
            entity.destroy_render();
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