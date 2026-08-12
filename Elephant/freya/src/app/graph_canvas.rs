//! Native Freya rendering for Elephant's production knowledge graph.
//!
//! Tauri's `AtomicGraphView.vue` remains the visual/interaction reference:
//! the graph uses the real knowledge-core projection, connectivity-scaled
//! nodes, cluster colors, weighted links, label thresholds, a focus camera,
//! pan/zoom, hover and a note preview card.  No renderer-side sample graph or
//! arbitrary node-count limit is introduced here.

use super::explorer::ExplorerState;
use crate::{
    search_graph_contract::{GraphCluster, GraphEdge, GraphEdgeType, GraphNode, GraphNodeKind},
    theme,
};
use freya::prelude::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    f32::consts::{PI, TAU},
};

const BASE_WORLD_WIDTH: f32 = 1_800.;
const BASE_WORLD_HEIGHT: f32 = 1_200.;
const STAGE_PADDING: f32 = 40.;
const LABEL_THRESHOLD: f32 = 7.;
const MIN_ZOOM: f32 = 0.125;
const MAX_ZOOM: f32 = 20.;
const FOCUS_ZOOM_MULTIPLIER: f32 = 2.5;

// `graphThemes.js` -> Midnight/dark, the Tauri graph store's default theme.
const GRAPH_BG: (u8, u8, u8) = (18, 20, 28);
const GRAPH_LABEL: (u8, u8, u8) = (112, 136, 176);
const GRAPH_LABEL_ACTIVE: (u8, u8, u8) = (220, 228, 246);
const GRAPH_CARD_BG: (u8, u8, u8) = (16, 18, 26);
const GRAPH_CARD_BORDER: (u8, u8, u8) = (70, 90, 150);
const MIDNIGHT_NODE_PALETTE: [(u8, u8, u8); 8] = [
    (90, 130, 240),
    (60, 180, 190),
    (150, 90, 220),
    (80, 190, 140),
    (180, 80, 160),
    (100, 160, 210),
    (200, 130, 80),
    (120, 100, 220),
];

