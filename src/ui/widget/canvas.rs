use std::sync::Arc;

use egui::epaint;
use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::*;

use crate::core::toolpath::helpers::{
    arc_to_path, circle_to_path, complex_to_path, polygon_to_paths, rectangle_to_path,
    segment_to_path,
};
use crate::core::Primitive;
use crate::ui::pcb_transform::PcbTransform;

const GRID_STEP_MM: f64 = 5.0;
const GRID_MIN_PIXELS: f32 = 8.0;

pub fn draw_axes(painter: &egui::Painter, transform: &PcbTransform) {
    let origin = transform.to_screen(0.0, 0.0);
    let rect = transform.rect;

    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }

    let grid_color = egui::Color32::from_rgba_unmultiplied(120, 120, 120, 60);
    let grid_stroke = egui::Stroke::new(1.0, grid_color);

    let step_px = (GRID_STEP_MM as f32) * transform.scale;

    if step_px >= GRID_MIN_PIXELS {
        let mut x_mm = 0.0;
        while transform.to_screen(-x_mm, 0.0).x > rect.min.x {
            let screen_x = transform.to_screen(-x_mm, 0.0).x;
            painter.line_segment(
                [
                    egui::pos2(screen_x, rect.min.y),
                    egui::pos2(screen_x, rect.max.y),
                ],
                grid_stroke,
            );
            x_mm += GRID_STEP_MM;
        }
        x_mm = GRID_STEP_MM;
        while transform.to_screen(x_mm, 0.0).x < rect.max.x {
            let screen_x = transform.to_screen(x_mm, 0.0).x;
            painter.line_segment(
                [
                    egui::pos2(screen_x, rect.min.y),
                    egui::pos2(screen_x, rect.max.y),
                ],
                grid_stroke,
            );
            x_mm += GRID_STEP_MM;
        }

        let mut y_mm = 0.0;
        while transform.to_screen(0.0, -y_mm).y < rect.max.y {
            let screen_y = transform.to_screen(0.0, -y_mm).y;
            painter.line_segment(
                [
                    egui::pos2(rect.min.x, screen_y),
                    egui::pos2(rect.max.x, screen_y),
                ],
                grid_stroke,
            );
            y_mm += GRID_STEP_MM;
        }
        y_mm = GRID_STEP_MM;
        while transform.to_screen(0.0, y_mm).y > rect.min.y {
            let screen_y = transform.to_screen(0.0, y_mm).y;
            painter.line_segment(
                [
                    egui::pos2(rect.min.x, screen_y),
                    egui::pos2(rect.max.x, screen_y),
                ],
                grid_stroke,
            );
            y_mm += GRID_STEP_MM;
        }
    }

    painter.line_segment(
        [
            egui::pos2(rect.min.x, origin.y),
            egui::pos2(rect.max.x, origin.y),
        ],
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 0, 0, 100)),
    );

    painter.line_segment(
        [
            egui::pos2(origin.x, rect.min.y),
            egui::pos2(origin.x, rect.max.y),
        ],
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 255, 0, 100)),
    );
}

pub fn draw_paths(
    painter: &egui::Painter,
    paths: &clipper2::Paths,
    transform: &PcbTransform,
    color: egui::Color32,
    opacity: f32,
) {
    let color = egui::Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        (opacity * 255.0) as u8,
    );

    for path in paths.iter() {
        let points: Vec<egui::Pos2> = path
            .iter()
            .map(|p| transform.to_screen(p.x(), p.y()))
            .collect();

        if points.len() < 3 {
            continue;
        }

        let mesh = tessellate_path(&points, color);
        painter.add(egui::Shape::Mesh(Arc::new(mesh)));
    }
}

pub fn draw_paths_stroke(
    painter: &egui::Painter,
    paths: &clipper2::Paths,
    transform: &PcbTransform,
    color: egui::Color32,
    opacity: f32,
) {
    let color = egui::Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        (opacity * 255.0) as u8,
    );

    for path in paths.iter() {
        let points: Vec<egui::Pos2> = path
            .iter()
            .map(|p| transform.to_screen(p.x(), p.y()))
            .collect();

        if points.len() < 3 {
            continue;
        }

        painter.add(egui::Shape::Path(epaint::PathShape {
            points,
            closed: true,
            fill: egui::Color32::TRANSPARENT,
            stroke: epaint::PathStroke::new(1.0, color),
        }));
    }
}

