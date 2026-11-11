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
    pub(crate) const ALL: &'static [ExportFormat] = &[ExportFormat::GCode, ExportFormat::Mach3];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub(crate) enum ExportImageFormat {
    #[default]
    Svg,
    Png,
}

impl ExportImageFormat {
    pub(crate) const ALL: &'static [ExportImageFormat] =
        &[ExportImageFormat::Svg, ExportImageFormat::Png];

    pub(crate) fn label(&self) -> &'static str {
        match self {
            ExportImageFormat::Svg => "SVG",
            ExportImageFormat::Png => "PNG",
        }
    }

    pub(crate) fn extension(&self) -> &'static str {
        match self {
            ExportImageFormat::Svg => "svg",
            ExportImageFormat::Png => "png",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct ColorExportSettings {
    #[serde(default = "default_bw")]
    pub bw: bool,
    #[serde(default)]
    pub invert: bool,
    #[serde(default)]
    pub transparent_bg: bool,
    #[serde(default = "default_background_rgb")]
    pub background: [u8; 3],
    #[serde(default = "default_tracks_rgb")]
    pub tracks: [u8; 3],
}

impl Default for ColorExportSettings {
    fn default() -> Self {
        Self {
            bw: default_bw(),
            invert: false,
            transparent_bg: false,
            background: default_background_rgb(),
            tracks: default_tracks_rgb(),
        }
    }
}

fn default_bw() -> bool {
    true
}

fn default_background_rgb() -> [u8; 3] {
    [0x0d, 0x1a, 0x0d]
}

fn default_tracks_rgb() -> [u8; 3] {
    [0x5c, 0x39, 0x19]
}

impl ColorExportSettings {
    pub(crate) fn resolved(&self) -> (Option<[u8; 3]>, [u8; 3]) {
        if self.bw {
            let (bg, fg) = if self.invert {
                ([0u8, 0, 0], [255u8, 255, 255])
            } else {
                ([255u8, 255, 255], [0u8, 0, 0])
            };
            let bg = if self.transparent_bg { None } else { Some(bg) };
            (bg, fg)
        } else {
            let bg = if self.transparent_bg {
                None
            } else {
                Some(self.background)
            };
            (bg, self.tracks)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SvgExportSettings {
    #[serde(default = "default_svg_margin")]
    pub margin_mm: f64,
}

impl Default for SvgExportSettings {
    fn default() -> Self {
        Self {
            margin_mm: default_svg_margin(),
        }
    }
}

fn default_svg_margin() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PngExportSettings {
    #[serde(default = "default_png_pixels_per_mm")]
    pub pixels_per_mm: f64,
    #[serde(default = "default_png_scale_factor")]
    pub scale_factor: f64,
}

impl Default for PngExportSettings {
    fn default() -> Self {
        Self {
            pixels_per_mm: default_png_pixels_per_mm(),
            scale_factor: default_png_scale_factor(),
        }
    }
}

fn default_png_pixels_per_mm() -> f64 {
    10.0
}

fn default_png_scale_factor() -> f64 {
    1.0
}

pub(crate) const PNG_SCALE_FACTORS: &[f64] = &[1.0, 2.0, 4.0, 8.0];

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

    #[serde(default)]
    pub image_format: ExportImageFormat,

    #[serde(default)]
    pub svg: SvgExportSettings,

    #[serde(default)]
    pub png: PngExportSettings,

    #[serde(default)]
    pub color: ColorExportSettings,
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