#[derive(Clone, Debug, PartialEq)]
enum DragState {
    Pan {
        pointer: [f32; 2],
        pan: [f32; 2],
    },
    Node {
        id: String,
        pointer: [f32; 2],
        position: [f32; 2],
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphCanvasState {
    pan: [f32; 2],
    zoom: f32,
    size: [f32; 2],
    origin: [f32; 2],
    content_size: [f32; 2],
    dragging: Option<DragState>,
    hovered: Option<String>,
    overrides: BTreeMap<String, [f32; 2]>,
    fitted: bool,
}

impl Default for GraphCanvasState {
    fn default() -> Self {
        Self {
            pan: [0., 0.],
            zoom: 0.5,
            size: [800., 560.],
            origin: [0., 0.],
            content_size: [BASE_WORLD_WIDTH, BASE_WORLD_HEIGHT],
            dragging: None,
            hovered: None,
            overrides: BTreeMap::new(),
            fitted: false,
        }
    }
}

impl GraphCanvasState {
    fn viewport_label(&self) -> String {
        format!(
            "Graph canvas viewport pan=({:.1},{:.1}) zoom={:.3}",
            self.pan[0], self.pan[1], self.zoom
        )
    }

    fn set_size(&mut self, area: Area) {
        let next_size = [area.size.width.max(1.), area.size.height.max(1.)];
        let resized = self.size != next_size;
        self.size = next_size;
        self.origin = [area.origin.x, area.origin.y];
        if resized || !self.fitted {
            self.fit_to_content();
        }
    }

    fn set_content_size(&mut self, content_size: [f32; 2]) {
        if self.content_size == content_size {
            return;
        }
        self.content_size = content_size;
        self.fit_to_content();
    }

    pub fn fit_to_content(&mut self) {
        let available_width = (self.size[0] - STAGE_PADDING * 2.).max(1.);
        let available_height = (self.size[1] - STAGE_PADDING * 2.).max(1.);
        let scale_x = available_width / self.content_size[0].max(1.);
        let scale_y = available_height / self.content_size[1].max(1.);
        self.zoom = scale_x.min(scale_y).clamp(MIN_ZOOM, MAX_ZOOM);
        self.pan = [
            (self.size[0] - self.content_size[0] * self.zoom) / 2.,
            (self.size[1] - self.content_size[1] * self.zoom) / 2.,
        ];
        self.fitted = true;
    }

    fn focus_on(&mut self, position: [f32; 2]) {
        let available_width = (self.size[0] - STAGE_PADDING * 2.).max(1.);
        let available_height = (self.size[1] - STAGE_PADDING * 2.).max(1.);
        let fit_zoom = (available_width / self.content_size[0].max(1.))
            .min(available_height / self.content_size[1].max(1.));
        // Tauri focuses a selected node with Sigma camera ratio 0.4, i.e.
        // 2.5x the reset scale.
        self.zoom = (fit_zoom * FOCUS_ZOOM_MULTIPLIER).clamp(MIN_ZOOM, MAX_ZOOM);
        self.center_on(position);
    }

    fn center_on(&mut self, position: [f32; 2]) {
        self.pan = [
            self.size[0] / 2. - position[0] * self.zoom,
            self.size[1] / 2. - position[1] * self.zoom,
        ];
    }

    fn local_pointer(&self, pointer: [f32; 2]) -> [f32; 2] {
        [pointer[0] - self.origin[0], pointer[1] - self.origin[1]]
    }

    fn world_at(&self, pointer: [f32; 2]) -> [f32; 2] {
        let local = self.local_pointer(pointer);
        [
            (local[0] - self.pan[0]) / self.zoom,
            (local[1] - self.pan[1]) / self.zoom,
        ]
    }

    fn screen_position(&self, position: [f32; 2]) -> [f32; 2] {
        [
            self.pan[0] + position[0] * self.zoom,
            self.pan[1] + position[1] * self.zoom,
        ]
    }

    fn begin_pan(&mut self, pointer: [f32; 2]) {
        self.dragging = Some(DragState::Pan {
            pointer,
            pan: self.pan,
        });
    }

    fn begin_node_drag(&mut self, id: String, pointer: [f32; 2], position: [f32; 2]) {
        self.dragging = Some(DragState::Node {
            id,
            pointer,
            position,
        });
    }

    fn move_pointer(&mut self, pointer: [f32; 2]) {
        match self.dragging.clone() {
            Some(DragState::Pan {
                pointer: start,
                pan,
            }) => {
                let dx = pointer[0] - start[0];
                let dy = pointer[1] - start[1];
                self.pan = [pan[0] + dx, pan[1] + dy];
            }
            Some(DragState::Node {
                id,
                pointer: start,
                position,
            }) => {
                self.overrides.insert(
                    id,
                    [
                        position[0] + (pointer[0] - start[0]) / self.zoom,
                        position[1] + (pointer[1] - start[1]) / self.zoom,
                    ],
                );
            }
            None => {}
        }
    }

    fn end_pointer(&mut self) {
        self.dragging = None;
    }

    fn set_hovered(&mut self, id: Option<String>) {
        if self.hovered != id {
            self.hovered = id;
        }
    }

    fn is_dragging(&self) -> bool {
        self.dragging.is_some()
    }

    fn zoom_at(&mut self, pointer: [f32; 2], delta_y: f64) {
        let local = self.local_pointer(pointer);
        let before = self.world_at(pointer);
        let factor = if delta_y > 0. { 0.9 } else { 1.1 };
        self.zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        self.pan = [
            local[0] - before[0] * self.zoom,
            local[1] - before[1] * self.zoom,
        ];
    }
}

#[derive(Clone, Debug)]
struct CanvasNode {
    id: String,
    title: String,
    kind: GraphNodeKind,
    position: [f32; 2],
    radius: f32,
    connectivity: f32,
    cluster_index: usize,
}

pub fn render(
    mut explorer: State<ExplorerState>,
    snapshot: &ExplorerState,
    mut canvas: State<GraphCanvasState>,
) -> Element {
    let graph_node_count = snapshot
        .graph
        .snapshot
        .as_ref()
        .map_or(0, |graph| graph.nodes.len());
    let content_size = world_size_for_count(graph_node_count);
    if canvas.read().content_size != content_size {
        canvas.write().set_content_size(content_size);
    }

    let nodes = layout_nodes(snapshot, &canvas.read().overrides);
    let visible_ids = nodes
        .iter()
        .filter(|node| {
            snapshot
                .graph
                .snapshot
                .as_ref()
                .and_then(|graph| graph.nodes.iter().find(|candidate| candidate.id == node.id))
                .is_some_and(|candidate| snapshot.graph.filters.matches_current_vue(candidate))
        })
        .map(|node| node.id.as_str())
        .collect::<BTreeSet<_>>();
    let viewport = canvas.read().clone();
    let edges = snapshot.visible_graph_edges();
    let selected_id = snapshot.graph.selected_node_id.clone();
    let hovered_id = viewport.hovered.clone();
    let active_id = selected_id.as_deref().or(hovered_id.as_deref());
    let neighbors = build_neighbors(edges.iter().copied());

    let visible_weights = edges
        .iter()
        .filter(|edge| {
            visible_ids.contains(edge.source.as_str()) && visible_ids.contains(edge.target.as_str())
        })
        .map(|edge| edge.weight)
        .collect::<Vec<_>>();
    let min_weight = visible_weights.iter().copied().reduce(f32::min).unwrap_or(0.);
    let max_weight = visible_weights.iter().copied().reduce(f32::max).unwrap_or(1.);

    let mut edge_elements = Vec::new();
    for edge in edges {
        if !visible_ids.contains(edge.source.as_str())
            || !visible_ids.contains(edge.target.as_str())
        {
            continue;
        }
        let Some(source) = nodes.iter().find(|node| node.id == edge.source) else {
            continue;
        };
        let Some(target) = nodes.iter().find(|node| node.id == edge.target) else {
            continue;
        };
        let emphasized = active_id.is_some_and(|active| edge.source == active || edge.target == active);
        edge_elements.push(render_edge(
            edge,
            source.position,
            target.position,
            &viewport,
            normalize_edge_weight(edge.weight, min_weight, max_weight),
            emphasized,
            active_id.is_some(),
        ));
    }

    let mut node_elements = Vec::new();
    for node in nodes
        .iter()
        .filter(|node| visible_ids.contains(node.id.as_str()))
    {
        let selected = selected_id.as_deref() == Some(node.id.as_str());
        let hovered = hovered_id.as_deref() == Some(node.id.as_str());
        let neighboring = active_id.is_some_and(|active| {
            neighbors
                .get(active)
                .is_some_and(|set| set.contains(node.id.as_str()))
        });
        let display_radius = display_radius(node.radius, selected, hovered, neighboring, active_id.is_some());
        let center = viewport.screen_position(node.position);
        let id = node.id.clone();
        let position = node.position;
        let mut node_canvas = canvas.clone();
        let mut node_explorer = explorer.clone();
        let label_visible = selected || hovered || display_radius >= LABEL_THRESHOLD;

        node_elements.push(
            rect()
                .key(("graph-node", node.id.clone()))
                .position(
                    Position::new_absolute()
                        .left(center[0] - display_radius)
                        .top(center[1] - display_radius),
                )
                .width(Size::px(display_radius * 2.))
                .height(Size::px(display_radius * 2.))
                .background(node_color(node.cluster_index, node.connectivity))
                .with_corner_radius(display_radius)
                .on_mouse_down(move |event: Event<MouseEventData>| {
                    if event.button == Some(MouseButton::Left) {
                        let pointer = point(event.global_location);
                        node_canvas
                            .write()
                            .begin_node_drag(id.clone(), pointer, position);
                        node_canvas.write().focus_on(position);
                        node_explorer.write().select_graph_node(id.clone());
                        event.stop_propagation();
                    }
                })
                .a11y_alt(format!("Select graph node {}", node.title))
                .into_element(),
        );

        if label_visible {
            let font_size = if selected { 14. } else if hovered { 13. } else { 12. };
            node_elements.push(
                label()
                    .key(("graph-node-title", node.id.clone()))
                    .position(
                        Position::new_absolute()
                            .left(center[0] + display_radius + 7.)
                            .top(center[1] - font_size / 2.),
                    )
                    .width(Size::px(190.))
                    .height(Size::px(22.))
                    .font_size(font_size)
                    .font_weight(if selected {
                        FontWeight::BOLD
                    } else if hovered {
                        FontWeight::SEMI_BOLD
                    } else {
                        FontWeight::MEDIUM
                    })
                    .color(rgb(if selected || hovered {
                        GRAPH_LABEL_ACTIVE
                    } else {
                        GRAPH_LABEL
                    }))
                    .text(trunc_label(&node.title, 28))
                    .into_element(),
            );
        }
    }

    let stats_text = format!("{} nœuds · {} liens", visible_ids.len(), edge_elements.len());
    let viewport_label = viewport.viewport_label();
    let selected_card = selected_card(&mut explorer, snapshot, &nodes, &viewport);
    let mut size_canvas = canvas.clone();
    let mut stage_canvas = canvas.clone();
    let mut stage_explorer = explorer.clone();
    let mut move_canvas = canvas.clone();
    let mut end_canvas = canvas.clone();
    let mut wheel_canvas = canvas.clone();
    let mut reset_canvas = canvas.clone();
    let hit_nodes = nodes.clone();
    let click_hit_nodes = nodes.clone();
    let stats_top = (viewport.size[1] - 34.).max(8.);
    let reset_top = (viewport.size[1] - 72.).max(8.);
    let zoom_percent = (viewport.zoom * 100.).round();

    rect()
        .expanded()
        .background(rgb(GRAPH_BG))
        .overflow(Overflow::Clip)
        .a11y_alt(viewport_label)
        .on_sized(move |event: Event<SizedEventData>| {
            size_canvas.write().set_size(event.area);
        })
        .on_mouse_down(move |event: Event<MouseEventData>| {
            if event.button == Some(MouseButton::Left) {
                let pointer = point(event.global_location);
                if let Some(node) = hit_test_node(&click_hit_nodes, &stage_canvas.read(), pointer) {
                    stage_canvas
                        .write()
                        .begin_node_drag(node.id.clone(), pointer, node.position);
                    stage_canvas.write().focus_on(node.position);
                    stage_explorer.write().select_graph_node(node.id.clone());
                } else {
                    stage_canvas.write().begin_pan(pointer);
                }
                event.stop_propagation();
            }
        })
        .on_global_pointer_move(move |event: Event<PointerEventData>| {
            if !event.is_primary() {
                return;
            }
            let pointer = point(event.global_location());
            if move_canvas.read().is_dragging() {
                move_canvas.write().move_pointer(pointer);
            } else {
                let hovered = hit_test_node(&hit_nodes, &move_canvas.read(), pointer)
                    .map(|node| node.id.clone());
                move_canvas.write().set_hovered(hovered);
            }
            event.stop_propagation();
        })
        .on_global_pointer_press(move |event: Event<PointerEventData>| {
            if event.is_primary() {
                end_canvas.write().end_pointer();
            }
        })
        .on_wheel(move |event: Event<WheelEventData>| {
            wheel_canvas
                .write()
                .zoom_at(point(event.global_location), event.delta_y);
            event.stop_propagation();
        })
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .position(Position::new_absolute().left(0.).top(0.))
                .a11y_alt("Graph edges")
                .children(edge_elements),
        )
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .position(Position::new_absolute().left(0.).top(0.))
                .a11y_alt("Graph nodes")
                .children(node_elements),
        )
        .child(
            rect()
                .position(Position::new_absolute().left(12.).top(stats_top))
                .height(Size::px(26.))
                .padding(Gaps::new(0., 12., 0., 12.))
                .center()
                .background(rgb(GRAPH_CARD_BG))
                .border(Border::new().fill(rgb(GRAPH_CARD_BORDER)).width(1.))
                .with_corner_radius(13.)
                .a11y_alt(stats_text.clone())
                .child(
                    label()
                        .font_size(12.)
                        .color(rgb(GRAPH_LABEL))
                        .text(stats_text),
                ),
        )
        .child(
            rect()
                .position(Position::new_absolute().left(12.).top(reset_top))
                .height(Size::px(30.))
                .padding(Gaps::new(0., 10., 0., 10.))
                .center()
                .background(rgb(GRAPH_CARD_BG))
                .border(Border::new().fill(rgb(GRAPH_CARD_BORDER)).width(1.))
                .with_corner_radius(9.)
                .on_mouse_up(move |_| reset_canvas.write().fit_to_content())
                .a11y_alt("Recenter graph")
                .child(
                    label()
                        .font_size(12.)
                        .color(rgb(GRAPH_LABEL_ACTIVE))
                        .text(format!("Recentrer · {zoom_percent:.0}%")),
                ),
        )
        .maybe_child(selected_card)
        .into_element()
}

