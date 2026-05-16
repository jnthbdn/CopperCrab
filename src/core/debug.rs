#![allow(unused)]

use clipper2::Paths;

use crate::core::{PcbLayer, Primitive};
use std::fmt::Write;
use std::path::Path;

pub fn export_svg(layer: &PcbLayer, path: &Path) -> std::io::Result<()> {
    let mut svg = String::new();

    // On calcule le bounding box pour le viewBox
    let (min_x, min_y, max_x, max_y) = bounding_box(layer);
    let w = max_x - min_x;
    let h = max_y - min_y;
    let margin = w.max(h) * 0.05;

    writeln!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}">"#,
        min_x - margin,
        min_y - margin,
        w + margin * 2.0,
        h + margin * 2.0
    )
    .unwrap();

    // Fond PCB
    writeln!(
        svg,
        r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#1a3a1a"/>"##,
        min_x - margin,
        min_y - margin,
        w + margin * 2.0,
        h + margin * 2.0
    )
    .unwrap();

    for trace in &layer.traces {
        for primitive in &trace.primitives {
            match primitive {
                Primitive::Segment(s) => {
                    writeln!(svg,
                        r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#B87333" stroke-width="{}" stroke-linecap="round"/>"##,
                        s.start.x, s.start.y, s.end.x, s.end.y, s.width
                    ).unwrap();
                }
                Primitive::Circle(c) => {
                    writeln!(
                        svg,
                        r##"<circle cx="{}" cy="{}" r="{}" fill="#B87333"/>"##,
                        c.center.x,
                        c.center.y,
                        c.diameter / 2.0
                    )
                    .unwrap();
                }
                Primitive::Rectangle(r) => {
                    writeln!(
                        svg,
                        r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#B87333"/>"##,
                        r.center.x - r.width / 2.0,
                        r.center.y - r.height / 2.0,
                        r.width,
                        r.height
                    )
                    .unwrap();
                }
                Primitive::Arc(a) => {
                    // SVG arc path
                    writeln!(svg,
                        r##"<path d="M {} {} A {} {} 0 0 {} {} {}" fill="none" stroke="#B87333" stroke-width="{}"/>"##,
                        a.start.x, a.start.y,
                        a.width / 2.0, a.width / 2.0,
                        if a.clockwise { 1 } else { 0 },
                        a.end.x, a.end.y,
                        a.width
                    ).unwrap();
                }
            }
        }
    }

    writeln!(svg, "</svg>").unwrap();

    std::fs::write(path, svg)?;
    log::info!("SVG debug exporté : {}", path.display());
    Ok(())
}

fn bounding_box(layer: &PcbLayer) -> (f64, f64, f64, f64) {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for trace in &layer.traces {
        for primitive in &trace.primitives {
            match primitive {
                Primitive::Segment(s) => {
                    min_x = min_x.min(s.start.x).min(s.end.x);
                    min_y = min_y.min(s.start.y).min(s.end.y);
                    max_x = max_x.max(s.start.x).max(s.end.x);
                    max_y = max_y.max(s.start.y).max(s.end.y);
                }
                Primitive::Circle(c) => {
                    min_x = min_x.min(c.center.x - c.diameter / 2.0);
                    min_y = min_y.min(c.center.y - c.diameter / 2.0);
                    max_x = max_x.max(c.center.x + c.diameter / 2.0);
                    max_y = max_y.max(c.center.y + c.diameter / 2.0);
                }
                Primitive::Rectangle(r) => {
                    min_x = min_x.min(r.center.x - r.width / 2.0);
                    min_y = min_y.min(r.center.y - r.height / 2.0);
                    max_x = max_x.max(r.center.x + r.width / 2.0);
                    max_y = max_y.max(r.center.y + r.height / 2.0);
                }
                Primitive::Arc(a) => {
                    min_x = min_x.min(a.start.x).min(a.end.x);
                    min_y = min_y.min(a.start.y).min(a.end.y);
                    max_x = max_x.max(a.start.x).max(a.end.x);
                    max_y = max_y.max(a.start.y).max(a.end.y);
                }
            }
        }
    }

    (min_x, min_y, max_x, max_y)
}

pub fn paths_to_svg(paths: &Paths, output: &Path) -> std::io::Result<()> {
    // Bounding box
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for path in paths.iter() {
        for point in path.iter() {
            min_x = min_x.min(point.x());
            min_y = min_y.min(point.y());
            max_x = max_x.max(point.x());
            max_y = max_y.max(point.y());
        }
    }

    let margin = (max_x - min_x).max(max_y - min_y) * 0.05;
    let mut svg = String::new();

    writeln!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}">"#,
        min_x - margin,
        min_y - margin,
        (max_x - min_x) + margin * 2.0,
        (max_y - min_y) + margin * 2.0
    )
    .unwrap();

    writeln!(
        svg,
        r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#1a3a1a"/>"##,
        min_x - margin,
        min_y - margin,
        (max_x - min_x) + margin * 2.0,
        (max_y - min_y) + margin * 2.0
    )
    .unwrap();

    for path in paths.iter() {
        let mut points = path.iter();
        if let Some(first) = points.next() {
            let mut d = format!("M {} {}", first.x(), first.y());
            for point in points {
                write!(d, " L {} {}", point.x(), point.y()).unwrap();
            }
            d.push_str(" Z");

            writeln!(svg, r##"<path d="{}" fill="#B87333" fill-opacity="0.6" stroke="#B87333" stroke-width="0.05"/>"##, d).unwrap();
        }
    }

    writeln!(svg, "</svg>").unwrap();
    std::fs::write(output, svg)?;
    log::info!("SVG exported to {}", output.display());
    Ok(())
}

pub fn write_string_to_file(string: &String, output: &Path) -> std::io::Result<()> {
    std::fs::write(output, string)?;
    log::info!("File wrote to {}", output.display());
    Ok(())
}
