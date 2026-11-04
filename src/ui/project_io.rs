use std::path::Path;

use crate::{
    core::{
        geometry::Point2d,
        parser::{excellon::load_excellon, gerber::load_gerber},
        toolpath::ToolpathGenerator,
    },
    ui::{
        CopperCrabApp,
        project::{PROJECT_VERSION, ProjectFile, SourceFile, SourceKind, SourceKindTag},
    },
};

impl CopperCrabApp {
    pub(crate) fn new_project(&mut self) {
        self.files.clear();
        self.tree_files_states = Default::default();
        self.app_config.common_parameters = Default::default();
        self.current_project_path = None;
    }

    pub(crate) fn close_project(&mut self) {
        if self.files.is_empty() {
            self.current_project_path = None;
            return;
        }

        self.files.clear();
        self.tree_files_states = Default::default();
        self.current_project_path = None;

        self.canvas = Default::default();

        log::info!("Project closed");
    }

    pub(crate) fn save_project_to(&self, path: &Path) -> Result<(), String> {
        let mut files = Vec::new();
        for f in &self.files {
            match f.to_snapshot() {
                Some(snap) => files.push(snap),
                None => log::warn!(
                    "File '{}' has no source path — skipped when saving project",
                    f.name
                ),
            }
        }

        let project = ProjectFile {
            version: PROJECT_VERSION,
            common_parameters: self.app_config.common_parameters.clone(),
            files,
        };

        project.save(path).map_err(|e| e.to_string())
    }

    pub(crate) fn load_project_from(&mut self, path: &Path) -> Result<(), String> {
        let project = ProjectFile::load(path).map_err(|e| e.to_string())?;

        self.app_config.common_parameters = project.common_parameters;
        self.files.clear();
        self.tree_files_states = Default::default();

        for snap in project.files {
            let mut source = match snap.kind {
                SourceKindTag::Copper => {
                    let layer = load_gerber(&snap.path)
                        .map_err(|e| format!("{}: {}", snap.path.display(), e))?;
                    let generator =
                        ToolpathGenerator::new(&layer, false).map_err(|e| e.to_string())?;
                    SourceKind::Copper { generator, layer }
                }
                SourceKindTag::Outline => {
                    let layer = load_gerber(&snap.path)
                        .map_err(|e| format!("{}: {}", snap.path.display(), e))?;
                    let generator =
                        ToolpathGenerator::new(&layer, true).map_err(|e| e.to_string())?;
                    SourceKind::Outline(generator)
                }
                SourceKindTag::Drill => {
                    let layer = load_excellon(&snap.path)
                        .map_err(|e| format!("{}: {}", snap.path.display(), e))?;
                    SourceKind::Drill(layer)
                }
            };

        
            if !snap.transform.is_identity() {
                source.apply_saved_transform(&snap.transform);
            }

            let mut file = SourceFile::new(snap.name, source, Some(snap.path));
            file.is_visible = snap.is_visible;
            file.operations = snap.operations;
            file.next_id_operation = snap.next_id_operation;
            file.hidden_primitives = snap.hidden_primitives.into_iter().collect();
            file.transform = snap.transform;

            self.files.push(file);
        }

        for id in 0..self.files.len() {
            self.generate_file_toolpaths(id);
        }

        self.center_view_to_board();
        Ok(())
    }
}
