use std::collections::HashMap;
use std::path::Path;

use rust_i18n::t;

use crate::core::parser::ParseError;
use crate::core::{DrillHole, DrillLayer};

pub fn load_excellon(path: &Path) -> Result<DrillLayer, ParseError> {
    log::info!(
        "{}",
        t!("excellon.info.load_file", path = path.display().to_string())
    );

    let content = std::fs::read_to_string(path)?;
    let mut tools: HashMap<u32, f64> = HashMap::new();
    let mut current_tool_diameter: Option<f64> = None;
    let mut holes: Vec<DrillHole> = Vec::new();
    let mut in_header = true;

    for line in content.lines() {
        let line = line.trim();

        // Ignore comments and empty line
        if line.starts_with(';') || line.is_empty() {
            continue;
        }

        // header end
        if line == "%" {
            in_header = false;
            continue;
        }

        if in_header {
            // Tool definition : T1C0.800
            if let Some(rest) = line.strip_prefix('T') {
                if let Some((tool_id, diameter)) = parse_tool_definition(rest) {
                    tools.insert(tool_id, diameter);
                }
            }
            continue;
        }

        // Body - Tool selection
        if line.starts_with('T') {
            if let Ok(id) = line[1..].parse::<u32>() {
                current_tool_diameter = tools.get(&id).copied();
            }
            continue;
        }

        // Hole coord. : X142.0Y-67.92
        if line.starts_with('X') {
            if let Some(diameter) = current_tool_diameter {
                if let Some((x, y)) = parse_xy(line) {
                    holes.push(DrillHole { x, y, diameter });
                }
            }
            continue;
        }

        // End of file
        if line == "M30" || line == "M02" {
            break;
        }
    }

    log::info!("{}", t!("excellon.info.loaded", holes = holes.len()));
    Ok(DrillLayer { holes })
}

fn parse_tool_definition(s: &str) -> Option<(u32, f64)> {
    // "1C0.800" → (1, 0.8)
    let c_pos = s.find('C')?;
    let id = s[..c_pos].parse::<u32>().ok()?;
    let diameter = s[c_pos + 1..].parse::<f64>().ok()?;
    Some((id, diameter))
}

fn parse_xy(s: &str) -> Option<(f64, f64)> {
    // "X142.0Y-67.92" → (142.0, -67.92)
    let y_pos = s.find('Y')?;
    let x = s[1..y_pos].parse::<f64>().ok()?;
    let y = s[y_pos + 1..].parse::<f64>().ok()?;
    Some((x, y))
}