fn world_size_for_count(node_count: usize) -> [f32; 2] {
    // Exact scaling rule used by AtomicGraphView before building its semantic
    // view model.  This is the important >200 density behavior: the world
    // grows with sqrt(N) instead of silently truncating nodes.
    let scale = (node_count as f32).sqrt().div_euclid(12.).max(1.);
    // `div_euclid` on floats is not the Tauri expression. Correct the quotient
    // to ordinary division while keeping this helper explicit and testable.
    let scale = ((node_count as f32).sqrt() / 12.).max(scale).max(1.);
    [BASE_WORLD_WIDTH * scale, BASE_WORLD_HEIGHT * scale]
}

fn layout_nodes(
    snapshot: &ExplorerState,
    overrides: &BTreeMap<String, [f32; 2]>,
) -> Vec<CanvasNode> {
    let Some(graph) = snapshot.graph.snapshot.as_ref() else {
        return Vec::new();
    };
    layout_graph_nodes(&graph.nodes, &graph.edges, &graph.clusters, overrides)
}

fn layout_graph_nodes(
    graph_nodes: &[GraphNode],
    edges: &[GraphEdge],
    clusters: &[GraphCluster],
    overrides: &BTreeMap<String, [f32; 2]>,
) -> Vec<CanvasNode> {
    let world_size = world_size_for_count(graph_nodes.len());
    let center = [world_size[0] / 2., world_size[1] / 2.];

    let mut edge_counts = BTreeMap::<String, usize>::new();
    for edge in edges {
        *edge_counts.entry(edge.source.clone()).or_default() += 1;
        *edge_counts.entry(edge.target.clone()).or_default() += 1;
    }
    let max_edges = edge_counts.values().copied().max().unwrap_or(1).max(1) as f32;

    let mut cluster_by_path = BTreeMap::<String, usize>::new();
    for (index, cluster) in clusters.iter().enumerate() {
        for path in &cluster.paths {
            cluster_by_path.insert(path.clone(), index);
        }
    }

    let mut groups = BTreeMap::<String, Vec<&GraphNode>>::new();
    for node in graph_nodes {
        let cluster_index = cluster_by_path
            .get(&node.id)
            .or_else(|| cluster_by_path.get(&node.relative_path))
            .copied();
        let group = cluster_index
            .map(|index| format!("cluster-{index:08}"))
            .unwrap_or_else(|| {
                if !node.folder.trim().is_empty() {
                    format!("folder-{}", node.folder)
                } else {
                    format!(
                        "folder-{}",
                        node.relative_path
                            .rsplit_once('/')
                            .map(|(folder, _)| folder)
                            .unwrap_or("root")
                    )
                }
            });
        groups.entry(group).or_default().push(node);
    }
    for group in groups.values_mut() {
        group.sort_by(|left, right| {
            (
                left.kind != GraphNodeKind::Folder,
                left.title.as_str(),
                left.id.as_str(),
            )
                .cmp(&(
                    right.kind != GraphNodeKind::Folder,
                    right.title.as_str(),
                    right.id.as_str(),
                ))
        });
    }

    let cluster_radius = world_size[0].min(world_size[1]) * 0.28;
    let cluster_count = groups.len().max(1);
    let golden_angle = PI * (3. - 5_f32.sqrt());
    let mut positions = BTreeMap::<String, ([f32; 2], usize)>::new();

    for (group_index, (_group_key, group)) in groups.iter().enumerate() {
        let cluster_center = if cluster_count == 1 {
            center
        } else {
            let angle = -PI / 2. + group_index as f32 * TAU / cluster_count as f32;
            [
                center[0] + cluster_radius * angle.cos(),
                center[1] + cluster_radius * angle.sin(),
            ]
        };
        let cluster_index = group
            .iter()
            .find_map(|node| {
                cluster_by_path
                    .get(&node.id)
                    .or_else(|| cluster_by_path.get(&node.relative_path))
                    .copied()
            })
            .unwrap_or(group_index);

        for (index, node) in group.iter().enumerate() {
            // Deterministic phyllotaxis avoids the severe overlap produced by
            // a single orbit for large folders while remaining cheap enough to
            // recompute during native interaction. Source/saved positions win.
            let radius = 36. * (index as f32).sqrt();
            let angle = -PI / 2. + index as f32 * golden_angle;
            let generated = [
                cluster_center[0] + radius * angle.cos(),
                cluster_center[1] + radius * angle.sin(),
            ];
            let position = node
                .position
                .as_ref()
                .map(|value| [value.x, value.y])
                .unwrap_or(generated);
            positions.insert(
                node.id.clone(),
                (
                    overrides.get(&node.id).copied().unwrap_or(position),
                    cluster_index,
                ),
            );
        }
    }

    graph_nodes
        .iter()
        .filter_map(|node| {
            positions
                .get(&node.id)
                .copied()
                .map(|(position, cluster_index)| {
                    let connectivity = edge_counts.get(&node.id).copied().unwrap_or(0) as f32
                        / max_edges;
                    let radius = 3.
                        + connectivity * 6.
                        + if node.kind == GraphNodeKind::Folder {
                            3.
                        } else {
                            0.
                        };
                    CanvasNode {
                        id: node.id.clone(),
                        title: node.title.clone(),
                        kind: node.kind,
                        position,
                        radius,
                        connectivity,
                        cluster_index,
                    }
                })
        })
        .collect()
}

