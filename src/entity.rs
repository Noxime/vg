use components::Component;
use graphics::*;
use vectors::*;

pub struct Entity {
    pub position: Vec2<f32>,
    pub rotation: Vec2<f32>,
    pub scale: Vec2<f32>,
    components: Vec<Box<Component>>,
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

    pub fn render_init(&mut self, data: &mut APIData) {
        for c in self.components.iter_mut() { c.render_init(data) }
    }

    pub fn render(&mut self, data: &mut APIData) {
        for c in self.components.iter_mut() { c.render(data) }
    }

    pub fn render_destroy(&mut self, data: &mut APIData) {
        for c in self.components.iter_mut() { c.render_destroy(data) }
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
