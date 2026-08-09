use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use super::muya_deterministic::deterministic_contract;

fn parity(name: &str) -> Vec<Value> {
  let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = manifest_dir.ancestors().nth(3).unwrap();
  let raw = fs::read_to_string(root.join("agent/tauri-parity/elephant_tauri").join("parity").join(name)).unwrap();
  serde_json::from_str(&raw).unwrap()
}

fn markdown_for(name: &str) -> String {
  parity("muya_deterministic_cases.json")
    .into_iter()
    .find(|case| case["name"] == name)
    .unwrap()["markdown"]
    .as_str()
    .unwrap()
    .to_string()
}

#[test]
fn compares_frontmatter_snapshot() {
  let snapshot = parity("muya_source_snapshots.json").into_iter().find(|case| case["name"] == "rich-frontmatter").unwrap();
  let contract = deterministic_contract(&markdown_for("rich-frontmatter"));
  assert_eq!(contract["frontmatter"], snapshot["expect"]["frontmatter"]);
}

#[test]
fn compares_footnote_snapshot() {
  let snapshot = parity("muya_source_snapshots.json").into_iter().find(|case| case["name"] == "footnotes").unwrap();
  let contract = deterministic_contract(&markdown_for("footnotes"));
  assert_eq!(contract["footnotes"]["definitions"][0]["label"], snapshot["expect"]["footnotes"]["definitionLabels"][0]);
  assert_eq!(contract["footnotes"]["references"][0]["label"], snapshot["expect"]["footnotes"]["referenceLabels"][0]);
  let html = contract["footnotes"]["html"].as_str().unwrap();
  assert!(html.contains("footnotes"));
  assert!(html.contains("Footnote text"));
}
