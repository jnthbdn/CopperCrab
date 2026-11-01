use clipper2::{EndType, FillRule, JoinType, Path, Paths, Point};

use std::fmt::{self, Write};

use rust_i18n::t;

use crate::core::{
    DrillLayer, PcbLayer, Primitive,
    bounding_box::BoundingBox,
    geometry::Point2d,
    toolpath::helpers::{
        arc_to_path, circle_to_path, complex_to_path, polygon_to_paths, rectangle_to_path,
        segment_to_path,
    },
    tools::CncTool,
};

mod helpers;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub enum ToolpathError {
    BuildError(String),
}

impl fmt::Display for ToolpathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolpathError::BuildError(msg) => write!(f, "Build error: {}", msg),
        }
    }
}

impl std::error::Error for ToolpathError {}

#[derive(Debug, Clone)]
pub struct ToolpathGenerator {
    pub traces: Paths,
}

impl ToolpathGenerator {
    pub fn new(layer: &PcbLayer, is_outline: bool) -> Result<Self, ToolpathError> {
        log::info!("{}", t!("toolpath.info.build_path"));
        let mut traces: Vec<Paths> = Vec::new();
        let mut s: Self;

        for trace in &layer.traces {
            for primitive in &trace.primitives {
                traces.push(primitive_to_paths(primitive));
            }
        }

        if traces.len() >= 2 {
            let mut final_clipper = traces[0].to_clipper_subject().add_clip(traces[1].clone());

            for i in 2..traces.len() {
                final_clipper = final_clipper
                    .union(FillRule::default())
                    .map_err(|_| {
                        ToolpathError::BuildError("Failed to execute union operation".into())
                    })?
                    .to_clipper_subject()
                    .add_clip(traces[i].clone());
            }

            s = Self {
                traces: final_clipper.union(FillRule::default()).map_err(|_| {
                    ToolpathError::BuildError("Failed to execute union operation".into())
                })?,
            };
        } else {
            s = Self {
                traces: if traces.len() > 0 {
                    traces[0].clone()
                } else {
                    Default::default()
                },
            };
        }

        if is_outline {
            s.traces = s
                .traces
                .into_iter()
                .filter(|p| p.signed_area() > 0.0)
                .collect();
        }

        Ok(s)
    }

    pub fn generate_toolpaths(
        &self,
        tool: &dyn CncTool,
        depth: f64,
        nb_passes: u32,
        pass_overlap: f64,
        start_offset: f64,
    ) -> Vec<Paths> {
        let width = tool.cutting_width(depth);
        let step = width * (1.0 - pass_overlap.min(1.0).max(0.0));

        (0..nb_passes)
            .map(|pass| {
                let offset = start_offset + (width / 2.0) + (pass as f64) * step;
                self.traces
                    .inflate(offset, JoinType::Round, EndType::Polygon, 2.0)
            })
            .collect::<Vec<Paths>>()
    }

    pub fn mirror_x(&mut self, center_x: Option<f64>) {
        let mut b_box = BoundingBox::new();

        if center_x.is_none() {
            for path in self.traces.iter() {
                b_box.update_from_bounds(&path.bounds());
            }
        }

        if !b_box.is_infinite() || center_x.is_some() {
            let center_x = center_x.unwrap_or(b_box.get_center().x);
            let mut new_traces = Vec::new();

            for path in self.traces.iter() {
                let mut new_path = Vec::new();
                for p in path.iter() {
                    new_path.push(Point::new(p.x() - 2.0 * (p.x() - center_x), p.y()));
                }

                new_traces.push(Path::new(new_path));
            }

            self.traces = Paths::new(new_traces);
        }
    }

    pub fn mirror_y(&mut self, center_y: Option<f64>) {
        let mut b_box = BoundingBox::new();

        if center_y.is_none() {
            for path in self.traces.iter() {
                b_box.update_from_bounds(&path.bounds());
            }
        }

        if !b_box.is_infinite() || center_y.is_some() {
            let center_y = center_y.unwrap_or(b_box.get_center().y);
            let mut new_traces = Vec::new();

            for path in self.traces.iter() {
                let mut new_path = Vec::new();
                for p in path.iter() {
                    new_path.push(Point::new(p.x(), p.y() - 2.0 * (p.y() - center_y)));
                }

                new_traces.push(Path::new(new_path));
            }

            self.traces = Paths::new(new_traces);
        }
    }

    pub fn translate(&mut self, offset: Point2d) {
        let mut new_traces = Vec::new();

        for path in self.traces.iter() {
            let mut new_path = Vec::new();
            for p in path.iter() {
                new_path.push(Point::new(p.x() + offset.x, p.y() + offset.y));
            }

            new_traces.push(Path::new(new_path));
        }

        self.traces = Paths::new(new_traces);
    }
}

