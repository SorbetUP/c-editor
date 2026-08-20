use crate::{
    compute_bucket_fill, create_element, BucketFillFailureReason, BucketFillOptions,
    BucketFillPlacement, DrawingCurrentStyle, DrawingElement, DrawingScene, DrawingTool,
};
use serde_json::{json, Value};

pub const DEFAULT_BUCKET_FILL_BACKGROUND: &str = "#b2f2bb";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BucketFillMutation {
    Inserted { element_id: String },
    Restyled { element_id: String },
    Unchanged { element_id: String },
}

pub fn apply_bucket_fill(
    scene: &mut DrawingScene,
    point: [f32; 2],
    id: impl Into<String>,
    style: &DrawingCurrentStyle,
    options: BucketFillOptions,
) -> Result<BucketFillMutation, BucketFillFailureReason> {
    let hit_id = topmost_hit(scene, point).map(|element| element.id.clone());
    let computed = compute_bucket_fill(scene, point, options);

    let result = match computed {
        Ok(result) => result,
        Err(reason) => {
            if let Some(hit_id) = hit_id.as_deref() {
                if scene
                    .element_by_id(hit_id)
                    .is_some_and(is_bucket_fill_compatible)
                {
                    return Ok(restyle(scene, hit_id, style));
                }
            }
            return Err(reason);
        }
    };

    if let Some(hit_id) = hit_id.as_deref() {
        if scene
            .element_by_id(hit_id)
            .is_some_and(|element| is_restylable_fill(element, &result.scene_points))
        {
            return Ok(restyle(scene, hit_id, style));
        }
    }

    let id = id.into();
    let first = *result
        .scene_points
        .first()
        .ok_or(BucketFillFailureReason::InvalidPolygon)?;
    let mut fill = create_element(DrawingTool::Line, first, id.clone());
    fill.points = result
        .scene_points
        .iter()
        .map(|point| [point[0] - first[0], point[1] - first[1]])
        .collect();
    let (width, height) = point_size(&fill.points);
    fill.width = width;
    fill.height = height;
    fill.stroke_color = "transparent".to_owned();
    fill.background_color = bucket_background(&style.background_color).to_owned();
    fill.stroke_width = 1.0;
    fill.stroke_style = "solid".to_owned();
    fill.fill_style = style.fill_style.clone();
    fill.opacity = style.opacity.clamp(0.0, 100.0);
    fill.start_arrowhead = None;
    fill.end_arrowhead = None;
    fill.extra.insert("polygon".to_owned(), json!(true));
    fill.extra.insert("roughness".to_owned(), json!(0));
    fill.extra.insert("roundness".to_owned(), Value::Null);

    let owner = result
        .owner_id
        .as_deref()
        .and_then(|owner_id| scene.element_by_id(owner_id));
    let frame_id = owner.map_or_else(
        || top_frame_at(scene, point).map(|frame| frame.id.clone()),
        |owner| {
            if matches!(owner.kind.as_str(), "frame" | "magicframe") {
                Some(owner.id.clone())
            } else {
                owner
                    .extra
                    .get("frameId")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            }
        },
    );
    fill.extra.insert(
        "frameId".to_owned(),
        frame_id.map(Value::String).unwrap_or(Value::Null),
    );
    let group_ids = owner
        .map(group_ids_of)
        .unwrap_or_else(|| common_boundary_groups(scene, &result.boundary_element_ids));
    fill.extra.insert("groupIds".to_owned(), json!(group_ids));

    let anchor_index = scene
        .elements
        .iter()
        .position(|element| element.id == result.insertion.element_id);
    let insertion_index = anchor_index.map_or(scene.elements.len(), |index| {
        if result.insertion.placement == BucketFillPlacement::Above {
            index + 1
        } else {
            index
        }
    });
    scene.elements.insert(insertion_index, fill);
    scene.sync_fractional_indices();
    Ok(BucketFillMutation::Inserted { element_id: id })
}

pub fn is_bucket_fill_compatible(element: &DrawingElement) -> bool {
    element.kind == "line"
        && element
            .extra
            .get("polygon")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        && element.stroke_color.eq_ignore_ascii_case("transparent")
}

fn restyle(scene: &mut DrawingScene, id: &str, style: &DrawingCurrentStyle) -> BucketFillMutation {
    let Some(index) = scene
        .elements
        .iter()
        .position(|element| element.id == id && !element.is_deleted)
    else {
        return BucketFillMutation::Unchanged {
            element_id: id.to_owned(),
        };
    };
    let element = &mut scene.elements[index];
    let background = bucket_background(&style.background_color);
    let opacity = style.opacity.clamp(0.0, 100.0);
    if element.background_color == background
        && element.fill_style == style.fill_style
        && (element.opacity - opacity).abs() <= f32::EPSILON
    {
        return BucketFillMutation::Unchanged {
            element_id: id.to_owned(),
        };
    }
    element.background_color = background.to_owned();
    element.fill_style = style.fill_style.clone();
    element.opacity = opacity;
    touch(element);
    BucketFillMutation::Restyled {
        element_id: id.to_owned(),
    }
}

