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

#[derive(Debug, Clone)]
pub(crate) enum Primitive {
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
    pub(crate) primitives: Vec<Primitive>,
}

#[derive(Debug, Default)]
pub struct PcbLayer {
    pub unit: LayerUnit,
    pub traces: Vec<PcbTrace>,
    pub aperture_macro: HashMap<String, Vec<MacroContent>>,
}

impl PcbLayer {
    pub fn translate(&mut self, offset: Point2d) {
        for trace in &mut self.traces {
            for prim in &mut trace.primitives {
                translate_primitive(prim, &offset);
            }
        }
    }

    pub fn mirror_x(&mut self, center_x: Option<f64>) {
        let center_x = match center_x {
            Some(c) => c,
            None => {
                let mut bb = BoundingBox::new();
                self.update_bounding_box(&mut bb);
                if bb.is_infinite() {
                    return;
                }
                bb.get_center().x
            }
        };

        for trace in &mut self.traces {
            for prim in &mut trace.primitives {
                mirror_primitive_x(prim, center_x);
            }
        }
    }

    pub fn mirror_y(&mut self, center_y: Option<f64>) {
        let center_y = match center_y {
            Some(c) => c,
            None => {
                let mut bb = BoundingBox::new();
                self.update_bounding_box(&mut bb);
                if bb.is_infinite() {
                    return;
                }
                bb.get_center().y
            }
        };

        for trace in &mut self.traces {
            for prim in &mut trace.primitives {
                mirror_primitive_y(prim, center_y);
            }
        }
    }

    fn update_bounding_box(&self, bb: &mut BoundingBox) {
        for trace in &self.traces {
            for prim in &trace.primitives {
                update_bb_from_primitive(prim, bb);
            }
        }
    }
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

fn translate_primitive(prim: &mut Primitive, offset: &Point2d) {
    match prim {
        Primitive::Segment(s) => {
            s.start.x += offset.x;
            s.start.y += offset.y;
            s.end.x += offset.x;
            s.end.y += offset.y;
        }
        Primitive::Arc(a) => {
            a.start.x += offset.x;
            a.start.y += offset.y;
            a.end.x += offset.x;
            a.end.y += offset.y;
            a.center.x += offset.x;
            a.center.y += offset.y;
        }
        Primitive::Circle(c) => {
            c.center.x += offset.x;
            c.center.y += offset.y;
        }
        Primitive::Rectangle(r) => {
            r.center.x += offset.x;
            r.center.y += offset.y;
        }
        Primitive::Polygon(points) => {
            for p in points {
                p.x += offset.x;
                p.y += offset.y;
            }
        }
        Primitive::Complex(children) => {
            for child in children {
                translate_primitive(child, offset);
            }
        }
    }
}

fn mirror_primitive_x(prim: &mut Primitive, center_x: f64) {
    let mirror = |x: &mut f64| {
        *x -= 2.0 * (*x - center_x);
    };

    match prim {
        Primitive::Segment(s) => {
            mirror(&mut s.start.x);
            mirror(&mut s.end.x);
        }
        Primitive::Arc(a) => {
            mirror(&mut a.start.x);
            mirror(&mut a.end.x);
            mirror(&mut a.center.x);
            a.clockwise = !a.clockwise;
        }
        Primitive::Circle(c) => mirror(&mut c.center.x),
        Primitive::Rectangle(r) => {
            mirror(&mut r.center.x);
            r.rotation = -r.rotation;
        }
        Primitive::Polygon(points) => {
            for p in points.iter_mut() {
                mirror(&mut p.x);
            }
            points.reverse();
        }
        Primitive::Complex(children) => {
            for child in children {
                mirror_primitive_x(child, center_x);
            }
        }
    }
}

fn mirror_primitive_y(prim: &mut Primitive, center_y: f64) {
    let mirror = |y: &mut f64| {
        *y -= 2.0 * (*y - center_y);
    };

    match prim {
        Primitive::Segment(s) => {
            mirror(&mut s.start.y);
            mirror(&mut s.end.y);
        }
        Primitive::Arc(a) => {
            mirror(&mut a.start.y);
            mirror(&mut a.end.y);
            mirror(&mut a.center.y);
            a.clockwise = !a.clockwise;
        }
        Primitive::Circle(c) => mirror(&mut c.center.y),
        Primitive::Rectangle(r) => {
            mirror(&mut r.center.y);
            r.rotation = -r.rotation;
        }
        Primitive::Polygon(points) => {
            for p in points.iter_mut() {
                mirror(&mut p.x);
            }
            points.reverse();
        }
        Primitive::Complex(children) => {
            for child in children {
                mirror_primitive_y(child, center_y);
            }
        }
    }
}

fn update_bb_from_primitive(prim: &Primitive, bb: &mut BoundingBox) {
    match prim {
        Primitive::Segment(s) => {
            bb.update_from_point(s.start.x, s.start.y);
            bb.update_from_point(s.end.x, s.end.y);
        }
        Primitive::Arc(a) => {
            bb.update_from_point(a.start.x, a.start.y);
            bb.update_from_point(a.end.x, a.end.y);
            bb.update_from_point(a.center.x, a.center.y);
        }
        Primitive::Circle(c) => {
            let r = c.diameter / 2.0;
            bb.update_from_point(c.center.x - r, c.center.y - r);
            bb.update_from_point(c.center.x + r, c.center.y + r);
        }
        Primitive::Rectangle(r) => {
            let hw = r.width / 2.0;
            let hh = r.height / 2.0;
            bb.update_from_point(r.center.x - hw, r.center.y - hh);
            bb.update_from_point(r.center.x + hw, r.center.y + hh);
        }
        Primitive::Polygon(points) => {
            for p in points {
                bb.update_from_point(p.x, p.y);
            }
        }
        Primitive::Complex(children) => {
            for child in children {
                update_bb_from_primitive(child, bb);
            }
        }
    }
}
