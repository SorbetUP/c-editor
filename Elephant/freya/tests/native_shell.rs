use elephant_freya::app::{app_with_shared_view, app_with_test_context};
use elephant_freya::vault::VaultService;
use freya::elements::{label::Label, rect::Rect};
use freya_testing::prelude::*;
use std::fs;
use std::path::PathBuf;

#[test]
fn renders_native_shell_headlessly() {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../migration/freya/baseline/fixture");
    let initial = VaultService::open(&fixture)
        .expect("open visual fixture")
        .snapshot()
        .expect("snapshot visual fixture");
    let (mut test, _) = TestingRunner::new(
        app_with_test_context,
        (1200., 800.).into(),
        |runner| {
            runner.provide_root_context(|| State::create(0usize));
            runner.provide_root_context(|| State::create(Ok::<_, String>(initial)));
        },
        1.,
    );
    test.sync_and_update();

    let artifact = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../migration/freya/baseline/freya-after-native.png");
    test.render_to_file(&artifact);
    assert!(artifact.is_file(), "Freya did not produce {artifact:?}");
}

#[test]
fn navigation_click_changes_the_visible_page() {
    let (mut test, view) = TestingRunner::new(
        app_with_shared_view,
        (1200., 800.).into(),
        |runner| runner.provide_root_context(|| State::create(0usize)),
        1.,
    );
    test.sync_and_update();

    test.click_cursor((36., 60. + 3. * 44. + 22.));
    test.sync_and_update();

    assert_eq!(*view.peek(), 3, "native pointer input must select Settings");
}

#[test]
fn create_menu_actions_write_real_vault_entries() {
    let root = unique_vault_root("ui-create-note");
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale test vault");
    }
    fs::create_dir_all(&root).expect("create test vault");
    let initial = VaultService::open(&root)
        .expect("open test vault")
        .snapshot()
        .expect("snapshot test vault");

    let (mut test, _) = TestingRunner::new(
        app_with_test_context,
        (1200., 800.).into(),
        |runner| {
            runner.provide_root_context(|| State::create(0usize));
            runner.provide_root_context(|| State::create(Ok::<_, String>(initial)));
        },
        1.,
    );
    test.sync_and_update();

    let fab_area = test
        .find(|node, element| {
            Rect::try_downcast(element)?;
            node.children().into_iter().find_map(|child| {
                let label = Label::try_downcast(child.element().as_ref())?;
                (label.text.as_ref() == "+").then(|| node.layout().visible_area())
            })
        })
        .expect("create button must be present");
    assert!(fab_area.size.width > 0.0);
    assert!(fab_area.size.height > 0.0);
    let center = (
        f64::from(fab_area.origin.x + fab_area.size.width / 2.0),
        f64::from(fab_area.origin.y + fab_area.size.height / 2.0),
    );
    test.click_cursor(center);
    test.sync_and_update();

    let note_area = test
        .find(|node, element| {
            Rect::try_downcast(element)?;
            let has_note_label = node.children().into_iter().any(|child| {
                child.children().into_iter().any(|grandchild| {
                    Label::try_downcast(grandchild.element().as_ref())
                        .is_some_and(|label| label.text.as_ref() == "Note")
                })
            });
            has_note_label.then(|| node.layout().visible_area())
        })
        .expect("Note create-menu action must be present");
    let note_center = (
        f64::from(note_area.origin.x + note_area.size.width / 2.0),
        f64::from(note_area.origin.y + note_area.size.height / 2.0),
    );
    test.click_cursor(note_center);
    test.sync_and_update();

    let created = root.join("Untitled.md");
    assert!(created.is_file(), "New note action must create {created:?}");
    assert_eq!(
        fs::read_to_string(&created).expect("read created note"),
        "# Untitled\n"
    );

    test.click_cursor(center);
    test.sync_and_update();
    let folder_area = test
        .find(|node, element| {
            Rect::try_downcast(element)?;
            let has_folder_label = node.children().into_iter().any(|child| {
                child.children().into_iter().any(|grandchild| {
                    Label::try_downcast(grandchild.element().as_ref())
                        .is_some_and(|label| label.text.as_ref() == "Folder")
                })
            });
            has_folder_label.then(|| node.layout().visible_area())
        })
        .expect("Folder create-menu action must be present");
    let folder_center = (
        f64::from(folder_area.origin.x + folder_area.size.width / 2.0),
        f64::from(folder_area.origin.y + folder_area.size.height / 2.0),
    );
    test.click_cursor(folder_center);
    test.sync_and_update();
    assert!(root.join("New Folder").is_dir());
    fs::remove_dir_all(root).expect("remove test vault");
}

#[test]
fn note_card_opens_real_markdown_content() {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../migration/freya/baseline/fixture");
    let snapshot = VaultService::open(&fixture)
        .expect("open visual fixture")
        .snapshot()
        .expect("snapshot visual fixture");
    let (mut test, _) = TestingRunner::new(
        app_with_test_context,
        (1200., 800.).into(),
        |runner| {
            runner.provide_root_context(|| State::create(0usize));
            runner.provide_root_context(|| State::create(Ok::<_, String>(snapshot)));
        },
        1.,
    );
    test.sync_and_update();

    let card_area = test
        .find(|node, element| {
            Rect::try_downcast(element)?;
            node.children()
                .into_iter()
                .any(|child| {
                    Label::try_downcast(child.element().as_ref())
                        .is_some_and(|label| label.text.as_ref() == "▤  Freya migration fixture")
                })
                .then(|| node.layout().visible_area())
        })
        .expect("markdown note card must be present");
    let card_center = (
        f64::from(card_area.origin.x + card_area.size.width / 2.0),
        f64::from(card_area.origin.y + card_area.size.height / 2.0),
    );
    test.click_cursor(card_center);
    test.sync_and_update();

    let editor_content_visible = test.find(|_, element| {
        let label = Label::try_downcast(element)?;
        label
            .text
            .as_ref()
            .contains("# Freya migration fixture")
            .then_some(())
    });
    assert!(
        editor_content_visible.is_some(),
        "opening a note card must render its real Markdown content"
    );
}

fn unique_vault_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "elephant-freya-native-shell-{name}-{}",
        std::process::id()
    ))
}