fn build_neighbors<'a>(edges: impl Iterator<Item = &'a GraphEdge>) -> BTreeMap<String, BTreeSet<String>> {
    let mut neighbors = BTreeMap::<String, BTreeSet<String>>::new();
    for edge in edges {
        neighbors
            .entry(edge.source.clone())
            .or_default()
            .insert(edge.target.clone());
        neighbors
            .entry(edge.target.clone())
            .or_default()
            .insert(edge.source.clone());
    }
    neighbors
}

fn display_radius(
    base: f32,
    selected: bool,
    hovered: bool,
    neighboring: bool,
    focus_active: bool,
) -> f32 {
    if selected {
        base * 1.45
    } else if hovered {
        base * 1.35
    } else if neighboring {
        base * 1.12
    } else if focus_active {
        base * 0.5
    } else {
        base
    }
}

fn hit_test_node<'a>(
    nodes: &'a [CanvasNode],
    viewport: &GraphCanvasState,
    pointer: [f32; 2],
) -> Option<&'a CanvasNode> {
    let local = viewport.local_pointer(pointer);
    nodes.iter().find(|node| {
        let center = viewport.screen_position(node.position);
        let hit_radius = node.radius.max(6.) + 7.;
        let dx = local[0] - center[0];
        let dy = local[1] - center[1];
        dx * dx + dy * dy <= hit_radius * hit_radius
    })
}

