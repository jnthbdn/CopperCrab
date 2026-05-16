use serde::{Deserialize, Serialize};

use crate::core::tools::CncTool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VBit {
    pub name: String,
    pub angle_deg: f64,
    pub tip_diameter: f64,
    pub max_depth: f64,
    pub feed_rate: f64,
    pub plunge_rate: f64,
    pub spindle_speed: f64,
}

impl CncTool for VBit {
    fn name(&self) -> &str {
        &self.name
    }

    fn cutting_width(&self, depth: f64) -> f64 {
        self.tip_diameter + 2.0 * depth * (self.angle_deg / 2.0).to_radians().tan()
    }

    fn max_depth(&self) -> f64 {
        self.max_depth
    }
    fn feed_rate(&self) -> f64 {
        self.feed_rate
    }
    fn plunge_rate(&self) -> f64 {
        self.plunge_rate
    }

    fn to_string(&self) -> String {
        format!(
            "{} - tip: {} mm - θ: {}°",
            self.name, self.tip_diameter, self.angle_deg
        )
    }

    fn spindle_speed(&self) -> f64 {
        self.spindle_speed
    }
}
