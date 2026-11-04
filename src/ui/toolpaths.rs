use crate::{
    core::{PcbLayer, PcbTrace, toolpath::ToolpathGenerator, tools::CncTool},
    ui::{
        CopperCrabApp,
        project::{OperationKind, SelectedTool, SourceKind},
    },
};

impl CopperCrabApp {
    pub(crate) fn generate_file_toolpaths(&mut self, id_file: usize) {
        let nb_ops = self.files.get(id_file).map_or(0, |f| f.operations.len());

        for id_op in 0..nb_ops {
            self.generate_operation_toolpaths(id_file, id_op);
        }
    }

    pub(crate) fn generate_operation_toolpaths(&mut self, id_file: usize, id_op: usize) {
        if let Some(file) = self.files.get_mut(id_file) {
            if let Some(op) = file.operations.get_mut(id_op) {
                match op.kind {
                    OperationKind::Isolation {
                        passes,
                        overlap,
                        offset,
                    } => {
                        let tool: Option<&dyn CncTool> = match op.tool {
                            SelectedTool::VBit(id) => self
                                .tools
                                .tool_library
                                .vbits
                                .get(id)
                                .map(|t| t as &dyn CncTool),
                            SelectedTool::EndMill(id) => self
                                .tools
                                .tool_library
                                .end_mills
                                .get(id)
                                .map(|t| t as &dyn CncTool),
                            SelectedTool::Drill(_) | SelectedTool::None => None,
                        };

                        if let Some(tool) = tool {
                            if let SourceKind::Copper { layer, .. } = &file.source {
                                let mut filtered_layer = PcbLayer {
                                    unit: layer.unit.clone(),
                                    traces: Vec::new(),
                                    aperture_macro: layer.aperture_macro.clone(),
                                };

                                for (t_idx, trace) in layer.traces.iter().enumerate() {
                                    let mut new_trace = PcbTrace {
                                        primitives: Vec::new(),
                                    };
                                    for (p_idx, prim) in trace.primitives.iter().enumerate() {
                                        if !file.hidden_primitives.contains(&(t_idx, p_idx)) {
                                            new_trace.primitives.push(prim.clone());
                                        }
                                    }
                                    if !new_trace.primitives.is_empty() {
                                        filtered_layer.traces.push(new_trace);
                                    }
                                }

                                if let Ok(generator) =
                                    ToolpathGenerator::new(&filtered_layer, false)
                                {
                                    op.toolpath = generator.generate_toolpaths(
                                        tool,
                                        op.depth,
                                        passes,
                                        overlap / 100.0,
                                        offset,
                                    );
                                } else {
                                    op.toolpath = Vec::new();
                                }
                            } else {
                                op.toolpath = Vec::new();
                            }
                        } else {
                            op.toolpath = Vec::new();
                        }
                    }
                    OperationKind::Outline => {
                        let tool: Option<&dyn CncTool> = match op.tool {
                            SelectedTool::EndMill(id) => self
                                .tools
                                .tool_library
                                .end_mills
                                .get(id)
                                .map(|t| t as &dyn CncTool),
                            SelectedTool::VBit(_) | SelectedTool::Drill(_) | SelectedTool::None => {
                                None
                            }
                        };

                        if let Some(tool) = tool {
                            op.toolpath = file
                                .source
                                .as_outline()
                                .unwrap()
                                .generate_toolpaths(tool, op.depth, 1, 1.0, 0.0);
                        } else {
                            op.toolpath = Vec::new();
                        }
                    }
                    OperationKind::Drill { peck_step: _ } => (),
                };
            }
        }
    }
}