fn normalize_edge_weight(weight: f32, min_weight: f32, max_weight: f32) -> f32 {
    let range = max_weight - min_weight;
    if range.abs() < f32::EPSILON {
        1.
    } else {
        ((weight - min_weight) / range).clamp(0., 1.)
    }
}

fn render_edge(
    edge: &GraphEdge,
    source: [f32; 2],
    target: [f32; 2],
    viewport: &GraphCanvasState,
    normalized_weight: f32,
    emphasized: bool,
    focus_active: bool,
) -> Element {
    let source = viewport.screen_position(source);
    let target = viewport.screen_position(target);
    let dx = target[0] - source[0];
    let dy = target[1] - source[1];
    let length = (dx * dx + dy * dy).sqrt().max(1.);
    let angle = dy.atan2(dx).to_degrees();
    let mut thickness = 0.25 + normalized_weight * 0.7;
    if emphasized {
        thickness += 0.6;
    } else if focus_active {
        thickness *= 0.3;
    }
    thickness = thickness.max(0.35);
    let midpoint = [(source[0] + target[0]) / 2., (source[1] + target[1]) / 2.];

    rect()
        .key(("graph-edge", edge.id.clone()))
        .position(
            Position::new_absolute()
                .left(midpoint[0] - length / 2.)
                .top(midpoint[1] - thickness / 2.),
        )
        .width(Size::px(length))
        .height(Size::px(thickness))
        .background(edge_color(edge.edge_type))
        .with_corner_radius(2.)
        .rotation(angle)
        .a11y_alt(format!("Graph edge {} to {}", edge.source, edge.target))
        .child(label().font_size(1.).text(" "))
        .into_element()
}

