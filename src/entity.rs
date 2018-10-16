use components::Component;
use graphics::*;
use vectors::*;

use std::{any::TypeId, collections::HashMap, intrinsics::type_name};

pub struct Entity {
    pub position: Vec2<f32>,
    pub rotation: Vec2<f32>,
    pub scale: Vec2<f32>,
    components: HashMap<TypeId, Box<dyn Component>>,
}

impl Entity {
    pub fn empty() -> Entity {
        debug!("new empty entity");
        Entity {
            position: Vec2::new(0.0, 0.0),
            rotation: Vec2::new(1.0, 0.0),
            scale: Vec2::new(1.0, 1.0),
            components: HashMap::new(),
        }
    }

    pub fn render_init(&mut self, renderer: &mut Renderer) {
        for (_, c) in self.components.iter_mut() {
            c.render_init(renderer)
        }
    }

    pub fn render(&mut self, renderer: &mut Renderer) {
        for (_, c) in self.components.iter_mut() {
            c.render(renderer)
        }
    }

    pub fn render_destroy(&mut self, renderer: &mut Renderer) {
        for (_, c) in self.components.iter_mut() {
            c.render_destroy(renderer)
        }
    }

    pub fn add<T: 'static + Component>(&mut self, component: T) {
        debug!("added component `{}` to entity", unsafe {
            type_name::<T>()
        });
        self.components.insert(
            TypeId::of::<T>(),
            Box::new(component) as Box<dyn Component>,
        );
    }

    pub fn with<T: 'static + Component>(mut self, component: T) -> Self {
        debug!("added component `{}` to entity", unsafe {
            type_name::<T>()
        });
        self.components.insert(
            TypeId::of::<T>(),
            Box::new(component) as Box<dyn Component>,
        );
        self
    }

    pub fn get<T: 'static + Component>(&mut self) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())
            .and_then(|v| v.as_any().downcast_ref::<T>())
    }
}