pub fn generate_isolation_gcode(
    passes: &Vec<Paths>,
    tool: &dyn CncTool,
    depth: f64,
    z_safe: f64,
    z_finish: f64,
) -> String {
    let mut gcode = String::new();

    // Header GRBL
    writeln!(gcode, "; Generated by CopperCrab - v{VERSION}").unwrap();
    writeln!(gcode, "; Tool: {}", tool.name()).unwrap();
    writeln!(gcode, "; Depth: {}mm", depth).unwrap();
    writeln!(gcode, "G21 ; mm").unwrap();
    writeln!(gcode, "G90 ; absolute coordinates").unwrap();
    writeln!(gcode, "G17 ; XY plane").unwrap();

    // Go to safe position
    writeln!(gcode, "G0 Z{}", z_safe).unwrap();
    writeln!(gcode, "G0 X0 Y0").unwrap();
    writeln!(gcode, "M3 S{}; Start spindle", tool.spindle_speed()).unwrap();

    for (pass_index, paths) in passes.iter().enumerate() {
        writeln!(gcode, "\n; ========== Pass {} ==========", pass_index + 1).unwrap();

        for path in paths.iter() {
            let mut points = path.iter();

            if let Some(first) = points.next() {
                writeln!(gcode, "; Move to the first point").unwrap();
                writeln!(gcode, "G0 Z{}", z_safe).unwrap();
                writeln!(gcode, "G0 X{:.4} Y{:.4}", first.x(), first.y()).unwrap();

                let mut current_depth = 0.0;

                while current_depth < depth {
                    current_depth = (current_depth + tool.max_depth()).min(depth);

                    writeln!(
                        gcode,
                        "; ----- Depth step {} / {} -----",
                        current_depth, depth
                    )
                    .unwrap();

                    // Plunge
                    writeln!(gcode, "G1 Z{:.4} F{}", -current_depth, tool.plunge_rate()).unwrap();

                    // Follow the path
                    for point in points.as_ref() {
                        writeln!(
                            gcode,
                            "G1 X{:.4} Y{:.4} F{}",
                            point.x(),
                            point.y(),
                            tool.feed_rate()
                        )
                        .unwrap();
                    }

                    // Closing path
                    writeln!(gcode, "G1 X{:.4} Y{:.4}", first.x(), first.y()).unwrap();
                }

                // Go safe Z
                writeln!(gcode, "G0 Z{}", z_safe).unwrap();
            }
        }
    }

    // Footer
    writeln!(gcode, "\n; End of program").unwrap();
    writeln!(gcode, "G0 Z{} ; Go to finish Z", z_finish).unwrap();
    writeln!(gcode, "M5 ; Spindle off").unwrap();
    writeln!(gcode, "M30 ; end").unwrap();

    gcode
}

fn primitive_to_paths(primitive: &Primitive) -> Paths {
    match primitive {
        Primitive::Segment(segment) => segment_to_path(segment),
        Primitive::Arc(arc) => arc_to_path(arc, 100),
        Primitive::Circle(circle) => circle_to_path(circle, 100),
        Primitive::Rectangle(rectangle) => rectangle_to_path(rectangle),
        Primitive::Complex(primitives) => complex_to_path(primitives, 100),
        Primitive::Polygon(point2ds) => polygon_to_paths(point2ds),
    }
}

pub fn generate_drill_gcode(
    drill_layer: &DrillLayer,
    tool: &dyn CncTool,
    depth: f64,
    z_safe: f64,
    z_finish: f64,
    peck_step: f64,
) -> String {
    let mut gcode = String::new();

    // Header GRBL
    writeln!(gcode, "; Generated by CopperCrab - v{VERSION}").unwrap();
    writeln!(gcode, "; Tool: {}", tool.name()).unwrap();
    writeln!(gcode, "; Depth: {}mm", depth).unwrap();
    writeln!(gcode, "G21 ; mm").unwrap();
    writeln!(gcode, "G90 ; absolute coordinates").unwrap();
    writeln!(gcode, "G17 ; XY plane").unwrap();

    // Go to safe position
    writeln!(gcode, "G0 Z{}", z_safe).unwrap();
    writeln!(gcode, "G0 X0 Y0").unwrap();
    writeln!(gcode, "M3 S{}; Start spindle", tool.spindle_speed()).unwrap();

    for hole in &drill_layer.holes {
        // Move to hole position
        writeln!(gcode, "G0 X{:.4} Y{:.4}", hole.x, hole.y).unwrap();

        // Run drill operation with peck step
        let mut current_depth = 0.0;
        while current_depth < depth {
            current_depth = (current_depth + peck_step).min(depth);
            let retract = (-current_depth + peck_step).min(0.0);

            writeln!(gcode, "G1 Z{} F{}", -current_depth, tool.plunge_rate()).unwrap();
            writeln!(gcode, "G1 Z{} F{}", retract, tool.plunge_rate()).unwrap();
        }

        // Go to safe position
        writeln!(gcode, "G0 Z{}", z_safe).unwrap();
    }

    // Footer
    writeln!(gcode, "\n; End of program").unwrap();
    writeln!(gcode, "G80 ; End drill cycle").unwrap();
    writeln!(gcode, "G0 Z{} ; Go to finish Z", z_finish).unwrap();
    writeln!(gcode, "M5 ; Spindle off").unwrap();
    writeln!(gcode, "M30 ; end").unwrap();

    gcode
}