fn selected_card(
    state: &mut State<ExplorerState>,
    snapshot: &ExplorerState,
    nodes: &[CanvasNode],
    viewport: &GraphCanvasState,
) -> Option<Element> {
    let selected_id = snapshot.graph.selected_node_id.as_deref()?;
    let selected_canvas = nodes.iter().find(|node| node.id == selected_id)?;
    let selected = snapshot
        .graph
        .snapshot
        .as_ref()?
        .nodes
        .iter()
        .find(|node| node.id == selected_id)?;
    let center = viewport.screen_position(selected_canvas.position);
    let left = (center[0] + 22.).clamp(12., (viewport.size[0] - 322.).max(12.));
    let top = (center[1] + 22.).clamp(12., (viewport.size[1] - 250.).max(12.));
    let title = selected.title.clone();
    let summary = if selected.summary.trim().is_empty() {
        "Aucun résumé pour cette note.".to_string()
    } else {
        selected.summary.clone()
    };
    let tags = selected
        .tags
        .iter()
        .take(12)
        .map(|tag| format!("#{tag}"))
        .collect::<Vec<_>>()
        .join("  ");
    let meta = format!(
        "{} · {} sources · {} chunks",
        match selected.kind {
            GraphNodeKind::Note => "note",
            GraphNodeKind::Folder => "folder",
            GraphNodeKind::Other => "other",
        },
        selected.source_count,
        selected.chunk_count
    );
    let mut state = state.clone();

    Some(
        rect()
            .position(Position::new_absolute().left(left).top(top))
            .width(Size::px(310.))
            .padding(Gaps::new_all(12.))
            .spacing(8.)
            .background(rgb(GRAPH_CARD_BG))
            .border(Border::new().fill(rgb(GRAPH_CARD_BORDER)).width(1.))
            .with_corner_radius(12.)
            .a11y_alt(format!("Graph node selected {title}"))
            .child(
                label()
                    .font_size(15.)
                    .font_weight(FontWeight::BOLD)
                    .color(rgb(GRAPH_LABEL_ACTIVE))
                    .text(title),
            )
            .child(label().font_size(11.).color(rgb(GRAPH_LABEL)).text(meta))
            .child(
                label()
                    .font_size(12.)
                    .color(rgb(GRAPH_LABEL_ACTIVE))
                    .text(trunc_label(&summary, 180)),
            )
            .maybe_child((!tags.is_empty()).then(|| {
                label()
                    .font_size(11.)
                    .color(Color::from_rgb(155, 108, 255))
                    .text(tags)
            }))
            .child(
                rect()
                    .height(Size::px(32.))
                    .padding(Gaps::new(0., 12., 0., 12.))
                    .center()
                    .background(Color::from_rgb(59, 91, 190))
                    .with_corner_radius(8.)
                    .on_mouse_up(move |_| state.write().open_selected_graph_node())
                    .a11y_alt("Open selected note")
                    .child(
                        label()
                            .font_size(12.)
                            .font_weight(FontWeight::SEMI_BOLD)
                            .color(Color::from_rgb(245, 247, 255))
                            .text("Ouvrir la note"),
                    ),
            )
            .into_element(),
    )
}

