use gerber_parser::gerber_types::{CoordinateOffset, Coordinates};

use crate::core::geometry::Point2d;

pub(super) fn coordinate_to_position(
    coord: &Option<Coordinates>,
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

pub(super) fn coordinate_offset_to_position(
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