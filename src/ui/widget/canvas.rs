use std::sync::Arc;

use egui::epaint;
use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::*;

use crate::ui::pcb_transform::PcbTransform;

pub fn draw_axes(painter: &egui::Painter, transform: &PcbTransform) {
    let origin = transform.to_screen(0.0, 0.0);
    let rect = transform.rect;

    // Axe X — rouge, de gauche à droite du canvas
    painter.line_segment(
        [
            egui::pos2(rect.min.x, origin.y),
            egui::pos2(rect.max.x, origin.y),
        ],
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 0, 0, 100)),
    );

    // Axe Y — vert, de haut en bas du canvas
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
