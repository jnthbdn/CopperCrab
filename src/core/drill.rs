use crate::core::{
    bounding_box::BoundingBox,
    geometry::Point2d,
};

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