use crate::core::geometry::{Arc, Circle, Point2d, Rectangle, Segment};

#[derive(Debug, Clone)]
pub enum Primitive {
    Segment(Segment),
    Arc(Arc),
    Circle(Circle),
    Rectangle(Rectangle),
    Polygon(Vec<Point2d>),
    Complex(Vec<Primitive>),
}

impl Primitive {
    pub(crate) fn translate(&mut self, offset: &Point2d) {
        match self {
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
                    child.translate(offset);
                }
            }
        }
    }

    pub(crate) fn mirror_x(&mut self, center_x: f64) {
        let mirror = |x: &mut f64| {
            *x -= 2.0 * (*x - center_x);
        };

        match self {
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
                    child.mirror_x(center_x);
                }
            }
        }
    }

    pub(crate) fn mirror_y(&mut self, center_y: f64) {
        let mirror = |y: &mut f64| {
            *y -= 2.0 * (*y - center_y);
        };

        match self {
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
                    mirror(&mut p.y);
                }
                points.reverse();
            }
            Primitive::Complex(children) => {
                for child in children {
                    child.mirror_y(center_y);
                }
            }
        }
    }

    pub(crate) fn update_bounding_box(&self, bb: &mut crate::core::bounding_box::BoundingBox) {
        match self {
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
                    child.update_bounding_box(bb);
                }
            }
        }
    }
}