pub fn draw_primitive(
    painter: &egui::Painter,
    primitive: &Primitive,
    transform: &PcbTransform,
    color: egui::Color32,
) {


    let paths = match primitive {
        Primitive::Segment(s) => segment_to_path(s),
        Primitive::Arc(a) => arc_to_path(a, 32),
        Primitive::Circle(c) => circle_to_path(c, 32),
        Primitive::Rectangle(r) => rectangle_to_path(r),
        Primitive::Polygon(p) => polygon_to_paths(p),
        Primitive::Complex(children) => complex_to_path(children, 32),
    };

    draw_paths(painter, &paths, transform, color, 1.0);
}

fn point_to_segment_dist(px: f64, py: f64, a: (f64, f64), b: (f64, f64)) -> f64 {
    let (ax, ay) = a;
    let (bx, by) = b;
    let dx = bx - ax;
    let dy = by - ay;
    let len2 = dx * dx + dy * dy;
    if len2 < 1e-12 {
        return ((px - ax).powi(2) + (py - ay).powi(2)).sqrt();
    }
    let t = (((px - ax) * dx + (py - ay) * dy) / len2).clamp(0.0, 1.0);
    let cx = ax + t * dx;
    let cy = ay + t * dy;
    ((px - cx).powi(2) + (py - cy).powi(2)).sqrt()
}

pub fn hit_test_primitive(primitive: &Primitive, x: f64, y: f64, tolerance: f64) -> bool {
    match primitive {
        Primitive::Segment(s) => {
            let d = point_to_segment_dist(x, y, (s.start.x, s.start.y), (s.end.x, s.end.y));
            d <= tolerance + s.width / 2.0
        }
        Primitive::Circle(c) => {
            let d = ((x - c.center.x).powi(2) + (y - c.center.y).powi(2)).sqrt();
            (d - c.diameter / 2.0).abs() <= tolerance
        }
        Primitive::Polygon(points) => {
            if points.len() < 2 {
                return false;
            }
            for i in 0..points.len() {
                let a = points[i].to_tuple();
                let b = points[(i + 1) % points.len()].to_tuple();
                if point_to_segment_dist(x, y, a, b) <= tolerance {
                    return true;
                }
            }
            false
        }
        Primitive::Arc(a) => {
            let seg1 = point_to_segment_dist(x, y, a.start.to_tuple(), a.end.to_tuple());
            let dc = ((x - a.center.x).powi(2) + (y - a.center.y).powi(2)).sqrt();
            let r = ((a.start.x - a.center.x).powi(2) + (a.start.y - a.center.y).powi(2)).sqrt();
            seg1 <= tolerance || (dc - r).abs() <= tolerance
        }
        Primitive::Rectangle(r) => {
            let hw = r.width / 2.0;
            let hh = r.height / 2.0;
            (x - r.center.x).abs() <= hw + tolerance && (y - r.center.y).abs() <= hh + tolerance
        }
        Primitive::Complex(children) => children
            .iter()
            .any(|c| hit_test_primitive(c, x, y, tolerance)),
    }
}

fn tessellate_path(points: &[egui::Pos2], color: egui::Color32) -> egui::Mesh {
    let mut builder = Path::builder();

    if let Some(first) = points.first() {
        builder.begin(point(first.x, first.y));
        for p in points.iter().skip(1) {
            builder.line_to(point(p.x, p.y));
        }
        builder.end(true);
    }

    let path = builder.build();

    let mut geometry: VertexBuffers<egui::Pos2, u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    tessellator
        .tessellate_path(
            &path,
            &FillOptions::default(),
            &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                egui::pos2(vertex.position().x, vertex.position().y)
            }),
        )
        .unwrap();

    let mut mesh = egui::Mesh::default();
    for vertex in geometry.vertices {
        mesh.colored_vertex(vertex, color);
    }
    for index in geometry.indices {
        mesh.indices.push(index);
    }

    mesh
}