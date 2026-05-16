use serde::{Deserialize, Serialize};

use crate::core::tools::CncTool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndMill {
    pub name: String,
    pub diameter: f64,
    pub max_depth: f64,
    pub feed_rate: f64,
    pub plunge_rate: f64,
    pub spindle_speed: f64,
}

impl CncTool for EndMill {
    fn name(&self) -> &str {
        &self.name
    }
    fn cutting_width(&self, _depth: f64) -> f64 {
        self.diameter
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
        format!("{} - dia: {} mm", self.name, self.diameter)
    }

    fn spindle_speed(&self) -> f64 {
        self.spindle_speed
    }
}
