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

    pub fn render(&mut self) -> Vec<DrawCall> {
        self.entities.iter_mut().fold(vec![], |mut s, v| {
            s.append(&mut v.render());
            s
        })
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
