use elephant_freya::app::app_with_vault;
use freya::prelude::*;
use freya_testing::{TestingNode, TestingRunner};
use std::{
    collections::HashSet,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const NOTE_COUNT: usize = 300;
const INITIAL_RENDER_COUNT: usize = 72;
const MAX_REVEAL_PER_SCROLL: usize = 72;

struct ProfileOverride {
    root: PathBuf,
    previous: Option<OsString>,
}

impl ProfileOverride {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "elephant-freya-library-pagination-profile-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create clean pagination profile");
        let previous = std::env::var_os("ELEPHANT_FREYA_PROFILE");
        std::env::set_var("ELEPHANT_FREYA_PROFILE", &root);
        Self { root, previous }
    }
}

impl Drop for ProfileOverride {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(previous) => std::env::set_var("ELEPHANT_FREYA_PROFILE", previous),
            None => std::env::remove_var("ELEPHANT_FREYA_PROFILE"),
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct FixtureVault {
    root: PathBuf,
}

impl FixtureVault {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after the Unix epoch")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("elephant-freya-library-pagination-parity-{stamp}"));
        fs::create_dir_all(&root).expect("create pagination fixture vault");
        for index in 0..NOTE_COUNT {
            fs::write(
                root.join(format!("pagination-note-{index:03}.md")),
                format!("# Pagination Note {index:03}\n\nFixture note.\n"),
            )
            .expect("write pagination fixture note");
        }
        Self { root }
    }

    fn root(&self) -> &Path {
        &self.root
    }
}

impl Drop for FixtureVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn pagination_note_count(runner: &TestingRunner) -> usize {
    let mut labels = HashSet::new();
    let mut nodes = library_scroll(runner).children();
    while let Some(node) = nodes.pop() {
        nodes.extend(node.children());
        if let Some(label) = node.element().accessibility().builder.label() {
            if label.starts_with("Pagination Note ") {
                labels.insert(label.to_owned());
            }
        }
    }
    labels.len()
}

fn library_scroll(runner: &TestingRunner) -> TestingNode {
    runner
        .find_many(|node, element| {
            Rect::try_downcast(element)
                .filter(|rect| rect.accessibility.builder.role() == AccessibilityRole::ScrollView)
                .map(|_| node)
        })
        .into_iter()
        .max_by(|left, right| {
            left.layout()
                .area
                .size
                .area()
                .partial_cmp(&right.layout().area.size.area())
                .expect("scroll view areas must be ordered")
        })
        .expect("the library scroll view must be mounted")
}

fn scroll_library(runner: &mut TestingRunner, delta_y: f64) {
    let center = library_scroll(runner).layout().area.center().to_f64();
    runner.scroll(center, (0., delta_y));
}

#[test]
fn one_scroll_action_never_crosses_two_backend_page_boundaries() {
    let _profile = ProfileOverride::new();
    let fixture = FixtureVault::new();
    let root = fixture.root().to_path_buf();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    assert_eq!(pagination_note_count(&runner), INITIAL_RENDER_COUNT);

    scroll_library(&mut runner, -10.);
    assert_eq!(
        pagination_note_count(&runner),
        INITIAL_RENDER_COUNT,
        "scrolling far from the 720px bottom trigger must not reveal or fetch entries"
    );

    for action in 1..=2 {
        let before = pagination_note_count(&runner);
        scroll_library(&mut runner, -100_000.);
        let after = pagination_note_count(&runner);
        assert!(
            after <= before + MAX_REVEAL_PER_SCROLL,
            "scroll action {action} jumped from {before} to {after} visible notes"
        );
    }

    assert_eq!(
        pagination_note_count(&runner),
        192,
        "two bottom-reaching actions must perform one buffered reveal and one backend fetch"
    );
}
