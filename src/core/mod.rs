use std::collections::HashMap;

use gerber_parser::gerber_types::MacroContent;

use crate::core::{
    bounding_box::BoundingBox,
    geometry::{Arc, Circle, Point2d, Rectangle, Segment},
};

pub mod bounding_box;
pub mod debug;
pub mod geometry;
pub mod parser;
pub mod toolpath;
pub mod tools;

#[derive(Debug)]
enum Primitive {
    Segment(Segment),
    Arc(Arc),
    Circle(Circle),
    Rectangle(Rectangle),
    Polygon(Vec<Point2d>),
    Complex(Vec<Primitive>),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum LayerUnit {
    #[default]
    Millimeter,
    Inch,
}

#[derive(Debug, Default)]
pub struct PcbTrace {
    primitives: Vec<Primitive>,
}

#[derive(Debug, Default)]
pub struct PcbLayer {
    pub unit: LayerUnit,
    pub traces: Vec<PcbTrace>,
    pub aperture_macro: HashMap<String, Vec<MacroContent>>,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct DrillHole {
    pub x: f64,
    pub y: f64,
    pub diameter: f64,
}

#[derive(Debug, Default, Clone)]
pub struct DrillLayer {
    pub holes: Vec<DrillHole>,
}

impl DrillLayer {
    pub fn mirror_x(&mut self, center_x: Option<f64>) {
        let mut b_box = BoundingBox::new();

        if center_x.is_none() {
            for p in &self.holes {
                b_box.update_from_point(p.x, p.y);
            }
        }

        if !b_box.is_infinite() || center_x.is_some() {
            let center_x = center_x.unwrap_or(b_box.get_center().x);

            for p in self.holes.iter_mut() {
                p.x -= 2.0 * (p.x - center_x);
            }
        }
    }

    pub fn mirror_y(&mut self, center_y: Option<f64>) {
        let mut b_box = BoundingBox::new();

        if center_y.is_none() {
            for p in &self.holes {
                b_box.update_from_point(p.x, p.y);
            }
        }

        if !b_box.is_infinite() || center_y.is_some() {
            let center_y = center_y.unwrap_or(b_box.get_center().y);

            for p in self.holes.iter_mut() {
                p.y -= 2.0 * (p.y - center_y);
            }
        }
    }

    pub fn translate(&mut self, offset: Point2d) {
        for p in self.holes.iter_mut() {
            p.x += offset.x;
            p.y += offset.y;
        }
    }
}
