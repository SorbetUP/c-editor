//! Freya canvas for the real knowledge-core graph projection.
//!
//! The layout follows the historical Atomic graph helper: deterministic
//! folder clusters, note orbits, stable sorting, and source positions when
//! supplied.  The renderer owns only view state (camera, hover, and drag
//! overrides); nodes and edges always come from `GraphSnapshot`.

use super::explorer::ExplorerState;
use crate::{
    search_graph_contract::{GraphEdge, GraphEdgeType, GraphNode, GraphNodeKind},
    theme,
};
use freya::prelude::*;
use std::{collections::BTreeMap, f32::consts::PI};

const WORLD_WIDTH: f32 = 1_800.;
const WORLD_HEIGHT: f32 = 1_200.;
const NODE_RADIUS: f32 = 13.;

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
        self.size = next_size;
        self.origin = [area.origin.x, area.origin.y];
        if !self.fitted {
            self.fit_to_content();
        }
    }

    pub fn fit_to_content(&mut self) {
        let scale_x = self.size[0] / WORLD_WIDTH;
        let scale_y = self.size[1] / WORLD_HEIGHT;
        self.zoom = (scale_x.min(scale_y) * 0.86).clamp(0.2, 2.5);
        self.pan = [
            (self.size[0] - WORLD_WIDTH * self.zoom) / 2.,
            (self.size[1] - WORLD_HEIGHT * self.zoom) / 2.,
        ];
        self.fitted = true;
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
        let before = [
            (local[0] - self.pan[0]) / self.zoom,
            (local[1] - self.pan[1]) / self.zoom,
        ];
        let factor = if delta_y > 0. { 0.9 } else { 1.1 };
        self.zoom = (self.zoom * factor).clamp(0.2, 2.5);
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
}