fn bucket_background(background: &str) -> &str {
    if background.is_empty() || background.eq_ignore_ascii_case("transparent") {
        DEFAULT_BUCKET_FILL_BACKGROUND
    } else {
        background
    }
}

fn topmost_hit(scene: &DrawingScene, point: [f32; 2]) -> Option<&DrawingElement> {
    scene.visible_elements().rev().find(|element| {
        if is_bucket_fill_compatible(element) {
            point_in_polygon(point, &absolute_points(element))
        } else {
            element.hit_test_with_tolerance(point, 0.0)
        }
    })
}

fn top_frame_at(scene: &DrawingScene, point: [f32; 2]) -> Option<&DrawingElement> {
    scene.visible_elements().rev().find(|frame| {
        if !matches!(frame.kind.as_str(), "frame" | "magicframe") {
            return false;
        }
        let (x, y, width, height) = frame.bounds();
        point[0] >= x && point[0] <= x + width && point[1] >= y && point[1] <= y + height
    })
}

fn group_ids_of(element: &DrawingElement) -> Vec<String> {
    element
        .extra
        .get("groupIds")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn common_boundary_groups(scene: &DrawingScene, boundary_ids: &[String]) -> Vec<String> {
    let mut groups = boundary_ids
        .iter()
        .filter_map(|id| scene.element_by_id(id))
        .map(group_ids_of);
    let Some(mut common) = groups.next() else {
        return Vec::new();
    };
    for group in groups {
        common.retain(|id| group.iter().any(|candidate| candidate == id));
    }
    common
}

fn is_restylable_fill(element: &DrawingElement, scene_points: &[[f32; 2]]) -> bool {
    if !is_bucket_fill_compatible(element) {
        return false;
    }
    let ring = absolute_points(element);
    if ring.len() < 3 || scene_points.len() < 3 {
        return false;
    }
    let fill_area = polygon_area(&ring).abs();
    let region_area = polygon_area(scene_points).abs();
    let same_area = (fill_area - region_area).abs() <= 0.05 * fill_area.max(region_area);
    let a = bounds(&ring);
    let b = bounds(scene_points);
    let same_bounds = (a[0] - b[0]).abs() <= 2.0
        && (a[1] - b[1]).abs() <= 2.0
        && (a[2] - b[2]).abs() <= 2.0
        && (a[3] - b[3]).abs() <= 2.0;
    same_area && same_bounds
}

fn absolute_points(element: &DrawingElement) -> Vec<[f32; 2]> {
    element
        .points
        .iter()
        .map(|point| [element.x + point[0], element.y + point[1]])
        .collect()
}

fn point_size(points: &[[f32; 2]]) -> (f32, f32) {
    let bounds = bounds(points);
    ((bounds[2] - bounds[0]).abs(), (bounds[3] - bounds[1]).abs())
}

fn bounds(points: &[[f32; 2]]) -> [f32; 4] {
    points.iter().fold(
        [
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ],
        |mut bounds, point| {
            bounds[0] = bounds[0].min(point[0]);
            bounds[1] = bounds[1].min(point[1]);
            bounds[2] = bounds[2].max(point[0]);
            bounds[3] = bounds[3].max(point[1]);
            bounds
        },
    )
}

fn polygon_area(points: &[[f32; 2]]) -> f32 {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .map(|(a, b)| a[0] * b[1] - b[0] * a[1])
        .sum::<f32>()
        / 2.0
}

fn point_in_polygon(point: [f32; 2], polygon: &[[f32; 2]]) -> bool {
    if polygon.len() < 3 {
        return false;
    }
    let mut inside = false;
    let mut previous = polygon[polygon.len() - 1];
    for current in polygon.iter().copied() {
        if (current[1] > point[1]) != (previous[1] > point[1]) {
            let x = (previous[0] - current[0]) * (point[1] - current[1])
                / (previous[1] - current[1])
                + current[0];
            if point[0] < x {
                inside = !inside;
            }
        }
        previous = current;
    }
    inside
}

fn touch(element: &mut DrawingElement) {
    let version = element
        .extra
        .get("version")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        .saturating_add(1);
    let nonce = element
        .extra
        .get("versionNonce")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        .wrapping_mul(1_664_525)
        .wrapping_add(1_013_904_223);
    element.extra.insert("version".to_owned(), json!(version));
    element
        .extra
        .insert("versionNonce".to_owned(), json!(nonce));
}
