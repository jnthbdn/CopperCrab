use egui::{Layout, RichText, Sense};

use crate::{
    core::tools::{drill::DrillBit, endmill::EndMill, library::ToolLibrary, vbit::VBit},
    ui::buttons::{button, button_danger, button_primary},
};

#[derive(Debug)]
struct ToolEditor {
    pub editing_index: Option<usize>,
    pub tool_type: ToolType,

    pub name: String,
    pub angle_deg: f64,
    pub tip_diameter: f64,
    pub diameter: f64,
    pub max_depth: f64,
    pub feed_rate: f64,
    pub plunge_rate: f64,
    pub spindle_speed: f64,
}

#[derive(Debug, PartialEq)]
enum ToolType {
    VBit,
    EndMill,
    DrillBit,
}

impl ToolEditor {
    pub fn new_vbit() -> Self {
        Self {
            editing_index: None,
            tool_type: ToolType::VBit,
            name: "New V-Bit".to_string(),
            angle_deg: 60.0,
            tip_diameter: 0.1,
            diameter: 1.0,
            max_depth: 0.5,
            feed_rate: 400.0,
            plunge_rate: 80.0,
            spindle_speed: 20_000.0,
        }
    }

    pub fn from_vbit(index: usize, vbit: &VBit) -> Self {
        Self {
            editing_index: Some(index),
            tool_type: ToolType::VBit,
            name: vbit.name.clone(),
            angle_deg: vbit.angle_deg,
            tip_diameter: vbit.tip_diameter,
            diameter: 1.0,
            max_depth: vbit.max_depth,
            feed_rate: vbit.feed_rate,
            plunge_rate: vbit.plunge_rate,
            spindle_speed: vbit.spindle_speed,
        }
    }

    pub fn new_end_mill() -> Self {
        Self {
            editing_index: None,
            tool_type: ToolType::EndMill,
            name: "New End Mill".to_string(),
            angle_deg: 0.0,
            tip_diameter: 0.0,
            diameter: 1.0,
            max_depth: 3.0,
            feed_rate: 600.0,
            plunge_rate: 100.0,
            spindle_speed: 15_000.0,
        }
    }

    pub fn from_end_mill(index: usize, end_mill: &EndMill) -> Self {
        Self {
            editing_index: Some(index),
            tool_type: ToolType::EndMill,
            name: end_mill.name.clone(),
            angle_deg: 0.0,
            tip_diameter: 0.0,
            diameter: end_mill.diameter,
            max_depth: end_mill.max_depth,
            feed_rate: end_mill.feed_rate,
            plunge_rate: end_mill.plunge_rate,
            spindle_speed: end_mill.spindle_speed,
        }
    }

    pub fn new_drill_bit() -> Self {
        Self {
            editing_index: None,
            tool_type: ToolType::DrillBit,
            name: "New Drill Bit".to_string(),
            angle_deg: 0.0,
            tip_diameter: 0.0,
            diameter: 0.8,
            max_depth: 2.0,
            feed_rate: 0.0,
            plunge_rate: 60.0,
            spindle_speed: 8_000.0,
        }
    }

    pub fn from_drill_bit(index: usize, drill_bit: &DrillBit) -> Self {
        Self {
            editing_index: Some(index),
            tool_type: ToolType::DrillBit,
            name: drill_bit.name.clone(),
            angle_deg: 0.0,
            tip_diameter: 0.0,
            diameter: drill_bit.diameter,
            max_depth: drill_bit.max_depth,
            feed_rate: 0.0,
            plunge_rate: drill_bit.plunge_rate,
            spindle_speed: drill_bit.spindle_speed,
        }
    }
}

#[derive(Debug)]
pub struct ToolLibraryUi {
    tool_editor: Option<ToolEditor>,
    img_vbit_big_scale: bool,
}

const IMG_VBIT: egui::ImageSource<'static> = egui::include_image!("../../img/vbit_diagram.png");

