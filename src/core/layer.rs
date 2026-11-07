use std::collections::HashMap;

use gerber_parser::gerber_types::MacroContent;

use crate::core::{
    bounding_box::BoundingBox,
    geometry::Point2d,
    primitive::Primitive,
};

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
                prim.translate(&offset);
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
                prim.mirror_x(center_x);
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
                prim.mirror_y(center_y);
            }
        }
    }

    fn update_bounding_box(&self, bb: &mut BoundingBox) {
        for trace in &self.traces {
            for prim in &trace.primitives {
                prim.update_bounding_box(bb);
            }
        }
    }
}