fn trunc_label(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let mut truncated = value
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect::<String>();
    truncated.push('…');
    truncated
}

fn node_color(cluster_index: usize, connectivity: f32) -> Color {
    let base = MIDNIGHT_NODE_PALETTE[cluster_index % MIDNIGHT_NODE_PALETTE.len()];
    // Same dark-theme modulation as graphThemes.js. AtomicGraphView passes
    // `0.3 + connectivity` into nodeColor.
    let effective_connectivity = (0.3 + connectivity).clamp(0., 1.);
    let factor = 0.6 + effective_connectivity * 0.4;
    Color::from_rgb(
        (f32::from(base.0) * factor).round() as u8,
        (f32::from(base.1) * factor).round() as u8,
        (f32::from(base.2) * factor).round() as u8,
    )
}

fn edge_color(kind: GraphEdgeType) -> Color {
    match kind {
        GraphEdgeType::Semantic | GraphEdgeType::Related => Color::from_rgb(109, 95, 211),
        GraphEdgeType::ExplicitLink => Color::from_rgb(59, 155, 150),
        GraphEdgeType::Folder => Color::from_rgb(217, 138, 59),
        GraphEdgeType::Tag | GraphEdgeType::Lexical => Color::from_rgb(155, 108, 255),
        GraphEdgeType::Other => Color::from_rgb(85, 92, 120),
    }
}

fn rgb(value: (u8, u8, u8)) -> Color {
    Color::from_rgb(value.0, value.1, value.2)
}

