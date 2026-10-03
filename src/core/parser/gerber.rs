use std::{fs::File, io::BufReader, path::Path};

use evalexpr::{Context, ContextWithMutableVariables, HashMapContext, eval_float_with_context};
use rust_i18n::t;

use gerber_parser::{
    GerberDoc,
    gerber_types::{
        Aperture, Command, CoordinateOffset, Coordinates, DCode, ExtendedCode, FunctionCode, GCode,
        InterpolationMode, MCode, MacroDecimal, Operation, Unit,
    },
};

use crate::core::{
    LayerUnit, PcbLayer, PcbTrace, Primitive,
    geometry::{Arc, Circle, Point2d, Rectangle, Segment},
    parser::ParseError,
};

pub fn load_gerber(path: &Path) -> Result<PcbLayer, ParseError> {
    log::info!(
        "{}",
        t!("gerber.info.load_file", path = path.to_string_lossy())
    );
    let file = File::open(path).map_err(ParseError::Io)?;
    let reader = BufReader::new(file);

    let gerber_doc =
        gerber_parser::parse(reader).map_err(|(_, err)| ParseError::Gerber(err.to_string()))?;

    let mut layer = PcbLayer::default();

    log::info!("{}", t!("gerber.info.generate_layer"));
    generate_layer(&gerber_doc, &mut layer)?;

    log::info!("{}", t!("gerber.info.loaded"));
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
                                            // return Err(ParseError::Gerber("Offset position is mandatory with (Counter) Clockwise interpolation mode".to_string()));
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
                                        return Err(ParseError::Gerber(
                                            t!("gerber.error.offset_circule_mode").into(),
                                        ));
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
                        log::warn!(
                            "\t{}",
                            t!("gerber.warn.G_unsupported", function = "Region Mode")
                        )
                    }

                    GCode::QuadrantMode(_quadrant_mode) => {
                        log::warn!(
                            "\t{}",
                            t!("gerber.warn.G_unsupported", function = "Quadrant Mode")
                        )
                    }

                    GCode::Comment(_) => (),

                    GCode::Unit(_) => {
                        log::warn!("\t{}", t!("gerber.warn.deprecated", function = "Unit"))
                    }

                    GCode::CoordinateMode(_) => {
                        log::warn!(
                            "\t{}",
                            t!("gerber.warn.deprecated", function = "Coordinate Mode")
                        )
                    }

                    GCode::SelectAperture => {
                        log::warn!(
                            "\t{}",
                            t!("gerber.warn.deprecated", function = "Select Aperture")
                        )
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
        }
        ExtendedCode::Unit(unit) => {
            layer.unit = match unit {
                Unit::Inches => LayerUnit::Inch,
                Unit::Millimeters => LayerUnit::Millimeter,
            };
        }

        ExtendedCode::ApertureDefinition(_) => {
            // Nothing to do...
        }

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

