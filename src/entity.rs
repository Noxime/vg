use vectors::*;
use components::Component;
use graphics::*;

pub struct Entity {
    pub position: Vec2<f32>,
    pub rotation: Vec2<f32>,
    pub scale: Vec2<f32>,
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

    pub fn render(&mut self) -> Vec<DrawCall> {
        self.components
            .iter_mut()
            .map(|c| c.render())
            .filter(|v| v.is_some())
            .map(|v| v.unwrap())
            .collect()
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