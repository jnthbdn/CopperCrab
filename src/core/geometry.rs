#[derive(Debug, Clone, Default)]
pub struct Point2d {
    pub x: f64,
    pub y: f64,
}

impl Point2d {
    pub fn new(x: f64, y: f64) -> Point2d {
        Point2d { x, y }
    }

    pub fn to_tuple(&self) -> (f64, f64) {
        (self.x, self.y)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Segment {
    pub start: Point2d,
    pub end: Point2d,
    pub width: f64,
}

#[derive(Debug, Clone, Default)]
pub struct Arc {
    pub start: Point2d,
    pub end: Point2d,
    pub center: Point2d,
    pub clockwise: bool,
    pub width: f64,
}

#[derive(Debug, Clone, Default)]
pub struct Circle {
    pub center: Point2d,
    pub diameter: f64,
}

#[derive(Debug, Clone, Default)]
pub struct Rectangle {
    pub center: Point2d,
    pub width: f64,
    pub height: f64,
    // pub rotation: f64,
}
