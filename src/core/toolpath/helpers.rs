use std::f64::consts::PI;

use clipper2::{EndType, JoinType, Path, Paths};

use crate::core::geometry::{Arc, Circle, Point2d, Rectangle, Segment};

pub fn segment_to_path(segment: &Segment) -> Paths {
    let path: Path = vec![segment.start.to_tuple(), segment.end.to_tuple()].into();

    path.inflate(segment.width / 2.0, JoinType::Round, EndType::Round, 2.0)
        .simplify(0.01, false)
}

pub fn circle_to_path(circle: &Circle, segments: usize) -> Paths {
    let mut points = vec![];

    for i in 0..segments {
        let angle = (i as f64 / segments as f64) * 2.0 * PI;

        points.push((
            angle.cos() * circle.diameter / 2.0 + circle.center.x,
            angle.sin() * circle.diameter / 2.0 + circle.center.y,
        ));
    }

    points.into()
}

pub fn arc_to_path(arc: &Arc, segments: usize) -> Paths {
    let mut points = vec![];

    let start_angle = point_to_angle(&arc.start, &arc.center);
    let end_angle = point_to_angle(&arc.end, &arc.center);

    let begin_angle;
    let mut delta_angle;

    if arc.clockwise {
        delta_angle = end_angle - start_angle;
        begin_angle = start_angle;
    } else {
        delta_angle = start_angle - end_angle;
        begin_angle = end_angle;
    };

    if delta_angle < 0.0 {
        delta_angle += 2.0 * PI;
    }

    let step_segment = delta_angle / segments as f64;
    let radius = radius(&arc.center, &arc.start);

    for i in 0..=segments {
        let angle = i as f64 * step_segment + begin_angle;

        points.push((
            angle.cos() * radius + arc.center.x,
            angle.sin() * radius + arc.center.y,
        ));
    }

    points.into()
}

fn point_to_angle(point: &Point2d, center: &Point2d) -> f64 {
    let angle = (point.y - center.y).atan2(point.x - center.x);
    if angle < 0.0 {
        angle + 2.0 * std::f64::consts::PI
    } else {
        angle
    }
}

fn radius(center: &Point2d, point: &Point2d) -> f64 {
    let dx = point.x - center.x;
    let dy = point.y - center.y;
    (dx * dx + dy * dy).sqrt()
}

pub fn rectangle_to_path(rectangle: &Rectangle) -> Paths {
    vec![
        (
            rectangle.center.x - rectangle.width / 2.0,
            rectangle.center.y - rectangle.height / 2.0,
        ),
        (
            rectangle.center.x + rectangle.width / 2.0,
            rectangle.center.y - rectangle.height / 2.0,
        ),
        (
            rectangle.center.x + rectangle.width / 2.0,
            rectangle.center.y + rectangle.height / 2.0,
        ),
        (
            rectangle.center.x - rectangle.width / 2.0,
            rectangle.center.y + rectangle.height / 2.0,
        ),
        (
            rectangle.center.x - rectangle.width / 2.0,
            rectangle.center.y - rectangle.height / 2.0,
        ),
    ]
    .into()
}
