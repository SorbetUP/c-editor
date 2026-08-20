use crate::distance_to_segment;
use std::cmp::Ordering;

const RESAMPLE_N: usize = 64;
const RECOGNITION_MIN_SCREEN_SIZE: f32 = 25.0;
const CLOSED_GAP_MAX_RATIO: f32 = 0.15;
const LINEAR_MAX_ELONGATION: f32 = 0.25;
const ARROWHEAD_ZONE_RATIO: f32 = 0.5;
const LINEAR_MAX_SHAFT_DEVIATION: f32 = 0.15;
const ARROW_MIN_SKEW: f32 = 0.3;
const CLOSED_SHAPE_MAX_DISTANCE: f32 = 1.5;
const TURN_WINDOW: usize = 3;
const HULL_FILL_RATIO_TOLERANCE: f32 = 0.2;
const CORNER_TURN_SHARE_TOLERANCE: f32 = 0.2;
const KURTOSIS_PRODUCT_TOLERANCE: f32 = 0.7;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecognizedShape {
    Rectangle,
    Diamond,
    Ellipse,
    Arrow,
    Line,
    FreeDraw,
}

impl RecognizedShape {
    pub fn element_type(self) -> &'static str {
        match self {
            Self::Rectangle => "rectangle",
            Self::Diamond => "diamond",
            Self::Ellipse => "ellipse",
            Self::Arrow => "arrow",
            Self::Line => "line",
            Self::FreeDraw => "freedraw",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShapeRecognition {
    pub shape: RecognizedShape,
    pub bounds: [f32; 4],
}

#[derive(Clone, Copy, Debug, Default)]
struct Features {
    gap_ratio: f32,
    elongation: f32,
    major_skew: f32,
    hull_fill_ratio: f32,
    corner_turn_share: f32,
    kurtosis_product: f32,
    shaft_deviation_ratio: f32,
}

pub fn recognize_shape(
    points: &[[f32; 2]],
    zoom: f32,
    previous_was_arrow: bool,
) -> ShapeRecognition {
    let bounds = bounds(points);
    let max_dim = (bounds[2] - bounds[0]).max(bounds[3] - bounds[1]);
    if points.len() < 3 || max_dim * zoom.max(0.0) < RECOGNITION_MIN_SCREEN_SIZE {
        return ShapeRecognition {
            shape: RecognizedShape::FreeDraw,
            bounds,
        };
    }

    let features = extract_features(points);
    let mut shape = if features.gap_ratio > CLOSED_GAP_MAX_RATIO {
        classify_open(features)
    } else {
        classify_closed(features)
    };
    if previous_was_arrow && shape != RecognizedShape::Arrow {
        shape = RecognizedShape::FreeDraw;
    }
    ShapeRecognition { shape, bounds }
}

pub fn recognized_arrow_endpoint(points: &[[f32; 2]], bounds: [f32; 4]) -> Option<[f32; 2]> {
    let start = *points.first()?;
    let [min_x, min_y, max_x, max_y] = bounds;
    if (max_x - min_x).abs() <= f32::EPSILON && (max_y - min_y).abs() <= f32::EPSILON {
        return points.last().copied();
    }
    let candidates = [
        [min_x, min_y],
        [(min_x + max_x) / 2.0, min_y],
        [max_x, min_y],
        [max_x, (min_y + max_y) / 2.0],
        [max_x, max_y],
        [(min_x + max_x) / 2.0, max_y],
        [min_x, max_y],
        [min_x, (min_y + max_y) / 2.0],
    ];
    let ideal = candidates
        .into_iter()
        .max_by(|a, b| distance_sq(start, *a).total_cmp(&distance_sq(start, *b)))?;
    points
        .iter()
        .copied()
        .min_by(|a, b| distance_sq(*a, ideal).total_cmp(&distance_sq(*b, ideal)))
}

fn classify_open(features: Features) -> RecognizedShape {
    if features.elongation > LINEAR_MAX_ELONGATION
        || features.shaft_deviation_ratio > LINEAR_MAX_SHAFT_DEVIATION
    {
        return RecognizedShape::FreeDraw;
    }
    if features.major_skew.abs() >= ARROW_MIN_SKEW {
        RecognizedShape::Arrow
    } else {
        RecognizedShape::Line
    }
}

fn classify_closed(features: Features) -> RecognizedShape {
    let prototypes = [
        (RecognizedShape::Rectangle, 1.0, 0.95, 1.83),
        (RecognizedShape::Diamond, 0.5, 0.95, 3.24),
        (
            RecognizedShape::Ellipse,
            std::f32::consts::PI / 4.0,
            0.55,
            2.25,
        ),
    ];
    let mut best = RecognizedShape::FreeDraw;
    let mut best_distance = CLOSED_SHAPE_MAX_DISTANCE;
    for (shape, hull, corner, kurtosis) in prototypes {
        let distance = (
            ((features.hull_fill_ratio - hull) / HULL_FILL_RATIO_TOLERANCE).powi(2)
                + ((features.corner_turn_share - corner) / CORNER_TURN_SHARE_TOLERANCE).powi(2)
                + ((features.kurtosis_product - kurtosis) / KURTOSIS_PRODUCT_TOLERANCE).powi(2)
        )
        .sqrt();
        if distance < best_distance {
            best_distance = distance;
            best = shape;
        }
    }
    best
}

fn extract_features(points: &[[f32; 2]]) -> Features {
    let sampled = resample(points, RESAMPLE_N);
    let path_length = sampled
        .windows(2)
        .map(|pair| distance(pair[0], pair[1]))
        .sum::<f32>();
    let gap = distance(sampled[0], sampled[sampled.len() - 1]);
    let axes = principal_axes(&sampled);
    let mut major = axes.major;
    let mut projections = project_major(&sampled, axes.centroid, major);
    if skewness(&projections) > 0.0 {
        major = [-major[0], -major[1]];
        projections = project_major(&sampled, axes.centroid, major);
    }
    let hull = convex_hull(&sampled);
    let [min_x, min_y, max_x, max_y] = bounds(&sampled);
    let box_area = (max_x - min_x) * (max_y - min_y);
    let xs = sampled.iter().map(|point| point[0]).collect::<Vec<_>>();
    let ys = sampled.iter().map(|point| point[1]).collect::<Vec<_>>();
    Features {
        gap_ratio: if path_length > 0.0 { gap / path_length } else { 0.0 },
        elongation: if axes.major_variance > 0.0 {
            axes.minor_variance / axes.major_variance
        } else {
            1.0
        },
        major_skew: skewness(&projections),
        hull_fill_ratio: if box_area > 0.0 {
            polygon_area(&hull) / box_area
        } else {
            0.0
        },
        corner_turn_share: corner_turn_share(&sampled),
        kurtosis_product: kurtosis(&xs) * kurtosis(&ys),
        shaft_deviation_ratio: shaft_deviation_ratio(&sampled),
    }
}

#[derive(Clone, Copy, Debug)]
struct PrincipalAxes {
    centroid: [f32; 2],
    major: [f32; 2],
    major_variance: f32,
    minor_variance: f32,
}

fn principal_axes(points: &[[f32; 2]]) -> PrincipalAxes {
    let centroid = centroid(points);
    let mut m20 = 0.0;
    let mut m02 = 0.0;
    let mut m11 = 0.0;
    for [x, y] in points {
        let dx = *x - centroid[0];
        let dy = *y - centroid[1];
        m20 += dx * dx;
        m02 += dy * dy;
        m11 += dx * dy;
    }
    let n = points.len().max(1) as f32;
    m20 /= n;
    m02 /= n;
    m11 /= n;
    let trace = m20 + m02;
    let diff = ((m20 - m02).powi(2) + (2.0 * m11).powi(2)).sqrt();
    let major_variance = (trace + diff) / 2.0;
    let minor_variance = (trace - diff) / 2.0;
    let major = if m11.abs() > f32::EPSILON {
        normalize([major_variance - m02, m11])
    } else if m20 >= m02 {
        [1.0, 0.0]
    } else {
        [0.0, 1.0]
    };
    PrincipalAxes {
        centroid,
        major,
        major_variance,
        minor_variance,
    }
}

fn project_major(points: &[[f32; 2]], centroid: [f32; 2], major: [f32; 2]) -> Vec<f32> {
    points
        .iter()
        .map(|[x, y]| (*x - centroid[0]) * major[0] + (*y - centroid[1]) * major[1])
        .collect()
}

fn standardized_moment(values: &[f32], order: i32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let mean = values.iter().sum::<f32>() / values.len() as f32;
    let variance = values
        .iter()
        .map(|value| (*value - mean).powi(2))
        .sum::<f32>()
        / values.len() as f32;
    let sigma = variance.sqrt();
    if sigma <= f32::EPSILON {
        return 0.0;
    }
    values
        .iter()
        .map(|value| (*value - mean).powi(order))
        .sum::<f32>()
        / values.len() as f32
        / sigma.powi(order)
}

fn skewness(values: &[f32]) -> f32 {
    standardized_moment(values, 3)
}

fn kurtosis(values: &[f32]) -> f32 {
    standardized_moment(values, 4)
}

fn shaft_deviation_ratio(points: &[[f32; 2]]) -> f32 {
    let start = points[0];
    let Some((tip, tip_distance)) = points
        .iter()
        .copied()
        .map(|point| (point, distance(start, point)))
        .max_by(|a, b| a.1.total_cmp(&b.1))
    else {
        return 0.0;
    };
    if tip_distance <= f32::EPSILON {
        return 0.0;
    }
    points
        .iter()
        .copied()
        .filter(|point| distance(*point, tip) > ARROWHEAD_ZONE_RATIO * tip_distance)
        .map(|point| distance_to_segment(point, start, tip))
        .fold(0.0, f32::max)
        / tip_distance
}

fn corner_turn_share(points: &[[f32; 2]]) -> f32 {
    let turns = windowed_turns(points);
    let total = turns.iter().sum::<f32>();
    if total <= f32::EPSILON {
        return 0.0;
    }
    let mut taken = vec![false; turns.len()];
    let mut top = 0.0;
    for _ in 0..4 {
        let Some((peak, _)) = turns
            .iter()
            .enumerate()
            .filter(|(index, _)| !taken[*index])
            .max_by(|a, b| a.1.total_cmp(b.1))
        else {
            break;
        };
        let start = peak.saturating_sub(TURN_WINDOW);
        let end = (peak + TURN_WINDOW).min(turns.len().saturating_sub(1));
        for index in start..=end {
            if !taken[index] {
                top += turns[index];
                taken[index] = true;
            }
        }
    }
    top / total
}

fn windowed_turns(points: &[[f32; 2]]) -> Vec<f32> {
    if points.len() <= TURN_WINDOW * 2 {
        return Vec::new();
    }
    (TURN_WINDOW..points.len() - TURN_WINDOW)
        .map(|index| {
            let a = points[index - TURN_WINDOW];
            let b = points[index];
            let c = points[index + TURN_WINDOW];
            let v1 = [b[0] - a[0], b[1] - a[1]];
            let v2 = [c[0] - b[0], c[1] - b[1]];
            (v1[0] * v2[1] - v1[1] * v2[0])
                .atan2(v1[0] * v2[0] + v1[1] * v2[1])
                .abs()
        })
        .collect()
}

fn resample(points: &[[f32; 2]], count: usize) -> Vec<[f32; 2]> {
    if points.is_empty() || count == 0 {
        return Vec::new();
    }
    if points.len() == 1 || count == 1 {
        return vec![points[0]; count.max(1)];
    }
    let mut cumulative = Vec::with_capacity(points.len());
    cumulative.push(0.0);
    for pair in points.windows(2) {
        cumulative.push(cumulative.last().copied().unwrap_or(0.0) + distance(pair[0], pair[1]));
    }
    let total = *cumulative.last().unwrap_or(&0.0);
    if total <= f32::EPSILON {
        return vec![points[0]; count];
    }
    let mut output = Vec::with_capacity(count);
    let mut segment = 0;
    for sample in 0..count {
        let target = total * sample as f32 / (count - 1) as f32;
        while segment + 1 < cumulative.len() && cumulative[segment + 1] < target {
            segment += 1;
        }
        if segment + 1 >= points.len() {
            output.push(*points.last().unwrap());
            continue;
        }
        let start_distance = cumulative[segment];
        let end_distance = cumulative[segment + 1];
        let denominator = end_distance - start_distance;
        let t = if denominator <= f32::EPSILON {
            0.0
        } else {
            (target - start_distance) / denominator
        };
        output.push([
            points[segment][0] + (points[segment + 1][0] - points[segment][0]) * t,
            points[segment][1] + (points[segment + 1][1] - points[segment][1]) * t,
        ]);
    }
    output
}

fn convex_hull(points: &[[f32; 2]]) -> Vec<[f32; 2]> {
    if points.len() <= 2 {
        return points.to_vec();
    }
    let mut sorted = points.to_vec();
    sorted.sort_by(|a, b| match a[0].total_cmp(&b[0]) {
        Ordering::Equal => a[1].total_cmp(&b[1]),
        other => other,
    });
    sorted.dedup_by(|a, b| a[0] == b[0] && a[1] == b[1]);
    if sorted.len() <= 2 {
        return sorted;
    }
    let mut lower = Vec::new();
    for point in &sorted {
        while lower.len() >= 2
            && cross(lower[lower.len() - 2], lower[lower.len() - 1], *point) <= 0.0
        {
            lower.pop();
        }
        lower.push(*point);
    }
    let mut upper = Vec::new();
    for point in sorted.iter().rev() {
        while upper.len() >= 2
            && cross(upper[upper.len() - 2], upper[upper.len() - 1], *point) <= 0.0
        {
            upper.pop();
        }
        upper.push(*point);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

fn polygon_area(points: &[[f32; 2]]) -> f32 {
    if points.len() < 3 {
        return 0.0;
    }
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .map(|(a, b)| a[0] * b[1] - b[0] * a[1])
        .sum::<f32>()
        .abs()
        / 2.0
}

fn bounds(points: &[[f32; 2]]) -> [f32; 4] {
    if points.is_empty() {
        return [0.0; 4];
    }
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for [x, y] in points {
        min_x = min_x.min(*x);
        min_y = min_y.min(*y);
        max_x = max_x.max(*x);
        max_y = max_y.max(*y);
    }
    [min_x, min_y, max_x, max_y]
}

fn centroid(points: &[[f32; 2]]) -> [f32; 2] {
    let (x, y) = points
        .iter()
        .fold((0.0, 0.0), |(sx, sy), point| (sx + point[0], sy + point[1]));
    let n = points.len().max(1) as f32;
    [x / n, y / n]
}

fn normalize(vector: [f32; 2]) -> [f32; 2] {
    let length = (vector[0] * vector[0] + vector[1] * vector[1]).sqrt();
    if length <= f32::EPSILON {
        [1.0, 0.0]
    } else {
        [vector[0] / length, vector[1] / length]
    }
}

fn cross(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn distance(a: [f32; 2], b: [f32; 2]) -> f32 {
    distance_sq(a, b).sqrt()
}

fn distance_sq(a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)
}
