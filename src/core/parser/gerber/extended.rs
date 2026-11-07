use gerber_parser::gerber_types::{ExtendedCode, Unit};
use rust_i18n::t;

use crate::core::{LayerUnit, PcbLayer};

pub(super) fn parse_extended_command(cmd: &ExtendedCode, layer: &mut PcbLayer) {
    match cmd {
        ExtendedCode::CoordinateFormat(_) => {}

        ExtendedCode::Unit(unit) => {
            layer.unit = match unit {
                Unit::Inches => LayerUnit::Inch,
                Unit::Millimeters => LayerUnit::Millimeter,
            };
        }

        ExtendedCode::ApertureDefinition(_) => {}

        ExtendedCode::ApertureMacro(aperture) => {
            layer
                .aperture_macro
                .insert(aperture.name.clone(), aperture.content.clone());
        }

        ExtendedCode::LoadPolarity(_) => {
            log::warn!(
                "\t{}",
                t!(
                    "gerber.warn.extended_unsupported",
                    function = "LoadPolarity"
                )
            )
        }

        ExtendedCode::LoadMirroring(_) => {
            log::warn!(
                "\t{}",
                t!(
                    "gerber.warn.extended_unsupported",
                    function = "LoadMirroring"
                )
            )
        }

        ExtendedCode::LoadRotation(_) => {
            log::warn!(
                "\t{}",
                t!(
                    "gerber.warn.extended_unsupported",
                    function = "LoadRotation"
                )
            )
        }

        ExtendedCode::LoadScaling(_) => {
            log::warn!(
                "\t{}",
                t!("gerber.warn.extended_unsupported", function = "LoadScaling")
            )
        }

        ExtendedCode::StepAndRepeat(_) => {
            log::warn!(
                "\t{}",
                t!(
                    "gerber.warn.extended_unsupported",
                    function = "StepAndRepeat"
                )
            )
        }

        ExtendedCode::ApertureBlock(_) => {
            log::warn!(
                "\t{}",
                t!(
                    "gerber.warn.extended_unsupported",
                    function = "ApertureBlock"
                )
            )
        }

        ExtendedCode::FileAttribute(_) => {
            log::warn!(
                "\t{}",
                t!(
                    "gerber.warn.extended_unsupported",
                    function = "FileAttribute"
                )
            )
        }

        ExtendedCode::ObjectAttribute(_) => {
            log::warn!(
                "\t{}",
                t!(
                    "gerber.warn.extended_unsupported",
                    function = "ObjectAttribute"
                )
            )
        }

        ExtendedCode::ApertureAttribute(_) => {
            log::warn!(
                "\t{}",
                t!(
                    "gerber.warn.extended_unsupported",
                    function = "ApertureAttribute"
                )
            )
        }

        ExtendedCode::DeleteAttribute(_) => {
            log::warn!(
                "\t{}",
                t!(
                    "gerber.warn.extended_unsupported",
                    function = "DeleteAttribute"
                )
            )
        }

        ExtendedCode::MirrorImage(_) => {
            log::warn!(
                "\t{}",
                t!("gerber.warn.extended_unsupported", function = "MirrorImage")
            )
        }

        ExtendedCode::OffsetImage(_) => {
            log::warn!(
                "\t{}",
                t!("gerber.warn.extended_unsupported", function = "OffsetImage")
            )
        }

        ExtendedCode::ScaleImage(_) => {
            log::warn!(
                "\t{}",
                t!("gerber.warn.extended_unsupported", function = "ScaleImage")
            )
        }

        ExtendedCode::RotateImage(_) => {
            log::warn!(
                "\t{}",
                t!("gerber.warn.extended_unsupported", function = "RotateImage")
            )
        }

        ExtendedCode::ImagePolarity(_) => {
            log::warn!(
                "\t{}",
                t!(
                    "gerber.warn.extended_unsupported",
                    function = "ImagePolarity"
                )
            )
        }

        ExtendedCode::AxisSelect(_) => {
            log::warn!(
                "\t{}",
                t!("gerber.warn.extended_unsupported", function = "AxisSelect")
            )
        }

        ExtendedCode::ImageName(_) => {
            log::warn!(
                "\t{}",
                t!("gerber.warn.extended_unsupported", function = "ImageName")
            )
        }
    }
}