fn aperture_to_primitive(
    aperture: &Aperture,
    position: &Point2d,
    layer: &PcbLayer,
) -> Option<Primitive> {
    match aperture {
        Aperture::Circle(circle) => Some(Primitive::Circle(Circle {
            center: position.clone(),
            diameter: circle.diameter,
        })),

        Aperture::Rectangle(rectangular) => Some(Primitive::Rectangle(Rectangle {
            center: position.clone(),
            width: rectangular.x,
            height: rectangular.y,
            rotation: 0.0,
        })),

        Aperture::Obround(_rectangular) => {
            log::warn!(
                "{}",
                t!("gerber.warn.aperture_unsupported", function = "Obround")
            );
            None
        }

        Aperture::Polygon(_polygon) => {
            log::warn!(
                "{}",
                t!("gerber.warn.aperture_unsupported", function = "Polygon")
            );
            None
        }

        Aperture::Macro(name, data) => {
            if let Some(aperture) = layer.aperture_macro.get(name) {
                let mut complex: Vec<Primitive> = Vec::new();
                let context = macro_data_to_context(data);

                for element in aperture {
                    match element {
                        gerber_parser::gerber_types::MacroContent::Circle(circle_primitive) => {
                            complex.push(Primitive::Circle(Circle {
                                center: macro_decimal_pair_to_point(
                                    &circle_primitive.center,
                                    &context,
                                )
                                .add(position),
                                diameter: resolve_macro_decimal(
                                    &circle_primitive.diameter,
                                    &context,
                                ),
                            }));
                        }

                        gerber_parser::gerber_types::MacroContent::VectorLine(vector_line) => {
                            let start = macro_decimal_pair_to_point(&vector_line.start, &context);
                            let end = macro_decimal_pair_to_point(&vector_line.end, &context);
                            let (dx, dy) = (end.x - start.x, end.y - start.y);
                            let len = dx.hypot(dy);

                            if resolve_macro_decimal(&vector_line.angle, &context) != 0.0 {
                                log::warn!("Angle is not supported for VectorLine");
                            }

                            if len > 0.0 {
                                let h = resolve_macro_decimal(&vector_line.width, &context) / 2.0;
                                let (nx, ny) = (-dy / len * h, dx / len * h);

                                complex.push(Primitive::Polygon(vec![
                                    Point2d::new(start.x + nx, start.y + ny).add(position),
                                    Point2d::new(start.x - nx, start.y - ny).add(position),
                                    Point2d::new(end.x - nx, end.y - ny).add(position),
                                    Point2d::new(end.x + nx, end.y + ny).add(position),
                                ]));
                            }
                        }

                        gerber_parser::gerber_types::MacroContent::CenterLine(center_line) => {
                            let center = macro_decimal_pair_to_point(&center_line.center, &context);
                            let size =
                                macro_decimal_pair_to_point(&center_line.dimensions, &context);

                            complex.push(Primitive::Rectangle(Rectangle {
                                center: center.add(position),
                                width: size.x,
                                height: size.y,
                            }));
                        }

                        gerber_parser::gerber_types::MacroContent::Outline(outline_primitive) => {
                            let mut points = Vec::new();
                            for p in &outline_primitive.points {
                                points.push(macro_decimal_pair_to_point(p, &context).add(position));
                            }

                            complex.push(Primitive::Polygon(points));

                            if resolve_macro_decimal(&outline_primitive.angle, &context) != 0.0 {
                                log::warn!("Angle is not supported for Outline");
                            }
                        }

                        gerber_parser::gerber_types::MacroContent::Polygon(polygon_primitive) => {
                            log::warn!("TODO ! Support polygon")
                        }

                        gerber_parser::gerber_types::MacroContent::Moire(moire_primitive) => {
                            log::warn!("TODO ! Support Moire")
                        }

                        gerber_parser::gerber_types::MacroContent::Thermal(thermal_primitive) => {
                            log::warn!("TODO ! Support Thermal")
                        }

                        gerber_parser::gerber_types::MacroContent::VariableDefinition(_)
                        | gerber_parser::gerber_types::MacroContent::Comment(_) => {}
                    }
                }

                Some(Primitive::Complex(complex))
            } else {
                log::error!("{}", t!("gerber.error.no_macro_aperture_name", name = name));
                None
            }
        }
    }
}

fn width_from_aperture(aperture: Option<&Aperture>) -> f64 {
    if let Some(aperture) = aperture {
        match aperture {
            Aperture::Circle(circle) => circle.diameter,

            Aperture::Rectangle(rectangular) => rectangular.x.max(rectangular.y),

            Aperture::Obround(_rectangular) => {
                log::warn!(
                    "{}",
                    t!("gerber.warn.aperture_unsupported", function = "Obround")
                );
                0.0
            }

            Aperture::Polygon(_polygon) => {
                log::warn!(
                    "{}",
                    t!("gerber.warn.aperture_unsupported", function = "Polygon")
                );
                0.0
            }

            Aperture::Macro(_, _macro_decimals) => {
                log::warn!(
                    "{}",
                    t!("gerber.warn.aperture_unsupported", function = "Macro")
                );
                0.0
            }
        }
    } else {
        0.0
    }
}

fn macro_data_to_context(data: &Option<Vec<MacroDecimal>>) -> HashMapContext {
    let mut result = HashMapContext::new();

    if let Some(data) = data {
        for (idx, elem) in data.iter().enumerate() {
            match elem {
                MacroDecimal::Value(v) => {
                    let _ = result.set_value(format!("${}", idx + 1), evalexpr::Value::Float(*v));
                }
                MacroDecimal::Variable(_) | MacroDecimal::Expression(_) => {
                    log::error!("{}", t!("gerber.warn.macro_aperture_parameter_not_number"));
                }
            };
        }
    }

    result
}

fn macro_decimal_pair_to_point(
    pair: &(MacroDecimal, MacroDecimal),
    macro_data: &HashMapContext,
) -> Point2d {
    Point2d {
        x: resolve_macro_decimal(&pair.0, macro_data),
        y: resolve_macro_decimal(&pair.1, macro_data),
    }
}

fn resolve_macro_decimal(value: &MacroDecimal, macro_data: &HashMapContext) -> f64 {
    match value {
        MacroDecimal::Value(v) => *v,
        MacroDecimal::Variable(var) => {
            if let Some(value) = macro_data.get_value(&format!("${}", var)) {
                if let Ok(value) = value.as_number() {
                    value
                } else {
                    log::error!(
                        "{}",
                        t!("gerber.error.macro_variable_not_number", var = var)
                    );
                    f64::NAN
                }
            } else {
                log::error!("{}", t!("gerber.error.macro_variable_not_found", var = var));
                f64::NAN
            }
        }
        MacroDecimal::Expression(expr) => match eval_float_with_context(&expr, macro_data) {
            Ok(val) => val,
            Err(e) => {
                log::error!(
                    "{}",
                    t!(
                        "gerber.error.macro_expression_error",
                        expr = expr,
                        err = e.to_string()
                    )
                );
                f64::NAN
            }
        },
    }
}
