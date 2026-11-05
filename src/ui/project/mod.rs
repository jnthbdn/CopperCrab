use std::collections::HashSet;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::{Primitive, bounding_box::BoundingBox, geometry::Point2d};

mod export;
mod operation;
mod project_file;
mod source;

pub(crate) use operation::{Operation, OperationKind, SelectedTool};
pub(crate) use project_file::{
    PROJECT_VERSION, ProjectFile, SourceFileSnapshot, SourceKindTag, SourceTransform, TransformOp,
};
pub(crate) use source::SourceKind;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct CommonParameters {
    pub(crate) z_safe: f64,
    pub(crate) z_finish: f64,
    #[serde(default = "default_move_step")]
    pub(crate) move_step: f64,
}

impl Default for CommonParameters {
    fn default() -> Self {
        Self {
            z_safe: 3.0,
            z_finish: 5.0,
            move_step: 1.0,
        }
    }
}

fn default_move_step() -> f64 {
    1.0
}

#[derive(Debug)]
pub(crate) struct SourceFile {
    pub name: String,
    pub is_visible: bool,
    pub source: SourceKind,
    pub operations: Vec<Operation>,

    pub path: Option<PathBuf>,

    pub(crate) next_id_operation: usize,

    pub(crate) selected_primitives: HashSet<(usize, usize)>,

    pub(crate) hidden_primitives: HashSet<(usize, usize)>,

    pub(crate) transform: SourceTransform,
}

impl SourceFile {
    pub(crate) fn new(name: String, source: SourceKind, path: Option<PathBuf>) -> Self {
        Self {
            name,
            is_visible: true,
            source,
            operations: Default::default(),
            path,
            next_id_operation: 0,
            selected_primitives: HashSet::new(),
            hidden_primitives: HashSet::new(),
            transform: SourceTransform::default(),
        }
    }

    pub(crate) fn next_operation_name(&mut self) -> String {
        self.next_id_operation += 1;
        format!(
            "{} #{}",
            self.source.operation_prefix(),
            self.next_id_operation
        )
    }

    pub(crate) fn translate(&mut self, offset: Point2d) {
        self.source.translate(offset.clone());
        self.transform.push(TransformOp::Translate {
            dx: offset.x,
            dy: offset.y,
        });
    }

    pub(crate) fn mirror_x(&mut self, center_x: Option<f64>) {
        let cx_current = match center_x {
            Some(c) => c,
            None => {
                let mut bb = BoundingBox::new();
                self.source.update_bounding_box(&mut bb);
                if bb.is_infinite() {
                    return;
                }
                bb.get_center().x
            }
        };

        self.source.mirror_x(Some(cx_current));
        self.transform.push(TransformOp::MirrorX { cx: cx_current });
    }

    pub(crate) fn mirror_y(&mut self, center_y: Option<f64>) {
        let cy_current = match center_y {
            Some(c) => c,
            None => {
                let mut bb = BoundingBox::new();
                self.source.update_bounding_box(&mut bb);
                if bb.is_infinite() {
                    return;
                }
                bb.get_center().y
            }
        };

        self.source.mirror_y(Some(cy_current));
        self.transform.push(TransformOp::MirrorY { cy: cy_current });
    }



    pub(crate) fn set_center(&mut self, x: f64, y: f64) {
        let mut bb = BoundingBox::new();
        self.source.update_bounding_box(&mut bb);
        if bb.is_infinite() {
            return;
        }
        let c = bb.get_center();
        let dx = x - c.x;
        let dy = y - c.y;
        if dx != 0.0 || dy != 0.0 {
            self.translate(Point2d::new(dx, dy));
        }
    }


    pub(crate) fn current_center(&self) -> Option<Point2d> {
        let mut bb = BoundingBox::new();
        self.source.update_bounding_box(&mut bb);
        if bb.is_infinite() {
            None
        } else {
            Some(bb.get_center())
        }
    }

    pub(crate) fn to_snapshot(&self) -> Option<SourceFileSnapshot> {
        let path = self.path.clone()?;
        let kind = match &self.source {
            SourceKind::Copper { .. } => SourceKindTag::Copper,
            SourceKind::Outline(_) => SourceKindTag::Outline,
            SourceKind::Drill(_) => SourceKindTag::Drill,
        };

        let hidden_primitives_vec: Vec<(usize, usize)> =
            self.hidden_primitives.iter().copied().collect();

        Some(SourceFileSnapshot {
            name: self.name.clone(),
            is_visible: self.is_visible,
            path,
            kind,
            next_id_operation: self.next_id_operation,
            operations: self.operations.clone(),
            hidden_primitives: hidden_primitives_vec,
            transform: self.transform.clone(),
        })
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
