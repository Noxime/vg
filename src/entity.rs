use vectors::*;
use components::Component;

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

    pub fn prepare_render(&mut self) {
        for mut component in self.components.iter_mut() {
            component.prepare_render();
        }
    }

    pub fn render(&mut self) {
        for mut component in self.components.iter_mut() {
            component.render();
        }
    }

    pub fn destroy_render(&mut self) {
        for mut component in self.components.iter_mut() {
            component.destroy_render();
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