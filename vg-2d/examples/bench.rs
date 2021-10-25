use glam::{Vec2, Vec4};
use vg_2d::{calculate_bounds, Renderer, Shape};

mod common;

fn main() {
    struct Stuff {
        renderer: Renderer,
        bounds: (Vec2, Vec2),
        shapes: Vec<Shape>,
    }

    common::run(
        |device, size, format| Stuff {
            renderer: Renderer::new(device, size, format),
            bounds: calculate_bounds(size),
            shapes: (0..50 * 50)
                .map(|i| ((i % 50) as f32 / 25.0 - 1.0, (i / 50) as f32 / 25.0 - 1.0))
                .map(|(x, y)| {
                    Shape::circle(Vec2::new(x, y))
                        .with_radius(0.01)
                        .with_color(Vec4::new(x.sin(), y.sin(), 1.0, 1.0) * 0.5 + 0.5)
                })
                .collect(),
        },
        |stuff, size, view, queue| {
            // Window resized
            if let Some(size) = size {
                stuff.bounds = calculate_bounds(size);
                stuff.renderer.resize(size);
            }

            stuff.renderer.render(
                queue,
                &stuff.shapes,
                Some(Vec4::new(0.0, 0.0, 0.0, 1.0)),
                &view,
                stuff.bounds,
            );
        },
    );
}
