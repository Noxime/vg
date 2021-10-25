use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec4};

use crate::Locals;

/// A shape which can be drawn by a [`Renderer`](crate::Renderer)
pub struct Shape {
    kind: ShapeKind,
    color: Vec4,
    radius: f32,
    outline: Option<f32>,
}

/// The specific kind of shape to draw
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum ShapeKind {
    /// A line from A to B, with rounded caps
    Line(Vec2, Vec2),
    /// Circle
    Circle(Vec2),
    /// Rectangle centered on A, with size B and angle Th
    Rect(Vec2, Vec2, f32),
    // Triangle going from A to B to C
    Triangle(Vec2, Vec2, Vec2),
    // A cubic Bezier curve with 2 ends and 2 control points
    Bezier(Vec2, Vec2, Vec2, Vec2),
}

unsafe impl Pod for ShapeKind {}
unsafe impl Zeroable for ShapeKind {}

impl Shape {
    /// Create a new line going from A to B. Do not forget to set the desired
    /// width with [`with_radius`](Shape::with_radius)
    pub fn line(a: Vec2, b: Vec2) -> Shape {
        Shape {
            kind: ShapeKind::Line(a, b),
            ..Default::default()
        }
    }

    /// Create a new circle centered around P. Do not forget to set the desired
    /// radius with [`with_radius`](Shape::with_radius)
    pub fn circle(p: Vec2) -> Shape {
        Shape {
            kind: ShapeKind::Circle(p),
            ..Default::default()
        }
    }

    /// Create a new rectangle around A with size B and angle (in radians) of Th
    pub fn rect(a: Vec2, b: Vec2, th: f32) -> Shape {
        Shape {
            kind: ShapeKind::Rect(a, b, th),
            ..Default::default()
        }
    }

    /// Create a new triangle with the corners at A, B, C
    pub fn triangle(a: Vec2, b: Vec2, c: Vec2) -> Shape {
        Shape {
            kind: ShapeKind::Triangle(a, b, c),
            ..Default::default()
        }
    }

    /// Create a new cubic Bézier curve with ends at A and D, and control points
    /// at B and C, respectively. Do not forget to set the width with 
    /// [`with_radius`](Shape::with_radius)
    pub fn bezier(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> Shape {
        Shape {
            kind: ShapeKind::Bezier(a, b, c, d),
            ..Default::default()
        }
    }

    /// Either the "rounding" or width, depending what shape is being drawn
    /// 
    /// In vg-2d, shapes like lines or circles are zero-width by default, so
    /// they get their width from the rounding effect
    pub fn with_radius(mut self, r: f32) -> Shape {
        self.radius = r;
        self
    }

    /// Set or unset the thickness of an outline look. Setting O to Some(0.0) is
    /// Same as None
    pub fn with_outline(mut self, o: Option<f32>) -> Shape {
        self.outline = o;
        self
    }

    /// Set the color for this shape
    pub fn with_color(mut self, c: Vec4) -> Shape {
        self.color = c;
        self
    }

    pub(crate) fn as_locals(&self) -> Locals {
        match self.kind {
            ShapeKind::Line(a, b) => Locals {
                xyzw: Vec4::new(a.x, a.y, b.x, b.y),
                ..self.locals_incomplete()
            },
            ShapeKind::Circle(p) => Locals {
                xyzw: Vec4::new(p.x, p.y, 0.0, 0.0),
                ..self.locals_incomplete()
            },
            ShapeKind::Rect(a, b, th) => {
                // A is center and B is radius, th is angle. In shader this is
                // more like a hard-edge line

                let o = Vec2::new(th.cos() * b.x, th.sin() * b.x);

                Locals {
                    xyzw: Vec4::new(a.x - o.x, a.y - o.y, a.x + o.x, a.y + o.y),
                    uvst: Vec4::new(b.y * 2.0, 0.0, 0.0, 0.0),
                    ..self.locals_incomplete()
                }
            }
            ShapeKind::Triangle(a, b, c) => Locals {
                xyzw: Vec4::new(a.x, a.y, b.x, b.y),
                uvst: Vec4::new(c.x, c.y, 0.0, 0.0),
                ..self.locals_incomplete()
            },
            ShapeKind::Bezier(a, b, c, d) => Locals {
                xyzw: Vec4::new(a.x, a.y, b.x, b.y),
                uvst: Vec4::new(c.x, c.y, d.x, d.y),
                ..self.locals_incomplete()
            },
        }
    }

    fn locals_incomplete(&self) -> Locals {
        Locals {
            color: self.color,
            props: Vec4::new(
                self.radius,
                self.outline.unwrap_or(0.0),
                bytemuck::bytes_of(&self.kind)[0] as f32, // Grab the discriminant, this is pretty nasty
                0.0,
            ),
            xyzw: Vec4::ZERO,
            uvst: Vec4::ZERO,
        }
    }    
}

impl Default for Shape {
    fn default() -> Shape {
        Shape {
            kind: ShapeKind::Line(Default::default(), Default::default()),
            color: Vec4::splat(1.0),
            radius: 0.0,
            outline: None,
        }
    }
}
