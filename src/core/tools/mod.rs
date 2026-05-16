pub mod drill;
pub mod endmill;
pub mod library;
pub mod vbit;

pub trait CncTool {
    fn name(&self) -> &str;
    fn cutting_width(&self, depth: f64) -> f64;
    fn max_depth(&self) -> f64;
    fn feed_rate(&self) -> f64;
    fn plunge_rate(&self) -> f64;
    fn spindle_speed(&self) -> f64;

    fn to_string(&self) -> String;
}
