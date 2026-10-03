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

    pub fn x_symmetry(&mut self, x_ref: f64) {
        self.x += (x_ref - self.x) * 2.0;
    }

    pub fn y_symmetry(&mut self, y_ref: f64) {
        self.y += (y_ref - self.y) * 2.0;
    }

    pub fn add(mut self, other: &Point2d) -> Self {
        self.x += other.x;
        self.y += other.y;

        self
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
    pub rotation: f64,
}