pub fn render(
    mut explorer: State<ExplorerState>,
    snapshot: &ExplorerState,
    canvas: State<GraphCanvasState>,
) -> Element {
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
        .collect::<std::collections::BTreeSet<_>>();
    let viewport = canvas.read().clone();
    let edges = snapshot.visible_graph_edges();
    let selected_id = snapshot.graph.selected_node_id.clone();
    let hovered_id = viewport.hovered.clone();

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
        edge_elements.push(render_edge(
            edge,
            source.position,
            target.position,
            &viewport,
        ));
    }

    let mut node_elements = Vec::new();
    for node in nodes
        .iter()
        .filter(|node| visible_ids.contains(node.id.as_str()))
    {
        let selected = selected_id.as_deref() == Some(node.id.as_str());
        let hovered = hovered_id.as_deref() == Some(node.id.as_str());
        let id = node.id.clone();
        let position = node.position;
        let mut node_canvas = canvas.clone();
        let mut node_explorer = explorer.clone();
        let mut click_canvas = canvas.clone();
        let mut click_explorer = explorer.clone();
        let click_id = node.id.clone();
        node_elements.push(
            rect()
                .key(("graph-node", node.id.clone()))
                .position(absolute(
                    &viewport,
                    [position[0] - NODE_RADIUS, position[1] - NODE_RADIUS],
                ))
                .width(Size::px(NODE_RADIUS * 2. * viewport.zoom))
                .height(Size::px(NODE_RADIUS * 2. * viewport.zoom))
                .background(node_color(node.kind, selected, hovered))
                .border(
                    Border::new()
                        .fill(if selected || hovered {
                            theme::color(theme::PRIMARY)
                        } else {
                            theme::color(theme::BORDER_STRONG)
                        })
                        .width(if selected || hovered { 3. } else { 1. }),
                )
                .with_corner_radius(NODE_RADIUS * viewport.zoom)
                .on_mouse_down(move |event: Event<MouseEventData>| {
                    if event.button == Some(MouseButton::Left) {
                        let pointer = point(event.global_location);
                        node_canvas
                            .write()
                            .begin_node_drag(id.clone(), pointer, position);
                        node_canvas.write().center_on(position);
                        node_explorer.write().select_graph_node(id.clone());
                        event.stop_propagation();
                    }
                })
                .on_mouse_up(move |_| {
                    click_canvas.write().center_on(position);
                    click_explorer.write().select_graph_node(click_id.clone());
                })
                .a11y_alt(format!("Select graph node {}", node.title))
                .into_element(),
        );
        node_elements.push(
            label()
                .key(("graph-node-title", node.id.clone()))
                .position(absolute(
                    &viewport,
                    [position[0] + NODE_RADIUS + 8., position[1] - 9.],
                ))
                .width(Size::px(180.))
                .height(Size::px(22.))
                .font_size(13.)
                .color(theme::color(theme::TEXT))
                .text(node.title.clone())
                .into_element(),
        );
    }

    let stats_text = format!(
        "{} nœuds · {} liens visibles",
        visible_ids.len(),
        edge_elements.len()
    );
    let stats = label()
        .a11y_alt(stats_text.clone())
        .color(theme::color(theme::MUTED))
        .text(stats_text);
    let selected_card = selected_card(&mut explorer, snapshot, &nodes);
    let viewport_label = viewport.viewport_label();
    let mut size_canvas = canvas.clone();
    let mut stage_canvas = canvas.clone();
    let mut stage_explorer = explorer.clone();
    let mut move_canvas = canvas.clone();
    let mut end_canvas = canvas.clone();
    let mut wheel_canvas = canvas.clone();
    let hit_nodes = nodes.clone();
    let click_hit_nodes = nodes.clone();

    let stage = rect()
        .expanded()
        .background(theme::color(theme::BG))
        .overflow(Overflow::Clip)
        .a11y_alt(viewport_label)
        .on_sized(move |event: Event<SizedEventData>| {
            size_canvas.write().set_size(event.area);
        })
        .on_mouse_down(move |event: Event<MouseEventData>| {
            if event.button == Some(MouseButton::Left) {
                let pointer = point(event.global_location);
                let world = stage_canvas.read().world_at(pointer);
                let hit = click_hit_nodes.iter().find(|node| {
                    let dx = world[0] - node.position[0];
                    let dy = world[1] - node.position[1];
                    dx * dx + dy * dy <= 34. * 34.
                });
                if let Some(node) = hit {
                    stage_canvas
                        .write()
                        .begin_node_drag(node.id.clone(), pointer, node.position);
                    stage_canvas.write().center_on(node.position);
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
                let world = move_canvas.read().world_at(pointer);
                let hovered = hit_nodes
                    .iter()
                    .filter_map(|node| {
                        let dx = world[0] - node.position[0];
                        let dy = world[1] - node.position[1];
                        (dx * dx + dy * dy <= 34. * 34.).then(|| node.id.clone())
                    })
                    .next();
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
                .a11y_alt("Edges")
                .children(edge_elements),
        )
        .child(
            rect()
                .width(Size::fill())
                .height(Size::fill())
                .position(Position::new_absolute().left(0.).top(0.))
                .a11y_alt("Nodes")
                .children(node_elements),
        )
        .maybe_child(selected_card)
        .into_element();

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .spacing(6.)
        .child(stats)
        .child(stage)
        .into_element()
}

fn layout_nodes(
    snapshot: &ExplorerState,
    overrides: &BTreeMap<String, [f32; 2]>,
) -> Vec<CanvasNode> {
    let Some(graph) = snapshot.graph.snapshot.as_ref() else {
        return Vec::new();
    };
    let mut groups = BTreeMap::<String, Vec<&GraphNode>>::new();
    for node in &graph.nodes {
        let folder = if !node.folder.trim().is_empty() {
            node.folder.clone()
        } else {
            node.relative_path
                .rsplit_once('/')
                .map(|(folder, _)| folder.to_string())
                .unwrap_or_else(|| "root".to_string())
        };
        groups.entry(folder).or_default().push(node);
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

    let center = [WORLD_WIDTH / 2., WORLD_HEIGHT / 2.];
    let cluster_radius = (WORLD_WIDTH.min(WORLD_HEIGHT) * 0.32).max(200.);
    let cluster_count = groups.len().max(1);
    let mut positions = BTreeMap::new();
    for (cluster_index, (_folder, group)) in groups.iter().enumerate() {
        let cluster_center = if cluster_count == 1 {
            center
        } else {
            let angle =
                -PI / 2. + cluster_index as f32 * std::f32::consts::TAU / cluster_count as f32;
            [
                center[0] + cluster_radius * angle.cos(),
                center[1] + cluster_radius * angle.sin(),
            ]
        };
        let note_radius = (54. + (group.len() as f32).sqrt() * 18.).max(80.);
        for (index, node) in group.iter().enumerate() {
            let angle = -PI / 2. + index as f32 * std::f32::consts::TAU / group.len().max(1) as f32;
            let position = node
                .position
                .as_ref()
                .map(|value| [value.x, value.y])
                .unwrap_or([
                    cluster_center[0] + note_radius * angle.cos(),
                    cluster_center[1] + note_radius * angle.sin(),
                ]);
            positions.insert(
                node.id.clone(),
                overrides.get(&node.id).copied().unwrap_or(position),
            );
        }
    }
    graph
        .nodes
        .iter()
        .filter_map(|node| {
            positions.get(&node.id).copied().map(|position| CanvasNode {
                id: node.id.clone(),
                title: node.title.clone(),
                kind: node.kind,
                position,
            })
        })
        .collect()
}

fn render_edge(
    edge: &GraphEdge,
    source: [f32; 2],
    target: [f32; 2],
    viewport: &GraphCanvasState,
) -> Element {
    let dx = target[0] - source[0];
    let dy = target[1] - source[1];
    let length = (dx * dx + dy * dy).sqrt().max(1.);
    let angle = dy.atan2(dx).to_degrees();
    let thickness = 1.5 + edge.weight.clamp(0., 3.);
    let midpoint = [(source[0] + target[0]) / 2., (source[1] + target[1]) / 2.];
    rect()
        .key(("graph-edge", edge.id.clone()))
        // Freya rotates a rect around its centre. Position the centre at the
        // edge midpoint so both endpoints remain attached to their nodes.
        .position(absolute(
            viewport,
            [midpoint[0] - length / 2., midpoint[1] - thickness / 2.],
        ))
        .width(Size::px(length * viewport.zoom))
        .height(Size::px(thickness * viewport.zoom))
        .background(edge_color(edge.edge_type))
        .with_corner_radius(3.)
        .rotation(angle)
        .a11y_alt(format!("Graph edge {} to {}", edge.source, edge.target))
        .child(label().font_size(1.).text(" "))
        .into_element()
}

fn selected_card(
    state: &mut State<ExplorerState>,
    snapshot: &ExplorerState,
    nodes: &[CanvasNode],
) -> Option<Element> {
    let selected = snapshot
        .graph
        .selected_node_id
        .as_deref()
        .and_then(|id| nodes.iter().find(|node| node.id == id))?;
    let title = selected.title.clone();
    let centered_label = if snapshot.graph.view_generation > 0 {
        format!("Graph viewport centered on {title}")
    } else {
        format!("Graph node selected {title}")
    };
    let mut state = state.clone();
    Some(
        rect()
            .position(Position::new_absolute().left(10.).top(10.))
            .width(Size::px(260.))
            .padding(Gaps::new_all(10.))
            .background(theme::color(theme::SOFT))
            .with_corner_radius(10.)
            .a11y_alt(centered_label)
            .child(
                label()
                    .font_weight(FontWeight::BOLD)
                    .text(format!("Selected: {title}")),
            )
            .child(
                label()
                    .font_size(12.)
                    .color(theme::color(theme::MUTED))
                    .text(selected.id.clone()),
            )
            .child(
                rect()
                    .height(Size::px(30.))
                    .padding(Gaps::new(0., 10., 0., 10.))
                    .center()
                    .background(theme::color(theme::SURFACE))
                    .with_corner_radius(7.)
                    .on_mouse_up(move |_| state.write().open_selected_graph_node())
                    .a11y_alt("Open selected note")
                    .child(label().text("Open note")),
            )
            .into_element(),
    )
}

fn absolute(viewport: &GraphCanvasState, position: [f32; 2]) -> Position {
    Position::new_absolute()
        .left(viewport.pan[0] + position[0] * viewport.zoom)
        .top(viewport.pan[1] + position[1] * viewport.zoom)
}

fn point(value: CursorPoint) -> [f32; 2] {
    let (x, y) = value.to_tuple();
    [x as f32, y as f32]
}

fn node_color(kind: GraphNodeKind, selected: bool, hovered: bool) -> Color {
    if selected {
        return theme::color(theme::PRIMARY);
    }
    if hovered {
        return theme::color(theme::SOFT);
    }
    match kind {
        GraphNodeKind::Folder => theme::color(theme::BORDER_STRONG),
        GraphNodeKind::Note => theme::color(theme::SURFACE),
        GraphNodeKind::Other => theme::color(theme::BG),
    }
}

fn edge_color(kind: GraphEdgeType) -> Color {
    match kind {
        GraphEdgeType::ExplicitLink => theme::color(theme::PRIMARY),
        GraphEdgeType::Semantic | GraphEdgeType::Related => theme::color(theme::BORDER_STRONG),
        GraphEdgeType::Folder
        | GraphEdgeType::Tag
        | GraphEdgeType::Lexical
        | GraphEdgeType::Other => theme::color(theme::BORDER),
    }
}
