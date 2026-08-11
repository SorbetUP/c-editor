use freya::prelude::*;
use freya_testing::{TestingNode, TestingRunner};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::Path};

pub fn labeled_nodes(runner: &TestingRunner, label: &str) -> Vec<TestingNode> {
    runner.find_many(|node, element| {
        (element.accessibility().builder.label() == Some(label)).then_some(node)
    })
}

pub fn require_label(runner: &TestingRunner, label: &str) -> TestingNode {
    labeled_nodes(runner, label)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("Freya action target is missing: accessibility label {label:?}"))
}

pub fn require_note_card(runner: &TestingRunner, label: &str) -> TestingNode {
    labeled_nodes(runner, label)
        .into_iter()
        .find(|node| node.layout().area.origin.x > 300.0)
        .unwrap_or_else(|| {
            panic!(
                "Freya action target is missing: visible library card with accessibility label {label:?}"
            )
        })
}

pub fn click_label(runner: &mut TestingRunner, label: &str) {
    let point = require_label(runner, label).layout().area.center().to_f64();
    runner.move_cursor(point);
    runner.click_cursor(point);
    runner.sync_and_update();
}

pub fn click_note_card(runner: &mut TestingRunner, label: &str) {
    let point = require_note_card(runner, label)
        .layout()
        .area
        .center()
        .to_f64();
    runner.move_cursor(point);
    runner.click_cursor(point);
    runner.sync_and_update();
}

pub fn paragraph_text(runner: &TestingRunner) -> String {
    labeled_nodes(runner, "Paragraph")
        .into_iter()
        .filter_map(|node| Paragraph::try_downcast(node.element().as_ref()))
        .map(|paragraph| {
            paragraph
                .spans
                .iter()
                .map(|span| span.text.to_string())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn all_accessibility_labels(runner: &TestingRunner) -> Vec<String> {
    let mut labels =
        runner.find_many(|_, element| element.accessibility().builder.label().map(str::to_owned));
    labels.sort();
    labels.dedup();
    labels
}

pub fn geometry_for_labels(runner: &TestingRunner) -> BTreeMap<String, Vec<Value>> {
    let mut geometry = BTreeMap::new();
    for label in all_accessibility_labels(runner) {
        let values = labeled_nodes(runner, &label)
            .into_iter()
            .map(|node| {
                let area = node.layout().area;
                json!({
                    "x": area.origin.x,
                    "y": area.origin.y,
                    "width": area.size.width,
                    "height": area.size.height,
                })
            })
            .collect::<Vec<_>>();
        geometry.insert(label, values);
    }
    geometry
}

pub fn background_for_label(runner: &TestingRunner, label: &str) -> Fill {
    Rect::try_downcast(require_label(runner, label).element().as_ref())
        .unwrap_or_else(|| {
            panic!(
                "HARD_ISSUE pointer action: accessibility target {label:?} has no rendered Rect style"
            )
        })
        .style
        .background
}

pub fn accessibility_value(runner: &TestingRunner, label: &str) -> Option<String> {
    labeled_nodes(runner, label).into_iter().find_map(|node| {
        node.element()
            .accessibility()
            .builder
            .value()
            .map(|value| value.to_owned())
    })
}

pub fn read_rail_order(vault_root: &Path) -> Vec<String> {
    let path = vault_root.join(".elephantnote/workspace.json");
    let Ok(raw) = fs::read_to_string(path) else {
        return vec!["sidebar-toggle".into(), "search".into()];
    };
    serde_json::from_str::<Value>(&raw)
        .ok()
        .and_then(|value| {
            value
                .get("freyaShell")?
                .get("railOrder")?
                .as_array()
                .cloned()
        })
        .map(|values| {
            values
                .into_iter()
                .filter_map(|value| value.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_else(|| vec!["sidebar-toggle".into(), "search".into()])
}