fn point(value: CursorPoint) -> [f32; 2] {
    let (x, y) = value.to_tuple();
    [x as f32, y as f32]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(index: usize, title: impl Into<String>) -> GraphNode {
        GraphNode {
            id: format!("note-{index:03}.md"),
            relative_path: format!("note-{index:03}.md"),
            title: title.into(),
            kind: GraphNodeKind::Note,
            folder: String::new(),
            tags: Vec::new(),
            summary: String::new(),
            headings: Vec::new(),
            key_terms: Vec::new(),
            updated_at: String::new(),
            weak_title: false,
            source_count: 0,
            chunk_count: 1,
            position: None,
        }
    }

    fn link(index: usize, source: usize, target: usize, weight: f32) -> GraphEdge {
        GraphEdge {
            id: format!("edge-{index}"),
            source: format!("note-{source:03}.md"),
            target: format!("note-{target:03}.md"),
            edge_type: GraphEdgeType::ExplicitLink,
            reason: "wiki link".to_string(),
            weight,
        }
    }

    #[test]
    fn tiny_and_medium_graphs_keep_every_node() {
        for count in [1, 16] {
            let nodes = (0..count)
                .map(|index| note(index, format!("Note {index}")))
                .collect::<Vec<_>>();
            let layout = layout_graph_nodes(&nodes, &[], &[], &BTreeMap::new());
            assert_eq!(layout.len(), count);
            assert_eq!(
                layout.iter().map(|node| node.id.as_str()).collect::<BTreeSet<_>>().len(),
                count
            );
        }
    }

    #[test]
    fn layout_has_no_two_hundred_node_cap() {
        let nodes = (0..205)
            .map(|index| {
                let title = if index == 204 {
                    "A deliberately very long graph label that must remain present and truncate only visually".to_string()
                } else {
                    format!("Note {index:03}")
                };
                note(index, title)
            })
            .collect::<Vec<_>>();
        let edges = (0..203)
            .map(|index| link(index, index, index + 1, 1.))
            .collect::<Vec<_>>();
        let layout = layout_graph_nodes(&nodes, &edges, &[], &BTreeMap::new());

        assert_eq!(layout.len(), 205);
        assert!(layout.iter().any(|node| node.id == "note-204.md"));
        assert!(world_size_for_count(205)[0] > BASE_WORLD_WIDTH);
        assert!(world_size_for_count(205)[1] > BASE_WORLD_HEIGHT);
        assert_eq!(
            layout
                .iter()
                .map(|node| (node.position[0].to_bits(), node.position[1].to_bits()))
                .collect::<BTreeSet<_>>()
                .len(),
            205,
            "large deterministic layout must not collapse nodes onto one position"
        );
    }

    #[test]
    fn connectivity_drives_tauri_node_size_formula() {
        let nodes = (0..4)
            .map(|index| note(index, format!("Note {index}")))
            .collect::<Vec<_>>();
        let edges = vec![
            link(0, 0, 1, 0.2),
            link(1, 0, 2, 0.6),
            link(2, 0, 3, 1.0),
        ];
        let layout = layout_graph_nodes(&nodes, &edges, &[], &BTreeMap::new());
        let hub = layout.iter().find(|node| node.id == "note-000.md").unwrap();
        let leaf = layout.iter().find(|node| node.id == "note-001.md").unwrap();

        assert_eq!(hub.radius, 9.);
        assert_eq!(leaf.radius, 5.);
        assert!(hub.connectivity > leaf.connectivity);
    }

    #[test]
    fn zoom_keeps_pointer_world_coordinate_anchored_and_respects_bounds() {
        let mut state = GraphCanvasState::default();
        state.size = [1_000., 700.];
        state.content_size = world_size_for_count(205);
        state.fit_to_content();
        let pointer = [420., 315.];
        let before = state.world_at(pointer);
        state.zoom_at(pointer, -1.);
        let after = state.world_at(pointer);

        assert!((before[0] - after[0]).abs() < 0.01);
        assert!((before[1] - after[1]).abs() < 0.01);
        assert!((MIN_ZOOM..=MAX_ZOOM).contains(&state.zoom));
    }

    #[test]
    fn focus_matches_tauri_selected_node_recentering() {
        let mut state = GraphCanvasState::default();
        state.size = [1_000., 700.];
        state.content_size = [BASE_WORLD_WIDTH, BASE_WORLD_HEIGHT];
        state.fit_to_content();
        let target = [1_350., 860.];
        let reset_zoom = state.zoom;
        state.focus_on(target);
        let screen = state.screen_position(target);

        assert!((screen[0] - state.size[0] / 2.).abs() < 0.01);
        assert!((screen[1] - state.size[1] / 2.).abs() < 0.01);
        assert!(state.zoom > reset_zoom);
    }

    #[test]
    fn long_labels_are_truncated_visually_without_changing_node_identity() {
        let title = "This is a very long graph node label that should not flood the viewport";
        let truncated = trunc_label(title, 28);
        assert_eq!(truncated.chars().count(), 28);
        assert!(truncated.ends_with('…'));
        assert_eq!(trunc_label("Short", 28), "Short");
    }

    #[test]
    fn edge_weight_normalization_preserves_multiple_link_strengths() {
        assert_eq!(normalize_edge_weight(0.2, 0.2, 1.0), 0.0);
        assert!((normalize_edge_weight(0.6, 0.2, 1.0) - 0.5).abs() < f32::EPSILON);
        assert_eq!(normalize_edge_weight(1.0, 0.2, 1.0), 1.0);
        assert_eq!(normalize_edge_weight(1.0, 1.0, 1.0), 1.0);
    }
}
