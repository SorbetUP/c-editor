use crate::{
    capture_support::{absolute, png_dimensions, sha256_file},
    differential_scenario::ScenarioAction,
};
use freya_testing::TestingRunner;
use serde::Serialize;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Clone, Debug, Serialize)]
pub struct FrameEvidence {
    pub index: usize,
    #[serde(rename = "relativeMs")]
    pub relative_ms: u64,
    pub kind: String,
    pub path: String,
    #[serde(rename = "sourcePath")]
    pub source_path: String,
    pub bytes: u64,
    pub width: u32,
    pub height: u32,
    pub sha256: String,
    #[serde(rename = "contentOnly")]
    pub content_only: bool,
    #[serde(rename = "captureMethod")]
    pub capture_method: &'static str,
}

pub fn frame_kind(action: &ScenarioAction, index: usize) -> &'static str {
    if index == 0 {
        "before"
    } else if index + 1 == action.frames.len() {
        "after"
    } else {
        "during"
    }
}

pub fn action_index(action: &ScenarioAction) -> usize {
    match action.id.as_str() {
        "launch" => 0,
        "move-to-alpha-card" => 1,
        "open-search" => 2,
        "search-alpha" => 3,
        "close-search" => 4,
        "navigate-all-notes" => 5,
        "open-alpha-note" => 6,
        "edit-alpha-note" => 7,
        "scroll-alpha-note" => 8,
        "close-alpha-note" => 9,
        "open-create-menu" => 10,
        "move-through-create-menu" => 11,
        "close-create-menu" => 12,
        "drag-search-rail-item" => 13,
        other => panic!("unknown shared action id {other:?}"),
    }
}

pub fn capture_frame(
    runner: &mut TestingRunner,
    output: &Path,
    action: &ScenarioAction,
    index: usize,
) -> FrameEvidence {
    runner.sync_and_update();
    let relative = PathBuf::from("checkpoints")
        .join(&action.checkpoint)
        .join("frames")
        .join(format!(
            "{:02}-{}-{:02}-{}ms.png",
            action_index(action),
            action.id,
            index,
            action.frames[index]
        ));
    let absolute_path = output.join(&relative);
    fs::create_dir_all(absolute_path.parent().expect("frame parent"))
        .expect("create Freya frame directory");
    runner.render_to_file(&absolute_path);
    let source_path = absolute(&absolute_path);
    let (width, height) = png_dimensions(&source_path);
    let bytes = fs::metadata(&source_path)
        .expect("read Freya frame metadata")
        .len();
    FrameEvidence {
        index,
        relative_ms: action.frames[index],
        kind: frame_kind(action, index).into(),
        path: relative.to_string_lossy().replace('\\', "/"),
        source_path: source_path.to_string_lossy().replace('\\', "/"),
        bytes,
        width,
        height,
        sha256: sha256_file(&source_path),
        content_only: true,
        capture_method: "TestingRunner.render",
    }
}

pub fn wait_and_capture(
    runner: &mut TestingRunner,
    output: &Path,
    action: &ScenarioAction,
    frames: &mut Vec<FrameEvidence>,
    previous_ms: &mut u64,
    index: usize,
) {
    let at = action.frames[index];
    if at > *previous_ms {
        runner.poll_n(Duration::from_millis(at - *previous_ms), 1);
    }
    *previous_ms = at;
    frames.push(capture_frame(runner, output, action, index));
}

pub fn distinct_frame_count(frames: &[FrameEvidence]) -> usize {
    frames
        .iter()
        .map(|frame| frame.sha256.as_str())
        .collect::<BTreeSet<_>>()
        .len()
}
