use std::path::PathBuf;

use rust_i18n::t;
use sanitize_filename::Options;

use crate::{
    core::{
        toolpath::{generate_drill_gcode, generate_isolation_gcode},
        tools::{CncTool, library::ToolLibrary},
    },
    ui::project::{CommonParameters, OperationKind, SelectedTool, SourceFile},
};

impl SourceFile {
    pub(crate) fn export_gcode(
        &self,
        folder: &PathBuf,
        tool_library: &ToolLibrary,
        common: &CommonParameters,
    ) {
        for op in self.operations.iter() {
            let gcode_file = folder.join(sanitize_filename::sanitize_with_options(
                format!("{} - {}.nc", self.name, op.name),
                Options {
                    replacement: "_",
                    ..Default::default()
                },
            ));

            match op.kind {
                OperationKind::Isolation {
                    passes: _,
                    overlap: _,
                    offset: _,
                } => {
                    let tool = match op.tool {
                        SelectedTool::VBit(id) => {
                            tool_library.vbits.get(id).map(|t| t as &dyn CncTool)
                        }
                        SelectedTool::EndMill(id) => {
                            tool_library.end_mills.get(id).map(|t| t as &dyn CncTool)
                        }
                        SelectedTool::Drill(_) | SelectedTool::None => None,
                    };

                    if tool.is_none() {
                        log::error!("{}", t!("ui.error.bad_isolation_tool"));
                        continue;
                    }

                    let gcode = generate_isolation_gcode(
                        &op.toolpath,
                        tool.unwrap(),
                        op.depth,
                        common.z_safe,
                        common.z_finish,
                    );

                    if let Err(e) = std::fs::write(&gcode_file, gcode) {
                        log::error!("{}", t!("ui.error.fail_write_isolation", e = e.to_string()))
                    }
                }

                OperationKind::Outline => {
                    let tool = match op.tool {
                        SelectedTool::EndMill(id) => {
                            tool_library.end_mills.get(id).map(|t| t as &dyn CncTool)
                        }
                        SelectedTool::VBit(_) | SelectedTool::Drill(_) | SelectedTool::None => None,
                    };

                    if tool.is_none() {
                        log::error!("{}", t!("ui.error.bad_outline_tool"));
                        continue;
                    }

                    let gcode = generate_isolation_gcode(
                        &op.toolpath,
                        tool.unwrap(),
                        op.depth,
                        common.z_safe,
                        common.z_finish,
                    );

                    if let Err(e) = std::fs::write(&gcode_file, gcode) {
                        log::error!("{}", t!("ui.error.fail_write_outline", e = e.to_string()))
                    }
                }

                OperationKind::Drill { peck_step } => {
                    let tool = match op.tool {
                        SelectedTool::Drill(id) => {
                            tool_library.drill_bits.get(id).map(|t| t as &dyn CncTool)
                        }
                        SelectedTool::VBit(_) | SelectedTool::EndMill(_) | SelectedTool::None => {
                            None
                        }
                    };

                    if tool.is_none() {
                        log::error!("{}", t!("ui.error.bad_drill_tool"));
                        continue;
                    }

                    let gcode = generate_drill_gcode(
                        self.source.as_drill().as_ref().unwrap(),
                        tool.unwrap(),
                        op.depth,
                        common.z_safe,
                        common.z_finish,
                        peck_step,
                    );

                    if let Err(e) = std::fs::write(&gcode_file, gcode) {
                        log::error!("{}", t!("ui.error.fail_write_drill", e = e.to_string()))
                    }
                }
            }
        }
    }
}
