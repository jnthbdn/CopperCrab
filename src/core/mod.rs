use crate::core::geometry::{Arc, Circle, Rectangle, Segment};

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
