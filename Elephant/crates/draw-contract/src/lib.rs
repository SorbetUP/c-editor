mod arrow;
mod autoshape;
mod autoshape_convert;
mod binding;
mod bucket_fill;
mod bucket_fill_action;
mod clipboard;
mod crop;
mod current_style;
mod delete;
mod document;
mod duplicate;
mod factory;
mod fractional_index;
mod frame;
mod geometry;
mod history;
mod laser;
mod library;
mod normalize;
mod rough;
mod scene_ops;
mod search;
mod selection;
mod snap;
mod svg;
mod svg_runtime;
mod text;
mod tool;
mod tool_type;
mod transform;

pub use arrow::{Arrowhead, ArrowheadPrimitive};
pub use autoshape::{recognize_shape, recognized_arrow_endpoint, RecognizedShape, ShapeRecognition};
pub use autoshape_convert::convert_autoshape;
pub use binding::ArrowEndpoint;
pub use bucket_fill::{
    compute_bucket_fill, BucketFillFailureReason, BucketFillInsertion, BucketFillOptions,
    BucketFillPlacement, BucketFillResult,
};
pub use bucket_fill_action::{
    apply_bucket_fill, is_bucket_fill_compatible, BucketFillMutation,
    DEFAULT_BUCKET_FILL_BACKGROUND,
};
pub use clipboard::SceneFragment;
pub use crop::{crop_source_rect, ImageCrop, MINIMAL_CROP_SIZE};
pub use current_style::DrawingCurrentStyle;
pub use delete::DeleteSelectionOutcome;
pub use document::{DrawingElement, DrawingScene, SceneError};
pub use duplicate::DuplicateSelectionOutcome;
pub use factory::create_element;
pub use fractional_index::{
    generate_key_between, generate_n_keys_between, validate_order_key, BASE_62_DIGITS,
};
pub use geometry::{distance_to_segment, rgba, rotate_point, Viewport, MAX_ZOOM, MIN_ZOOM};
pub use history::HistoryState;
pub use laser::{
    ease_out as laser_ease_out, LaserPoint, LaserSample, LaserTrail, LaserTrails,
    DEFAULT_LASER_COLOR, LASER_DECAY_LENGTH, LASER_DECAY_TIME_MS, LASER_SIMPLIFY,
    LASER_SIZE, LASER_STREAMLINE,
};
pub use library::{LibraryFile, LibraryItem, LibraryItemStatus};
pub use rough::{rough_shape_paths, RoughPath};
pub use search::{SceneSearchMatch, SearchField};
pub use selection::{SelectionMode, SelectionSet};
pub use snap::{SnapAxis, SnapGuide, SnapResult};
pub use svg_runtime::{image_data_url, render_scene_svg, SvgRenderOptions};
pub use text::{font_family_css, layout_text, TextLayout, TextLineLayout};
pub use tool::DrawingTool;
pub use tool_type::{ToolShortcut, ToolType};
pub use transform::TransformOutcome;
