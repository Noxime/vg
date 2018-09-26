use vectors::*;
use component::Component;

pub struct Entity {
    position: Vec2<f32>,
    rotation: Vec2<f32>,
    scale: Vec2<f32>,
    components: Vec<Box<Component>>
}

impl Entity {
    pub fn empty() -> Entity {
        debug!("new empty entity");
        Entity {
            position: Vec2::new(0.0, 0.0),
            rotation: Vec2::new(1.0, 0.0),
            scale: Vec2::new(1.0, 1.0),
            components: vec![],
        }
    }

    pub fn add_component(&mut self, component: Box<Component>) {
        debug!("added component to entity");
        self.components.push(component);
    }
    
    pub fn with_component(mut self, component: Box<Component>) -> Self {
        self.add_component(component);
        self
    }
}