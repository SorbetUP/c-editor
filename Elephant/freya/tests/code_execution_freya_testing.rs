use elephant_freya::app::app_with_vault;
use freya_testing::{TestingNode, TestingRunner};
use std::{fs, path::PathBuf, thread, time::Duration};

fn accessible_nodes(runner: &TestingRunner, label: &str) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        (element.accessibility().builder.label() == Some(label)).then_some(node)
    })
}

fn click_label(runner: &mut TestingRunner, label: &str) {
    let node = accessible_nodes(runner, label)
        .into_iter()
        .min_by(|left, right| {
            left.layout()
                .area
                .size
                .area()
                .partial_cmp(&right.layout().area.size.area())
                .expect("accessible node areas must be ordered")
        })
        .unwrap_or_else(|| panic!("missing accessible label {label:?}"));
    let area = node.layout().area;
    runner.click_cursor((
        f64::from(area.origin.x + area.size.width / 2.0),
        f64::from(area.origin.y + area.size.height / 2.0),
    ));
}

#[test]
fn code_block_exposes_copy_and_executes_through_the_real_service() {
    let root = std::env::temp_dir().join(format!("elephant-freya-code-test-{}", std::process::id()));
    fs::create_dir_all(&root).expect("fixture vault");
    fs::write(
        root.join("Code.md"),
        "# Code\n\n```python\nprint('freya integration')\n```\n",
    )
    .expect("fixture note");
    let app_root: PathBuf = root.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || app_with_vault(app_root.clone()),
        (1280., 840.).into(),
        |_| (),
        1.,
    );

    click_label(&mut runner, "Code");
    runner.sync_and_update();
    assert!(!accessible_nodes(&runner, "Copy code block").is_empty());
    assert!(!accessible_nodes(&runner, "Run code block").is_empty());

    click_label(&mut runner, "Run code block");
    for _ in 0..300 {
        runner.sync_and_update();
        if !accessible_nodes(&runner, "Code execution output").is_empty() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(!accessible_nodes(&runner, "Code execution output").is_empty());
    let _ = fs::remove_dir_all(root);
}
