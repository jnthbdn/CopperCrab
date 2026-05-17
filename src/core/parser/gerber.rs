use std::{fs::File, io::BufReader, path::Path};

use gerber_parser::{
    GerberDoc,
    gerber_types::{
        Aperture, Command, CoordinateOffset, Coordinates, DCode, ExtendedCode, FunctionCode, GCode,
        InterpolationMode, MCode, Operation, Unit,
    },
};

use crate::core::{
    LayerUnit, PcbLayer, PcbTrace, Primitive,
    geometry::{Arc, Circle, Point2d, Rectangle, Segment},
    parser::ParseError,
};

pub fn load_gerber(path: &Path) -> Result<PcbLayer, ParseError> {
    log::info!("Loading file '{}'", path.to_string_lossy());
    let file = File::open(path).map_err(ParseError::Io)?;
    let reader = BufReader::new(file);

    log::info!("Parse file");
    let gerber_doc =
        gerber_parser::parse(reader).map_err(|(_, err)| ParseError::Gerber(err.to_string()))?;

    let mut layer = PcbLayer::default();

    log::info!("Generating layer");
    generate_layer(&gerber_doc, &mut layer)?;

    log::info!("Gerber file loaded");
    Ok(layer)
}

fn generate_layer(gerber_doc: &GerberDoc, layer: &mut PcbLayer) -> Result<(), ParseError> {
    let mut position: Point2d = Point2d::new(0.0, 0.0);
    let mut aperture: i32 = 0;
    let mut primitives: Vec<Primitive> = Vec::new();
    let mut interpolation = InterpolationMode::Linear;

    for command in gerber_doc.commands() {
        match command {
            Command::FunctionCode(function_code) => match function_code {
                FunctionCode::DCode(dcode) => match dcode {
                    DCode::Operation(operation) => match operation {
                        Operation::Interpolate(coordinates, coordinate_offset) => {
                            let target_pos = coordinate_to_position(coordinates, &position);
                            let offset =
                                coordinate_offset_to_position(coordinate_offset, &position);

                            match interpolation {
                                InterpolationMode::Linear => {
                                    primitives.push(Primitive::Segment(Segment {
                                        start: position.clone(),
                                        end: target_pos.clone(),
                                        width: width_from_aperture(
                                            gerber_doc.apertures.get(&aperture),
                                        ),
                                    }));
                                }

                                InterpolationMode::ClockwiseCircular
                                | InterpolationMode::CounterclockwiseCircular => {
                                    if let Some(coord) = coordinate_offset {
                                        if coord.x.is_none() || coord.y.is_none() {
                                            return Err(ParseError::Gerber("Offset position is mandatory with (Counter) Clockwise interpolation mode".to_string()));
                                        } else {
                                            primitives.push(Primitive::Arc(Arc {
                                                start: position.clone(),
                                                end: target_pos.clone(),
                                                center: offset,
                                                clockwise: interpolation
                                                    == InterpolationMode::ClockwiseCircular,
                                                width: width_from_aperture(
                                                    gerber_doc.apertures.get(&aperture),
                                                ),
                                            }));
                                        }
                                    } else {
                                        return Err(ParseError::Gerber("Offset position is mandatory with (Counter) Clockwise interpolation mode".to_string()));
                                    }
                                }
                            };

                            position = target_pos;
                        }

                        Operation::Move(coordinates) => {
                            position = coordinate_to_position(&coordinates, &position);

                            if false == primitives.is_empty() {
                                layer.traces.push(PcbTrace { primitives });
                                primitives = Vec::new();
                            }
                        }

                        Operation::Flash(coordinates) => {
                            let flash_pos = coordinate_to_position(coordinates, &position);
                            if let Some(aperture) = gerber_doc.apertures.get(&aperture) {
                                if let Some(p) = aperture_to_primitive(aperture, &flash_pos) {
                                    primitives.push(p);
                                }
                            }
                        }
                    },

                    DCode::SelectAperture(id) => aperture = *id,
                },

                FunctionCode::GCode(gcode) => match gcode {
                    GCode::InterpolationMode(interpolation_mode) => {
                        interpolation = *interpolation_mode;
                    }

                    GCode::RegionMode(_) => {
                        log::warn!("\tUnsupported 'Region Mode'")
                    }

                    GCode::QuadrantMode(_quadrant_mode) => {
                        log::warn!("\tUnsupported 'QuadrantMode'")
                    }

                    GCode::Comment(_) => (),

                    GCode::Unit(_) => {
                        log::warn!("\tGCode 'Unit' is deprecated since December 2012.")
                    }

                    GCode::CoordinateMode(_) => {
                        log::warn!("\tGCode 'CoordinateMode' is deprecated since December 2012.")
                    }

                    GCode::SelectAperture => {
                        log::warn!("\tGCode 'SelectAperture' is deprecated since December 2012.")
                    }
                },

                FunctionCode::MCode(mcode) => match mcode {
                    MCode::EndOfFile => (),
                },
            },
            Command::ExtendedCode(extended_code) => parse_extended_command(extended_code, layer),
        }
    }

    layer.traces.push(PcbTrace { primitives });

    Ok(())
}

