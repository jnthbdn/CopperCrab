use gerber_parser::gerber_types::{Aperture, MacroContent};
use rust_i18n::t;

use crate::core::{
    PcbLayer,
    geometry::{Circle, Point2d, Rectangle, Segment},
    primitive::Primitive,
};

use super::macros::{macro_data_to_context, macro_decimal_pair_to_point, resolve_macro_decimal};

pub(super) fn aperture_to_primitive(
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

        Aperture::Obround(rect) => {
            let (x, y) = (rect.x, rect.y);

            if (x - y).abs() < f64::EPSILON {
                Some(Primitive::Circle(Circle {
                    center: position.clone(),
                    diameter: x,
                }))
            } else if x > y {
                let half = (x - y) / 2.0;
                Some(Primitive::Segment(Segment {
                    start: Point2d::new(position.x - half, position.y),
                    end: Point2d::new(position.x + half, position.y),
                    width: y,
                }))
            } else {
                let half = (y - x) / 2.0;
                Some(Primitive::Segment(Segment {
                    start: Point2d::new(position.x, position.y - half),
                    end: Point2d::new(position.x, position.y + half),
                    width: x,
                }))
            }
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
                        MacroContent::Circle(circle_primitive) => {
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

                        MacroContent::VectorLine(vector_line) => {
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

                        MacroContent::CenterLine(center_line) => {
                            let center = macro_decimal_pair_to_point(&center_line.center, &context);
                            let size =
                                macro_decimal_pair_to_point(&center_line.dimensions, &context);

                            complex.push(Primitive::Rectangle(Rectangle {
                                center: center.add(position),
                                width: size.x,
                                height: size.y,
                                rotation: resolve_macro_decimal(&center_line.angle, &context),
                            }));
                        }

                        MacroContent::Outline(outline_primitive) => {
                            let mut points = Vec::new();
                            for p in &outline_primitive.points {
                                points.push(macro_decimal_pair_to_point(p, &context).add(position));
                            }

                            complex.push(Primitive::Polygon(points));

                            if resolve_macro_decimal(&outline_primitive.angle, &context) != 0.0 {
                                log::warn!("Angle is not supported for Outline");
                            }
                        }

                        MacroContent::Polygon(_polygon_primitive) => {
                            log::warn!("TODO ! Support polygon")
                        }

                        MacroContent::Moire(_moire_primitive) => {
                            log::warn!("TODO ! Support Moire")
                        }

                        MacroContent::Thermal(_thermal_primitive) => {
                            log::warn!("TODO ! Support Thermal")
                        }

                        MacroContent::VariableDefinition(_) | MacroContent::Comment(_) => {}
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

pub(super) fn width_from_aperture(aperture: Option<&Aperture>) -> f64 {
    if let Some(aperture) = aperture {
        match aperture {
            Aperture::Circle(circle) => circle.diameter,

            Aperture::Rectangle(rectangular) => rectangular.x.max(rectangular.y),

            Aperture::Obround(rectangular) => rectangular.x.max(rectangular.y),

            Aperture::Polygon(polygon) => polygon.diameter,

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
