use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default,
)]
pub(crate) enum ExportFormat {
    #[default]
    GCode,
    Mach3,
}

impl ExportFormat {
    pub(crate) const ALL: &'static [ExportFormat] =
        &[ExportFormat::GCode, ExportFormat::Mach3];

    pub(crate) fn label(&self) -> &'static str {
        match self {
            ExportFormat::GCode => "G-Code",
            ExportFormat::Mach3 => "Mach3",
        }
    }

    pub(crate) fn ui_labels(&self) -> (String, String) {
        match self {
            ExportFormat::GCode | ExportFormat::Mach3 => (
                rust_i18n::t!("ui.label.export_settings.header").to_string(),
                rust_i18n::t!("ui.label.export_settings.footer").to_string(),
            ),
        }
    }

    pub(crate) fn wrap(&self, gcode: String, header: &str, footer: &str) -> String {
        match self {
            ExportFormat::GCode | ExportFormat::Mach3 => wrap_common(gcode, header, footer),
        }
    }
}

fn wrap_common(gcode: String, header: &str, footer: &str) -> String {
    let header = header.trim_end();
    let footer = footer.trim_start();

    let mut out = String::with_capacity(gcode.len() + header.len() + footer.len() + 4);

    if !header.is_empty() {
        out.push_str(header);
        out.push('\n');
    }

    out.push_str(&gcode);

    if !footer.is_empty() {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(footer);
    }

    out
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct FormatPayload {
    #[serde(default)]
    pub header: String,
    #[serde(default)]
    pub footer: String,
}

impl FormatPayload {
    pub(crate) fn wrap(&self, format: ExportFormat, gcode: String) -> String {
        format.wrap(gcode, &self.header, &self.footer)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct ExportSettings {
    #[serde(default)]
    pub format: ExportFormat,

    #[serde(default)]
    pub payloads: HashMap<ExportFormat, FormatPayload>,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub legacy_header: String,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub legacy_footer: String,
}

impl ExportSettings {
    pub(crate) fn payload(&self, format: ExportFormat) -> Option<&FormatPayload> {
        self.payloads.get(&format)
    }

    pub(crate) fn payload_mut(&mut self, format: ExportFormat) -> &mut FormatPayload {
        self.payloads.entry(format).or_default()
    }

    pub(crate) fn migrate_legacy(&mut self) {
        if self.legacy_header.is_empty() && self.legacy_footer.is_empty() {
            return;
        }
        let fmt = self.format;
        let payload = self.payloads.entry(fmt).or_default();
        if payload.header.is_empty() {
            payload.header = std::mem::take(&mut self.legacy_header);
        }
        if payload.footer.is_empty() {
            payload.footer = std::mem::take(&mut self.legacy_footer);
        }
    }

    pub(crate) fn wrap(&self, gcode: String) -> String {
        let fmt = self.format;
        match self.payloads.get(&fmt) {
            Some(p) => p.wrap(fmt, gcode),
            None => fmt.wrap(gcode, "", ""),
        }
    }
}