use std::fmt::Debug;

use clipper2::{Bounds, PointScaler};

use crate::core::geometry::Point2d;

#[derive(Debug)]
pub struct BoundingBox {
    pub min: Point2d,
    pub max: Point2d,
}

impl BoundingBox {
    pub fn new() -> Self {
        Self {
            min: Point2d {
                x: f64::INFINITY,
                y: f64::INFINITY,
            },
            max: Point2d {
                x: -f64::INFINITY,
                y: -f64::INFINITY,
            },
        }
    }

    pub fn update_from_point(&mut self, x: f64, y: f64) {
        self.min.x = self.min.x.min(x);
        self.min.y = self.min.y.min(y);
        self.max.x = self.max.x.max(x);
        self.max.y = self.max.y.max(y);
    }

    pub fn update_from_bounds<P: PointScaler + Debug>(&mut self, bounds: &Bounds<P>) {
        self.min.x = self.min.x.min(bounds.min.x());
        self.min.y = self.min.y.min(bounds.min.y());
        self.max.x = self.max.x.max(bounds.max.x());
        self.max.y = self.max.y.max(bounds.max.y());
    }

    pub fn to_tuples(&self) -> (f64, f64, f64, f64) {
        (self.min.x, self.min.y, self.max.x, self.max.y)
    }

    pub fn is_infinite(&self) -> bool {
        self.min.x.is_infinite()
            || self.min.y.is_infinite()
            || self.max.x.is_infinite()
            || self.max.y.is_infinite()
    }

    pub fn get_center(&self) -> Point2d {
        Point2d {
            x: self.min.x + ((self.max.x - self.min.x) / 2.0),
            y: self.min.y + ((self.max.y - self.min.y) / 2.0),
        }
    }
}
