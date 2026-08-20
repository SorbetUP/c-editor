use crate::{rotate_point, DrawingElement, DrawingScene};
use std::collections::{BTreeMap, BTreeSet, HashSet};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BucketFillOptions {
    pub snap_epsilon: f32,
    pub gap_tolerance: f32,
    pub min_area: f32,
    pub max_boundary_segments: usize,
    pub max_generated_points: usize,
    pub fallback_search_radius: f32,
}

impl Default for BucketFillOptions {
    fn default() -> Self {
        Self {
            snap_epsilon: 0.5,
            gap_tolerance: 6.0,
            min_area: 4.0,
            max_boundary_segments: 2560,
            max_generated_points: 1536,
            fallback_search_radius: 512.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BucketFillFailureReason {
    NoOwner,
    OpenRegion,
    TooComplex,
    TooSmall,
    InvalidPolygon,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BucketFillPlacement {
    Above,
    Below,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BucketFillInsertion {
    pub placement: BucketFillPlacement,
    pub element_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BucketFillResult {
    pub owner_id: Option<String>,
    pub boundary_element_ids: Vec<String>,
    pub scene_points: Vec<[f32; 2]>,
    pub insertion: BucketFillInsertion,
}

#[derive(Clone, Debug)]
struct SourceSegment {
    a: [f32; 2],
    b: [f32; 2],
    owner: String,
}

#[derive(Clone, Debug)]
struct Face {
    ring: Vec<[f32; 2]>,
    contributors: BTreeSet<String>,
    component: usize,
    signed_area: f32,
}

#[derive(Default)]
struct Graph {
    nodes: Vec<[f32; 2]>,
    adjacency: Vec<BTreeSet<usize>>,
    owners: BTreeMap<(usize, usize), BTreeSet<String>>,
}

pub fn compute_bucket_fill(
    scene: &DrawingScene,
    point: [f32; 2],
    options: BucketFillOptions,
) -> Result<BucketFillResult, BucketFillFailureReason> {
    let visible = scene
        .elements
        .iter()
        .filter(|element| !element.is_deleted)
        .collect::<Vec<_>>();
    if visible.is_empty() {
        return Err(BucketFillFailureReason::NoOwner);
    }

    let owner = visible
        .iter()
        .rev()
        .copied()
        .find(|element| is_closed_owner(element, point, options.gap_tolerance));
    let candidate_bounds = owner.map_or(
        [
            point[0] - options.fallback_search_radius,
            point[1] - options.fallback_search_radius,
            point[0] + options.fallback_search_radius,
            point[1] + options.fallback_search_radius,
        ],
        |element| {
            let [x1, y1, x2, y2] = element_bounds(element);
            [
                x1 - options.gap_tolerance,
                y1 - options.gap_tolerance,
                x2 + options.gap_tolerance,
                y2 + options.gap_tolerance,
            ]
        },
    );

    let mut segments = Vec::new();
    for element in &visible {
        if !bounds_intersect(candidate_bounds, element_bounds(element)) {
            continue;
        }
        segments.extend(element_segments(element));
        if segments.len() > options.max_boundary_segments {
            return Err(BucketFillFailureReason::TooComplex);
        }
    }
    if segments.is_empty() {
        return Err(if owner.is_some() {
            BucketFillFailureReason::OpenRegion
        } else {
            BucketFillFailureReason::NoOwner
        });
    }

    let split = split_at_intersections(&segments, options.snap_epsilon);
    if split.len() > options.max_boundary_segments.saturating_mul(4) {
        return Err(BucketFillFailureReason::TooComplex);
    }
    let mut graph = build_graph(&split, options.snap_epsilon);
    bridge_loose_ends(&mut graph, options.gap_tolerance, options.snap_epsilon);
    let components = connected_components(&graph);
    let faces = walk_faces(&graph, &components);
    let Some((selected, holes)) = select_face(&faces, point, options.min_area) else {
        return Err(BucketFillFailureReason::OpenRegion);
    };

    let mut scene_points = simplify_ring(selected.ring.clone(), options.snap_epsilon);
    if scene_points.len() < 3 {
        return Err(BucketFillFailureReason::InvalidPolygon);
    }
    let mut hole_indices = (0..holes.len()).collect::<Vec<_>>();
    hole_indices.sort_by(|a, b| {
        holes[*b]
            .signed_area
            .abs()
            .total_cmp(&holes[*a].signed_area.abs())
    });
    for index in hole_indices {
        let hole = simplify_ring(holes[index].ring.clone(), options.snap_epsilon);
        if hole.len() < 3 || scene_points.len() + hole.len() + 2 > options.max_generated_points {
            continue;
        }
        scene_points = splice_hole(scene_points, hole);
    }
    if scene_points.len() > options.max_generated_points || scene_points.len() < 3 {
        return Err(BucketFillFailureReason::InvalidPolygon);
    }
    if scene_points.first() != scene_points.last() {
        scene_points.push(scene_points[0]);
    }
    if polygon_area(&scene_points).abs() < options.min_area {
        return Err(BucketFillFailureReason::TooSmall);
    }

    let mut contributors = selected.contributors.clone();
    for hole in &holes {
        contributors.extend(hole.contributors.iter().cloned());
    }
    let owner_id = owner.map(|element| element.id.clone());
    let boundary_element_ids = contributors
        .iter()
        .filter(|id| owner_id.as_deref() != Some(id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let insertion = choose_insertion(&visible, owner, &contributors, &scene_points, point)
        .ok_or(BucketFillFailureReason::InvalidPolygon)?;

    Ok(BucketFillResult {
        owner_id,
        boundary_element_ids,
        scene_points,
        insertion,
    })
}

fn split_at_intersections(segments: &[SourceSegment], epsilon: f32) -> Vec<SourceSegment> {
    let mut parameters = vec![vec![0.0_f32, 1.0]; segments.len()];
    for i in 0..segments.len() {
        for j in i + 1..segments.len() {
            if let Some((ta, tb)) = intersection_parameters(
                segments[i].a,
                segments[i].b,
                segments[j].a,
                segments[j].b,
                epsilon,
            ) {
                parameters[i].push(ta);
                parameters[j].push(tb);
            }
        }
    }
    let mut output = Vec::new();
    for (segment, mut values) in segments.iter().zip(parameters) {
        values.sort_by(f32::total_cmp);
        values.dedup_by(|a, b| (*a - *b).abs() <= 1e-5);
        for pair in values.windows(2) {
            if pair[1] - pair[0] <= 1e-5 {
                continue;
            }
            let a = interpolate(segment.a, segment.b, pair[0]);
            let b = interpolate(segment.a, segment.b, pair[1]);
            if distance(a, b) <= epsilon * 0.1 {
                continue;
            }
            output.push(SourceSegment {
                a,
                b,
                owner: segment.owner.clone(),
            });
        }
    }
    output
}

fn build_graph(segments: &[SourceSegment], epsilon: f32) -> Graph {
    let mut graph = Graph::default();
    for segment in segments {
        let a = graph.node(segment.a, epsilon);
        let b = graph.node(segment.b, epsilon);
        if a != b {
            graph.add_edge(a, b, Some(&segment.owner));
        }
    }
    graph
}

impl Graph {
    fn node(&mut self, point: [f32; 2], epsilon: f32) -> usize {
        if let Some(index) = self
            .nodes
            .iter()
            .position(|existing| distance(*existing, point) <= epsilon)
        {
            return index;
        }
        self.nodes.push(point);
        self.adjacency.push(BTreeSet::new());
        self.nodes.len() - 1
    }

    fn add_edge(&mut self, a: usize, b: usize, owner: Option<&str>) {
        if a == b {
            return;
        }
        self.adjacency[a].insert(b);
        self.adjacency[b].insert(a);
        if let Some(owner) = owner {
            self.owners
                .entry(edge_key(a, b))
                .or_default()
                .insert(owner.to_owned());
        }
    }

    fn remove_edge(&mut self, a: usize, b: usize) -> BTreeSet<String> {
        self.adjacency[a].remove(&b);
        self.adjacency[b].remove(&a);
        self.owners.remove(&edge_key(a, b)).unwrap_or_default()
    }
}

fn bridge_loose_ends(graph: &mut Graph, tolerance: f32, epsilon: f32) {
    let loose = graph
        .adjacency
        .iter()
        .enumerate()
        .filter_map(|(index, neighbours)| (neighbours.len() == 1).then_some(index))
        .collect::<Vec<_>>();
    for node in loose {
        if graph
            .adjacency
            .get(node)
            .is_none_or(|edges| edges.len() != 1)
        {
            continue;
        }
        let point = graph.nodes[node];
        let best_node = graph
            .nodes
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != node && !graph.adjacency[node].contains(index))
            .map(|(index, candidate)| (index, distance(point, *candidate)))
            .filter(|(_, distance)| *distance <= tolerance)
            .min_by(|a, b| a.1.total_cmp(&b.1));

        let edges = graph.owners.keys().copied().collect::<Vec<_>>();
        let best_edge = edges
            .into_iter()
            .filter(|(a, b)| *a != node && *b != node)
            .filter_map(|(a, b)| {
                let (projection, t) = project_to_segment(point, graph.nodes[a], graph.nodes[b]);
                (t > 1e-4 && t < 1.0 - 1e-4).then(|| {
                    let d = distance(point, projection);
                    (a, b, projection, d)
                })
            })
            .filter(|(_, _, _, distance)| *distance <= tolerance)
            .min_by(|a, b| a.3.total_cmp(&b.3));

        match (best_node, best_edge) {
            (Some((_target, node_distance)), Some((a, b, projection, edge_distance)))
                if edge_distance + epsilon < node_distance =>
            {
                let owners = graph.remove_edge(a, b);
                let projection_index = graph.node(projection, epsilon);
                for owner in &owners {
                    graph.add_edge(a, projection_index, Some(owner));
                    graph.add_edge(projection_index, b, Some(owner));
                }
                graph.add_edge(node, projection_index, None);
            }
            (Some((target, _)), _) => graph.add_edge(node, target, None),
            (None, Some((a, b, projection, _))) => {
                let owners = graph.remove_edge(a, b);
                let projection_index = graph.node(projection, epsilon);
                for owner in &owners {
                    graph.add_edge(a, projection_index, Some(owner));
                    graph.add_edge(projection_index, b, Some(owner));
                }
                graph.add_edge(node, projection_index, None);
            }
            (None, None) => {}
        }
    }
}

fn connected_components(graph: &Graph) -> Vec<usize> {
    let mut component = vec![usize::MAX; graph.nodes.len()];
    let mut next = 0;
    for start in 0..graph.nodes.len() {
        if component[start] != usize::MAX || graph.adjacency[start].is_empty() {
            continue;
        }
        let mut stack = vec![start];
        component[start] = next;
        while let Some(node) = stack.pop() {
            for neighbour in &graph.adjacency[node] {
                if component[*neighbour] == usize::MAX {
                    component[*neighbour] = next;
                    stack.push(*neighbour);
                }
            }
        }
        next += 1;
    }
    component
}

fn walk_faces(graph: &Graph, components: &[usize]) -> Vec<Face> {
    let mut sorted = Vec::with_capacity(graph.nodes.len());
    for (node, neighbours) in graph.adjacency.iter().enumerate() {
        let mut neighbours = neighbours.iter().copied().collect::<Vec<_>>();
        neighbours.sort_by(|a, b| {
            angle(graph.nodes[node], graph.nodes[*a])
                .total_cmp(&angle(graph.nodes[node], graph.nodes[*b]))
        });
        sorted.push(neighbours);
    }
    let mut visited = HashSet::new();
    let mut faces = Vec::new();
    let max_steps = graph.owners.len().saturating_mul(2).saturating_add(8);
    for from in 0..graph.nodes.len() {
        for first in &sorted[from] {
            if visited.contains(&(from, *first)) {
                continue;
            }
            let mut ring = Vec::new();
            let mut a = from;
            let mut b = *first;
            let mut steps = 0;
            loop {
                if steps > max_steps || sorted[b].is_empty() {
                    ring.clear();
                    break;
                }
                steps += 1;
                visited.insert((a, b));
                ring.push(a);
                let outs = &sorted[b];
                let Some(twin) = outs.iter().position(|candidate| *candidate == a) else {
                    ring.clear();
                    break;
                };
                let next = outs[(twin + outs.len() - 1) % outs.len()];
                a = b;
                b = next;
                if a == from && b == *first {
                    break;
                }
            }
            if ring.len() < 3 {
                continue;
            }
            let points = ring
                .iter()
                .map(|index| graph.nodes[*index])
                .collect::<Vec<_>>();
            let signed_area = polygon_area(&points);
            if signed_area.abs() <= f32::EPSILON {
                continue;
            }
            let mut contributors = BTreeSet::new();
            for index in 0..ring.len() {
                let owners = graph
                    .owners
                    .get(&edge_key(ring[index], ring[(index + 1) % ring.len()]));
                if let Some(owners) = owners {
                    contributors.extend(owners.iter().cloned());
                }
            }
            faces.push(Face {
                ring: points,
                contributors,
                component: components[ring[0]],
                signed_area,
            });
        }
    }
    faces
}

fn select_face<'a>(
    faces: &'a [Face],
    point: [f32; 2],
    min_area: f32,
) -> Option<(&'a Face, Vec<&'a Face>)> {
    let selected = faces
        .iter()
        .filter(|face| face.signed_area > 0.0)
        .filter(|face| face.signed_area >= min_area)
        .filter(|face| point_in_polygon(point, &face.ring))
        .min_by(|a, b| a.signed_area.total_cmp(&b.signed_area))?;

    let candidates = faces
        .iter()
        .filter(|face| face.component != selected.component)
        .filter(|face| face.signed_area < -min_area)
        .filter(|face| !point_in_polygon(point, &face.ring))
        .filter(|face| {
            face.ring
                .first()
                .is_some_and(|p| point_in_polygon(*p, &selected.ring))
        })
        .collect::<Vec<_>>();
    let holes = candidates
        .iter()
        .copied()
        .filter(|hole| {
            !candidates.iter().any(|other| {
                other.component != hole.component
                    && other.ring.first().is_some_and(|_| {
                        hole.ring
                            .first()
                            .is_some_and(|point| point_in_polygon(*point, &other.ring))
                    })
            })
        })
        .collect();
    Some((selected, holes))
}

fn choose_insertion(
    elements: &[&DrawingElement],
    owner: Option<&DrawingElement>,
    contributors: &BTreeSet<String>,
    region: &[[f32; 2]],
    click: [f32; 2],
) -> Option<BucketFillInsertion> {
    let participant_ids = contributors
        .iter()
        .cloned()
        .chain(owner.map(|element| element.id.clone()))
        .collect::<BTreeSet<_>>();
    let mut lowest_participant = None;
    let mut covering = None;
    for element in elements {
        if opaque_fill(element) && paint_overlaps_region(element, region, click) {
            covering = Some(*element);
        }
        if lowest_participant.is_none() && participant_ids.contains(&element.id) {
            lowest_participant = Some(*element);
        }
    }
    covering
        .map(|element| BucketFillInsertion {
            placement: BucketFillPlacement::Above,
            element_id: element.id.clone(),
        })
        .or_else(|| {
            lowest_participant
                .or_else(|| elements.last().copied())
                .map(|element| BucketFillInsertion {
                    placement: BucketFillPlacement::Below,
                    element_id: element.id.clone(),
                })
        })
}

fn paint_overlaps_region(element: &DrawingElement, region: &[[f32; 2]], click: [f32; 2]) -> bool {
    if is_closed_owner(element, click, 0.0) {
        return true;
    }
    let outline = outline_points(element);
    outline
        .iter()
        .any(|point| point_in_polygon(*point, region))
        || region
            .iter()
            .any(|point| point_in_polygon(*point, &outline))
}

fn opaque_fill(element: &DrawingElement) -> bool {
    !element.background_color.is_empty()
        && !element
            .background_color
            .eq_ignore_ascii_case("transparent")
        && element.opacity > 0.0
}

fn is_closed_owner(element: &DrawingElement, point: [f32; 2], gap_tolerance: f32) -> bool {
    let outline = outline_points(element);
    if outline.len() < 3 {
        return false;
    }
    let closed = match element.kind.as_str() {
        "rectangle" | "diamond" | "ellipse" | "frame" | "magicframe" | "image" => true,
        "line" => element
            .extra
            .get("polygon")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        "freedraw" => outline
            .first()
            .zip(outline.last())
            .is_some_and(|(a, b)| distance(*a, *b) <= gap_tolerance.max(0.5)),
        _ => false,
    };
    closed && point_in_polygon(point, &outline)
}

fn element_segments(element: &DrawingElement) -> Vec<SourceSegment> {
    if matches!(element.kind.as_str(), "text" | "embeddable" | "iframe") {
        return Vec::new();
    }
    let points = outline_points(element);
    if points.len() < 2 {
        return Vec::new();
    }
    let closes = matches!(
        element.kind.as_str(),
        "rectangle" | "diamond" | "ellipse" | "frame" | "magicframe" | "image"
    ) || (element.kind == "line"
        && element
            .extra
            .get("polygon")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false));
    let mut output = points
        .windows(2)
        .map(|pair| SourceSegment {
            a: pair[0],
            b: pair[1],
            owner: element.id.clone(),
        })
        .collect::<Vec<_>>();
    if closes && points.first() != points.last() {
        output.push(SourceSegment {
            a: *points.last().unwrap(),
            b: points[0],
            owner: element.id.clone(),
        });
    }
    output
}

fn outline_points(element: &DrawingElement) -> Vec<[f32; 2]> {
    let (x, y, width, height) = element.bounds();
    let center = [x + width / 2.0, y + height / 2.0];
    match element.kind.as_str() {
        "ellipse" => (0..64)
            .map(|index| {
                let angle = std::f32::consts::TAU * index as f32 / 64.0;
                rotate_point(
                    [
                        center[0] + width / 2.0 * angle.cos(),
                        center[1] + height / 2.0 * angle.sin(),
                    ],
                    center,
                    element.angle,
                )
            })
            .collect(),
        "diamond" => [
            [center[0], y],
            [x + width, center[1]],
            [center[0], y + height],
            [x, center[1]],
        ]
        .into_iter()
        .map(|point| rotate_point(point, center, element.angle))
        .collect(),
        "line" | "arrow" | "freedraw" if !element.points.is_empty() => element
            .points
            .iter()
            .map(|[px, py]| rotate_point([element.x + px, element.y + py], center, element.angle))
            .collect(),
        _ => [
            [x, y],
            [x + width, y],
            [x + width, y + height],
            [x, y + height],
        ]
        .into_iter()
        .map(|point| rotate_point(point, center, element.angle))
        .collect(),
    }
}

fn simplify_ring(mut points: Vec<[f32; 2]>, epsilon: f32) -> Vec<[f32; 2]> {
    points.dedup_by(|a, b| distance(*a, *b) < epsilon.max(1e-4));
    if points.len() < 4 {
        return points;
    }
    let mut changed = true;
    while changed && points.len() > 3 {
        changed = false;
        for index in 0..points.len() {
            let prev = points[(index + points.len() - 1) % points.len()];
            let current = points[index];
            let next = points[(index + 1) % points.len()];
            if point_segment_distance(current, prev, next) < 0.05 {
                points.remove(index);
                changed = true;
                break;
            }
        }
    }
    points
}

fn splice_hole(ring: Vec<[f32; 2]>, mut hole: Vec<[f32; 2]>) -> Vec<[f32; 2]> {
    if polygon_area(&ring).signum() == polygon_area(&hole).signum() {
        hole.reverse();
    }
    let mut best = (0, 0, f32::INFINITY);
    for (i, outer) in ring.iter().enumerate() {
        for (j, inner) in hole.iter().enumerate() {
            let d = distance(*outer, *inner);
            if d < best.2 {
                best = (i, j, d);
            }
        }
    }
    let mut output = ring[..=best.0].to_vec();
    for offset in 0..=hole.len() {
        output.push(hole[(best.1 + offset) % hole.len()]);
    }
    output.push(ring[best.0]);
    output.extend_from_slice(&ring[best.0 + 1..]);
    output
}

fn intersection_parameters(
    a: [f32; 2],
    b: [f32; 2],
    c: [f32; 2],
    d: [f32; 2],
    epsilon: f32,
) -> Option<(f32, f32)> {
    let r = [b[0] - a[0], b[1] - a[1]];
    let s = [d[0] - c[0], d[1] - c[1]];
    let denominator = cross_vec(r, s);
    if denominator.abs() <= epsilon * 1e-4 {
        return None;
    }
    let ca = [c[0] - a[0], c[1] - a[1]];
    let t = cross_vec(ca, s) / denominator;
    let u = cross_vec(ca, r) / denominator;
    ((-1e-5..=1.0 + 1e-5).contains(&t) && (-1e-5..=1.0 + 1e-5).contains(&u))
        .then_some((t.clamp(0.0, 1.0), u.clamp(0.0, 1.0)))
}

fn project_to_segment(point: [f32; 2], a: [f32; 2], b: [f32; 2]) -> ([f32; 2], f32) {
    let delta = [b[0] - a[0], b[1] - a[1]];
    let length_sq = delta[0] * delta[0] + delta[1] * delta[1];
    if length_sq <= f32::EPSILON {
        return (a, 0.0);
    }
    let t = (((point[0] - a[0]) * delta[0] + (point[1] - a[1]) * delta[1]) / length_sq)
        .clamp(0.0, 1.0);
    (interpolate(a, b, t), t)
}

fn point_segment_distance(point: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    distance(point, project_to_segment(point, a, b).0)
}

fn point_in_polygon(point: [f32; 2], polygon: &[[f32; 2]]) -> bool {
    if polygon.len() < 3 {
        return false;
    }
    let mut inside = false;
    let mut previous = polygon[polygon.len() - 1];
    for current in polygon.iter().copied() {
        if point_segment_distance(point, previous, current) <= 1e-4 {
            return true;
        }
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

fn element_bounds(element: &DrawingElement) -> [f32; 4] {
    let (x, y, width, height) = element.bounds();
    [x, y, x + width, y + height]
}

fn bounds_intersect(a: [f32; 4], b: [f32; 4]) -> bool {
    a[0] <= b[2] && a[2] >= b[0] && a[1] <= b[3] && a[3] >= b[1]
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
        / 2.0
}

fn angle(a: [f32; 2], b: [f32; 2]) -> f32 {
    (b[1] - a[1]).atan2(b[0] - a[0])
}

fn edge_key(a: usize, b: usize) -> (usize, usize) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

fn interpolate(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

fn cross_vec(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[1] - a[1] * b[0]
}

fn distance(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}