impl ToolLibraryUi {
    pub fn show(&mut self, ctx: &egui::Context, open: &mut bool, tool_library: &mut ToolLibrary) {
        if *open {
            let modal = egui::Modal::new(egui::Id::new("modal_tool_library"))
                .frame(
                    egui::Frame::popup(&ctx.global_style())
                        .inner_margin(egui::Margin::symmetric(16, 16)),
                )
                .show(ctx, |ui| {
                    ui.set_min_width(ctx.viewport_rect().width() * 0.9);
                    ui.set_max_width(ctx.viewport_rect().width() * 0.9);
                    ui.set_min_height(ctx.viewport_rect().height() * 0.9);
                    ui.set_max_height(ctx.viewport_rect().height() * 0.9);

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.set_min_height(ui.available_height());
                        ui.columns(3, |columns| {
                            // V-Bits
                            columns[0].label("V-Bits");
                            columns[0].separator();

                            if button_primary(&mut columns[0], "+ Add V-Bit", true).clicked() {
                                self.tool_editor = Some(ToolEditor::new_vbit());
                            }
                            let mut to_delete = None;
                            for (i, vbit) in tool_library.vbits.iter().enumerate() {
                                columns[0].horizontal(|ui| {
                                    ui.label(egui::RichText::new(&vbit.name).strong());
                                    ui.label(format!("{}°", vbit.angle_deg));
                                    ui.label(format!("tip: {}mm", vbit.tip_diameter));
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if button_danger(ui, "🗑", false).clicked() {
                                                to_delete = Some(i);
                                            }
                                            if button(ui, "✏", false).clicked() {
                                                self.tool_editor =
                                                    Some(ToolEditor::from_vbit(i, vbit));
                                            }
                                        },
                                    );
                                });
                            }
                            if let Some(i) = to_delete {
                                tool_library.vbits.remove(i);
                            }

                            // End Mills
                            columns[1].label("End Mills");
                            columns[1].separator();

                            if button_primary(&mut columns[1], "+ Add End Mill", true).clicked() {
                                self.tool_editor = Some(ToolEditor::new_end_mill());
                            }
                            let mut to_delete = None;
                            for (i, end_mill) in tool_library.end_mills.iter().enumerate() {
                                columns[1].horizontal(|ui| {
                                    ui.label(egui::RichText::new(&end_mill.name).strong());
                                    ui.label(format!("dia: {}mm", end_mill.diameter));
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if button_danger(ui, "🗑", false).clicked() {
                                                to_delete = Some(i);
                                            }
                                            if button(ui, "✏", false).clicked() {
                                                self.tool_editor =
                                                    Some(ToolEditor::from_end_mill(i, end_mill));
                                            }
                                        },
                                    );
                                });
                            }
                            if let Some(i) = to_delete {
                                tool_library.end_mills.remove(i);
                            }

                            // Drill Bits
                            columns[2].label("Drill Bits");
                            columns[2].separator();

                            if button_primary(&mut columns[2], "+ Add Drill Bit", true).clicked() {
                                self.tool_editor = Some(ToolEditor::new_drill_bit());
                            }
                            let mut to_delete = None;
                            for (i, drill_bit) in tool_library.drill_bits.iter().enumerate() {
                                columns[2].horizontal(|ui| {
                                    ui.label(egui::RichText::new(&drill_bit.name).strong());
                                    ui.label(format!("dia: {}mm", drill_bit.diameter));
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if button_danger(ui, "🗑", false).clicked() {
                                                to_delete = Some(i);
                                            }
                                            if button(ui, "✏", false).clicked() {
                                                self.tool_editor =
                                                    Some(ToolEditor::from_drill_bit(i, drill_bit));
                                            }
                                        },
                                    );
                                });
                            }
                            if let Some(i) = to_delete {
                                tool_library.drill_bits.remove(i);
                            }
                        });
                    });
                    ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                        if button_primary(ui, "Close", false).clicked() {
                            *open = false;
                        }
                    });
                });

            if modal.should_close() {
                *open = false;
            }
        }

        self.ui_tool_editor(ctx, tool_library);
    }

    fn ui_tool_editor(&mut self, ctx: &egui::Context, tool_library: &mut ToolLibrary) {
        let open = self.tool_editor.is_some();

        if open {
            let modal = egui::Modal::new(egui::Id::new("modal_tool_editor")).show(ctx, |ui| {
                if let Some(editor) = &mut self.tool_editor {
                    ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                        egui::Grid::new("parameter_grid")
                            .num_columns(2)
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Name");
                                ui.text_edit_singleline(&mut editor.name)
                                    .on_hover_text("Name displayed in the tool library");
                                ui.end_row();

                                match editor.tool_type {
                                    ToolType::VBit => {
                                        ui.label("Angle");
                                        ui.add(
                                            egui::DragValue::new(&mut editor.angle_deg)
                                                .range(10.0..=180.0)
                                                .suffix(" °"),
                                        )
                                        .on_hover_text("Opening angle of the V in degrees. Determines cutting width based on depth. Typical values: 10°, 30°, 60° or 90°");
                                        ui.end_row();

                                        ui.label("Tip diameter");
                                        ui.add(
                                            egui::DragValue::new(&mut editor.tip_diameter)
                                                .speed(0.01)
                                                .suffix(" mm"),
                                        )
                                        .on_hover_text("Diameter of the V-bit tip. A 0.0mm tip is theoretical — in practice 0.1mm minimum to avoid breakage");
                                        ui.end_row();
                                    }
                                    ToolType::EndMill | ToolType::DrillBit => {
                                        ui.label("Diameter");
                                        ui.add(
                                            egui::DragValue::new(&mut editor.diameter)
                                                .speed(0.01)
                                                .suffix(" mm"),
                                        )
                                        .on_hover_text("Tool diameter. Directly corresponds to the cutting width for end mills, and hole diameter for drill bits");

                                        ui.end_row();
                                    }
                                }

                                ui.label("Max depth");
                                ui.add(
                                    egui::DragValue::new(&mut editor.max_depth)
                                        .speed(0.01)
                                        .suffix(" mm"),
                                )
                                .on_hover_text("Maximum depth per pass. The G-code generator automatically splits into multiple passes if the total depth exceeds this value");
                                ui.end_row();

                                if editor.tool_type != ToolType::DrillBit {
                                    ui.label("Feed rate");
                                    ui.add(
                                        egui::DragValue::new(&mut editor.feed_rate)
                                            .speed(1.0)
                                            .suffix(" mm/min"),
                                    )
                                    .on_hover_text("XY movement speed during cutting in mm/min. Too high a value degrades cut quality and risks breaking the tool");
                                    ui.end_row();
                                }

                                ui.label("Plunge rate");
                                ui.add(
                                    egui::DragValue::new(&mut editor.plunge_rate)
                                        .speed(1.0)
                                        .suffix(" mm/min"),
                                )
                                .on_hover_text("Z plunge speed in mm/min. Should be significantly lower than feed_rate to protect the tool when entering the material");
                                ui.end_row();

                                ui.label("Spindle speed");
                                ui.add(
                                    egui::DragValue::new(&mut editor.spindle_speed)
                                        .range(0..=100_000)
                                        .speed(1.0)
                                        .suffix(" RPM"),
                                )
                                .on_hover_text("Spindle rotation speed in RPM. V-bit: ~20000, End mill: ~15000, Drill: ~8000. Depends on tool diameter and material");
                                ui.end_row();
                            });

                        if editor.tool_type == ToolType::VBit {
                            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {

                                ui.set_max_width(0.0);
                                ui.set_min_width(0.0);

                                if ui.add(egui::Image::new(IMG_VBIT.clone() )
                                        .fit_to_original_size(
                                        if self.img_vbit_big_scale { 1.0 } else { 0.5 }
                                        )
                                        .sense(Sense::click() )
                                    )
                                    .clicked() {
                                    self.img_vbit_big_scale = !self.img_vbit_big_scale;
                                }

                                ui.label(RichText::new("Click to enlarge").italics());
                                ui.shrink_width_to_current();
                                ui.request_repaint();
                            });
                        }
                    });

                    ui.set_max_width(0.0);
                    ui.set_min_width(0.0);
                    ui.shrink_width_to_current();

                    ui.separator();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        if button_primary(ui, "Save", false).clicked() {
                            if let Some(editor) = &self.tool_editor {
                                match editor.tool_type {
                                    ToolType::VBit => {
                                        let vbit = VBit {
                                            name: editor.name.clone(),
                                            angle_deg: editor.angle_deg,
                                            tip_diameter: editor.tip_diameter,
                                            max_depth: editor.max_depth,
                                            feed_rate: editor.feed_rate,
                                            plunge_rate: editor.plunge_rate,
                                            spindle_speed: editor.spindle_speed,
                                        };
                                        match editor.editing_index {
                                            Some(i) => tool_library.vbits[i] = vbit,
                                            None => tool_library.vbits.push(vbit),
                                        }
                                    }
                                    ToolType::EndMill => {
                                        let endmill = EndMill {
                                            name: editor.name.clone(),
                                            diameter: editor.diameter,
                                            max_depth: editor.max_depth,
                                            feed_rate: editor.feed_rate,
                                            plunge_rate: editor.plunge_rate,
                                            spindle_speed: editor.spindle_speed,
                                        };
                                        match editor.editing_index {
                                            Some(i) => tool_library.end_mills[i] = endmill,
                                            None => tool_library.end_mills.push(endmill),
                                        }
                                    }
                                    ToolType::DrillBit => {
                                        let drill = DrillBit {
                                            name: editor.name.clone(),
                                            diameter: editor.diameter,
                                            max_depth: editor.max_depth,
                                            plunge_rate: editor.plunge_rate,
                                            spindle_speed: editor.spindle_speed,
                                        };
                                        match editor.editing_index {
                                            Some(i) => tool_library.drill_bits[i] = drill,
                                            None => tool_library.drill_bits.push(drill),
                                        }
                                    }
                                }
                                self.tool_editor = None;

                                if let Err(e) = tool_library.save() {
                                    log::warn!("Unable to save the tool library. {e}");
                                };
                            }
                        }
                        if button(ui, "Cancel", false).clicked() {
                            self.tool_editor = None;
                        }
                    });
                }
            });

            if modal.should_close() {
                self.tool_editor = None;
                self.img_vbit_big_scale = false;
            }
        }

        if !open {
            self.tool_editor = None;
        }
    }
}

impl Default for ToolLibraryUi {
    fn default() -> Self {
        Self {
            tool_editor: None,
            img_vbit_big_scale: false,
        }
    }
}