fn parse_extended_command(cmd: &ExtendedCode, layer: &mut PcbLayer) {
    match cmd {
        ExtendedCode::CoordinateFormat(_) => {
            // Nothing to do...
            // log::warn!("\tUnsupported extended command 'CoordinateFormat'")
        }
        ExtendedCode::Unit(unit) => {
            layer.unit = match unit {
                Unit::Inches => LayerUnit::Inch,
                Unit::Millimeters => LayerUnit::Millimeter,
            };
        }

        ExtendedCode::ApertureDefinition(_) => {
            // Nothing to do...
            // log::warn!("\tUnsupported extended command 'ApertureDefinition'")
        }

        ExtendedCode::ApertureMacro(_) => {
            log::warn!("\tUnsupported extended command 'ApertureMacro'")
        }

        ExtendedCode::LoadPolarity(_) => {
            log::warn!("\tUnsupported extended command 'LoadPolarity'")
        }

        ExtendedCode::LoadMirroring(_) => {
            log::warn!("\tUnsupported extended command 'LoadMirroring'")
        }

        ExtendedCode::LoadRotation(_) => {
            log::warn!("\tUnsupported extended command 'LoadRotation'")
        }

        ExtendedCode::LoadScaling(_) => {
            log::warn!("\tUnsupported extended command 'LoadScaling'")
        }

        ExtendedCode::StepAndRepeat(_) => {
            log::warn!("\tUnsupported extended command 'StepAndRepeat'")
        }

        ExtendedCode::ApertureBlock(_) => {
            log::warn!("\tUnsupported extended command 'ApertureBlock'")
        }

        ExtendedCode::FileAttribute(_) => {
            log::warn!("\tUnsupported extended command 'FileAttribute'")
        }

        ExtendedCode::ObjectAttribute(_) => {
            log::warn!("\tUnsupported extended command 'ObjectAttribute'")
        }

        ExtendedCode::ApertureAttribute(_) => {
            log::warn!("\tUnsupported extended command 'ApertureAttribute'")
        }

        ExtendedCode::DeleteAttribute(_) => {
            log::warn!("\tUnsupported extended command 'DeleteAttribute'")
        }

        ExtendedCode::MirrorImage(_) => {
            log::warn!("\tUnsupported extended command 'MirrorImage'")
        }

        ExtendedCode::OffsetImage(_) => {
            log::warn!("\tUnsupported extended command 'OffsetImage'")
        }

        ExtendedCode::ScaleImage(_) => {
            log::warn!("\tUnsupported extended command 'ScaleImage'")
        }

        ExtendedCode::RotateImage(_) => {
            log::warn!("\tUnsupported extended command 'RotateImage'")
        }

        ExtendedCode::ImagePolarity(_) => {
            log::warn!("\tUnsupported extended command 'ImagePolarity'")
        }

        ExtendedCode::AxisSelect(_) => {
            log::warn!("\tUnsupported extended command 'AxisSelect'")
        }

        ExtendedCode::ImageName(_) => {
            log::warn!("\tUnsupported extended command 'ImageName'")
        }
    }
}

fn coordinate_to_position(coord: &Option<Coordinates>, current_position: &Point2d) -> Point2d {
    let mut result = current_position.clone();

    if let Some(c) = coord {
        if let Some(x) = c.x {
            result.x = x.into();
        }

        if let Some(y) = c.y {
            result.y = y.into();
        }
    }

    result
}

fn coordinate_offset_to_position(
    coord: &Option<CoordinateOffset>,
    current_position: &Point2d,
) -> Point2d {
    let mut result = current_position.clone();

    if let Some(c) = coord {
        if let Some(x) = c.x {
            result.x = x.into();
        }

        if let Some(y) = c.y {
            result.y = y.into();
        }
    }

    result
}

fn aperture_to_primitive(aperture: &Aperture, position: &Point2d) -> Option<Primitive> {
    match aperture {
        Aperture::Circle(circle) => Some(Primitive::Circle(Circle {
            center: position.clone(),
            diameter: circle.diameter,
        })),

        Aperture::Rectangle(rectangular) => Some(Primitive::Rectangle(Rectangle {
            center: position.clone(),
            width: rectangular.x,
            height: rectangular.y,
        })),

        Aperture::Obround(_rectangular) => {
            log::warn!("Unsupported 'Obround' aperture");
            None
        }

        Aperture::Polygon(_polygon) => {
            log::warn!("Unsupported 'Polygon' aperture");
            None
        }

        Aperture::Macro(_, _macro_decimals) => {
            log::warn!("Unsupported 'Macro' aperture");
            None
        }
    }
}

fn width_from_aperture(aperture: Option<&Aperture>) -> f64 {
    if let Some(aperture) = aperture {
        match aperture {
            Aperture::Circle(circle) => circle.diameter,

            Aperture::Rectangle(rectangular) => rectangular.x.max(rectangular.y),

            Aperture::Obround(_rectangular) => {
                log::warn!("Unsupported 'Obround' aperture");
                0.0
            }

            Aperture::Polygon(_polygon) => {
                log::warn!("Unsupported 'Polygon' aperture");
                0.0
            }

            Aperture::Macro(_, _macro_decimals) => {
                log::warn!("Unsupported 'Macro' aperture");
                0.0
            }
        }
    } else {
        0.0
    }
}
