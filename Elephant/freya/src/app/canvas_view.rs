//! Functional Canvas route assembled from the real graph runtime.
//!
//! The graph source and node actions remain owned by `explorer_runtime` and
//! `graph_canvas`; this module only composes the Canvas route and keeps its
//! route label separate from the graph/search renderer.

use freya::prelude::*;

use crate::{
    canvas_contract::CanvasPosition,
    search_graph_contract::SurfaceError,
    theme,
};

use super::{explorer, graph_canvas, ShellState};

pub(super) fn workspace(
    shell: State<ShellState>,
    explorer_state: State<explorer::ExplorerState>,
    graph_canvas_state: State<graph_canvas::GraphCanvasState>,
    palette: theme::ThemePalette,
) -> Element {
    let snapshot = explorer_state.read().clone();
    let title = label()
        .font_size(24.)
        .font_weight(FontWeight::BOLD)
        .text("Semantic Canvas");
    let detail = label()
        .color(theme::token_color(palette, theme::ThemeToken::Muted))
        .text(format!(
            "{} nodes, {} links",
            snapshot.graph.snapshot.as_ref().map_or(0, |graph| graph.nodes.len()),
            snapshot.graph.snapshot.as_ref().map_or(0, |graph| graph.edges.len())
        ));
    let mut refresh_state = explorer_state;
    let refresh = rect()
        .padding(Gaps::new(6., 10., 6., 10.))
        .with_corner_radius(7.)
        .a11y_alt("Refresh Canvas graph")
        .on_press(move |_| refresh_state.write().request_graph_refresh())
        .child(label().text("Refresh"));
    let zoom = graph_canvas_state.read().zoom();
    let mut zoom_out_state = graph_canvas_state;
    let zoom_out = rect()
        .a11y_alt("Zoom out Canvas")
        .on_press(move |_| zoom_out_state.write().adjust_zoom(-0.1))
        .child(label().text("−"));
    let mut zoom_in_state = graph_canvas_state;
    let zoom_in = rect()
        .a11y_alt("Zoom in Canvas")
        .on_press(move |_| zoom_in_state.write().adjust_zoom(0.1))
        .child(label().text("+"));
    let zoom_label = label()
        .a11y_alt(format!("Canvas zoom {}%", (zoom * 100.).round() as i32))
        .text(format!("{}%", (zoom * 100.).round() as i32));
    let mut save_shell = shell;
    let save_canvas = graph_canvas_state;
    let mut save_explorer = explorer_state;
    let save = rect()
        .padding(Gaps::new(6., 10., 6., 10.))
        .with_corner_radius(7.)
        .a11y_alt("Save Canvas positions")
        .on_press(move |_| {
            let overrides = save_canvas.read().overrides().clone();
            let result = (|| {
                let mut shell = save_shell.write();
                let runtime = shell
                    .canvas
                    .as_mut()
                    .ok_or_else(|| "Canvas graph is not loaded".to_owned())?;
                for (id, position) in overrides {
                    runtime
                        .set_node_position(
                            &id,
                            CanvasPosition::new(position[0], position[1])
                                .map_err(|error| error.to_string())?,
                        )
                        .map_err(|error| error.to_string())?;
                }
                runtime.save().map_err(|error| error.to_string())
            })();
            if let Err(error) = result {
                save_explorer
                    .write()
                    .apply_graph_error(SurfaceError::Unknown(format!(
                        "Canvas positions could not be saved: {error}"
                    )));
            }
        })
        .child(label().text("Save positions"));
    let error = snapshot.graph.error.as_ref().map(|error| {
        rect()
            .a11y_alt("Canvas error")
            .child(label().text(format!("Canvas error: {error:?}")))
            .into_element()
    });
    rect()
        .width(Size::fill())
        .height(Size::fill())
        .padding(Gaps::new(12., 24., 20., 24.))
        .spacing(8.)
        .background(theme::token_color(palette, theme::ThemeToken::Bg))
        .a11y_alt("Semantic Canvas")
        .child(
            rect()
                .horizontal()
                .spacing(12.)
                .cross_align(Alignment::Center)
                .child(rect().spacing(3.).child(title).child(detail))
                .child(refresh)
                .child(zoom_out)
                .child(zoom_label)
                .child(zoom_in),
        )
        .child(save)
        .maybe_child(error)
        .child(graph_canvas::render(
            explorer_state,
            &snapshot,
            graph_canvas_state,
        ))
        .into_element()
}
