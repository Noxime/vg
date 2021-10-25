use std::time::Instant;

use glam::{Vec2, Vec4};
use vg_2d::{calculate_bounds, Renderer, Shape};

mod common;

fn main() {
    struct Stuff {
        renderer: Renderer,
        time: Instant,
        bounds: (Vec2, Vec2),
    }

    common::run(
        |device, size, format| Stuff {
            renderer: Renderer::new(device, size, format),
            time: Instant::now(),
            bounds: calculate_bounds(size),
        },
        |stuff, size, view, queue| {
            // Window resized
            if let Some(size) = size {
                stuff.bounds = calculate_bounds(size);
                stuff.renderer.resize(size);
            }

            let t = stuff.time.elapsed().as_secs_f32();

            let shapes = vec![
                Shape::line(
                    Vec2::new(-0.7 + t.sin() * 0.2, 0.4),
                    Vec2::new(-0.7 + t.cos() * 0.2, 0.8),
                )
                .with_radius(0.04 + t.sin() * 0.02)
                .with_color(Vec4::new(1.0, 0.5, 0.5, 1.0)),
                Shape::line(Vec2::new(-0.2, 0.45), Vec2::new(0.2, 0.8))
                    .with_radius(0.02)
                    .with_outline(Some(0.01 + t.sin() * 0.005))
                    .with_color(Vec4::new(0.5, 1.0, 0.5, 1.0)),
                Shape::circle(Vec2::new(-0.7, 0.0))
                    .with_radius(0.16 + t.sin() * 0.04)
                    .with_color(Vec4::new(0.4, 0.8, 0.8, 1.0)),
                Shape::circle(Vec2::new(0.0, 0.0))
                    .with_radius(0.12)
                    .with_outline(Some(0.04 + t.sin() * 0.02))
                    .with_color(Vec4::new(0.8, 0.4, 0.8, 1.0)),
                Shape::rect(Vec2::new(0.7, 0.6), Vec2::new(0.2, 0.2), t)
                    .with_radius(0.02 + t.sin() * 0.02)
                    .with_color(Vec4::new(0.5, 0.5, 1.0, 1.0)),
                Shape::rect(Vec2::new(0.7, 0.0), Vec2::new(0.2, 0.2), t.sin())
                    .with_radius(0.02 + t.sin() * 0.02)
                    .with_outline(Some(0.01 + t.sin() * 0.005))
                    .with_color(Vec4::new(0.8, 0.8, 0.4, 1.0)),
                Shape::triangle(
                    Vec2::new(-0.9, -0.8),
                    Vec2::new(-0.7, -0.4),
                    Vec2::new(-0.5, -0.8),
                )
                .with_outline(Some(0.01 + t.sin() * 0.005)),
                Shape::bezier(
                    Vec2::new(-0.2, -0.8),
                    Vec2::new(t.sin() * 0.2, -0.4),
                    Vec2::new(t.cos() * 0.2, -0.4),
                    Vec2::new(0.2, -0.8),
                )
                .with_radius(0.02)
                .with_outline(Some(0.01 + t.sin() * 0.005)),
            ];

            stuff.renderer.render(
                queue,
                &shapes,
                Some(Vec4::new(0.0, 0.0, 0.0, 1.0)),
                &view,
                stuff.bounds,
            );
        },
    );
}
