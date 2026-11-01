use egui::{Color32, Vec2};
use elegance::{Accent, Button};
use serde::{Deserialize, Serialize};

use crate::{
    core::bounding_box::BoundingBox,
    ui::{
        CopperCrabApp,
        colors::*,
        panels::left::FileOperationId,
        pcb_transform::PcbTransform,
        project::SourceKind,
        widget::canvas::{draw_axes, draw_paths, draw_paths_stroke},
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CanvasState {
    zoom: f32,
    offset: egui::Vec2,
    rect: egui::Rect,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            offset: egui::Vec2::ZERO,
            rect: egui::Rect {
                min: Default::default(),
                max: Default::default(),
            },
        }
    }
}

impl CopperCrabApp {
    pub(crate) fn ui_canvas(&mut self, ui: &mut egui::Ui) {
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

        let rect = response.rect;

        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(13, 26, 13));

        let transform = PcbTransform::new(rect, self.canvas.zoom, self.canvas.offset);

        draw_axes(&painter, &transform);

        let (sel_file_id, sel_op_id) = self
            .tree_files_states
            .selected()
            .first()
            .and_then(|f| match f {
                FileOperationId::Operation(file, op) => Some((*file, *op)),
                _ => None,
            })
            .unwrap_or((usize::MAX, usize::MAX));

        for (file_id, file) in self.files.iter().enumerate() {
            if file.is_visible == false {
                continue;
            }

            match &file.source {
                SourceKind::Copper(copper) => {
                    draw_paths(&painter, &copper.traces, &transform, COPPER, 1.0);

                    for (op_id, operation) in file.operations.iter().enumerate() {
                        let color = if file_id == sel_file_id && op_id == sel_op_id {
                            SELECTED_PATH
                        } else {
                            COPPER_PATH
                        };

                        for paths in &operation.toolpath {
                            draw_paths_stroke(&painter, paths, &transform, color, 1.0);
                        }
                    }
                }

                SourceKind::Outline(outline) => {
                    draw_paths_stroke(&painter, &outline.traces, &transform, OUTLINE, 1.0);

                    for operation in &file.operations {
                        for paths in &operation.toolpath {
                            draw_paths_stroke(&painter, paths, &transform, OUTLINE_PATH, 1.0);
                        }
                    }
                }

                SourceKind::Drill(drill) => {
                    for hole in &drill.holes {
                        painter.circle_filled(
                            transform.to_screen(hole.x, hole.y),
                            (hole.diameter / 2.0) as f32 * self.canvas.zoom,
                            Color32::WHITE,
                        );
                    }
                }
            }
        }

        // zoom/pan
        if response.dragged() {
            self.canvas.offset += response.drag_delta();
        }

        if let Some(pos) = response.hover_pos() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                let zoom_factor = 1.0 + scroll * 0.001;
                let cursor_in_canvas = pos - rect.center();

                self.zoom_on_position(zoom_factor, cursor_in_canvas);
            }

            let pcb_x = (pos.x - transform.rect.min.x - transform.offset.x) / transform.scale;
            let pcb_y = -(pos.y - transform.rect.min.y - transform.offset.y) / transform.scale;
            self.status_bar.set_mouse_pcb(pcb_x, pcb_y);
            self.canvas.rect = rect;
        }

        egui::Area::new(egui::Id::new("canvas_controls"))
            .fixed_pos(egui::pos2(rect.max.x - 110.0, rect.max.y - 40.0))
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add(Button::icon('−', "").accent(Accent::Purple))
                        .clicked()
                    {
                        self.zoom_on_position(0.95, Vec2::ZERO);
                    }
                    if ui
                        .add(Button::icon('+', "").accent(Accent::Purple))
                        .clicked()
                    {
                        self.zoom_on_position(1.05, Vec2::ZERO);
                    }
                    if ui
                        .add(Button::icon('⊡', "").accent(Accent::Purple))
                        .clicked()
                    {
                        // Reset zoom et offset
                        self.center_view_to_board();
                    }
                });
            });
    }

    fn zoom_on_position(&mut self, zoom_factor: f32, position: Vec2) {
        let old_zoom = self.canvas.zoom;
        self.canvas.zoom *= zoom_factor;

        self.canvas.offset =
            position + (self.canvas.offset - position) * (self.canvas.zoom / old_zoom);
    }

    pub(crate) fn get_bounding_box(&self) -> BoundingBox {
        let mut b_box = BoundingBox::new();

        for file in &self.files {
            match &file.source {
                SourceKind::Copper(toolpath_generator) => {
                    b_box.update_from_bounds(&toolpath_generator.traces.bounds())
                }
                SourceKind::Outline(toolpath_generator) => {
                    b_box.update_from_bounds(&toolpath_generator.traces.bounds())
                }
                SourceKind::Drill(drill_layer) => {
                    for p in drill_layer.holes.iter() {
                        b_box.update_from_point(p.x, p.y);
                    }
                }
            }

            for op in &file.operations {
                for p in &op.toolpath {
                    b_box.update_from_bounds(&p.bounds());
                }
            }
        }

        b_box
    }

    pub(crate) fn center_view_to_board(&mut self) {
        let b_box = self.get_bounding_box();

        if b_box.is_infinite() {
            self.canvas.offset.x = 0.0;
            self.canvas.offset.y = 0.0;
            self.canvas.zoom = 1.0;
        } else {
            let (min_x, min_y, max_x, max_y) = b_box.to_tuples();
            let pcb_w = (max_x - min_x) as f32;
            let pcb_h = (max_y - min_y) as f32;

            if pcb_w > 0.0 && pcb_h > 0.0 {
                let scale_x = self.canvas.rect.width() * 0.9 / pcb_w;
                let scale_y = self.canvas.rect.height() * 0.9 / pcb_h;
                self.canvas.zoom = scale_x.min(scale_y);
            } else {
                self.canvas.zoom = 1.0;
            }

            let pcb_cx = ((min_x + max_x) / 2.0) as f32;
            let pcb_cy = ((min_y + max_y) / 2.0) as f32;

            self.canvas.offset.x = -pcb_cx * self.canvas.zoom;
            self.canvas.offset.y = pcb_cy * self.canvas.zoom;
        }
    }
}
