use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use super::muya_deterministic::deterministic_contract;

fn load(name: &str) -> Vec<Value> {
  let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = manifest_dir.ancestors().nth(3).unwrap();
  let raw = fs::read_to_string(root.join("agent/tauri-parity/elephant_tauri").join("parity").join(name)).unwrap();
  serde_json::from_str(&raw).unwrap()
}

fn md(name: &str) -> String {
  load("muya_deterministic_cases.json")
    .into_iter()
    .find(|case| case["name"] == name)
    .unwrap()["markdown"]
    .as_str()
    .unwrap()
    .to_string()
}

#[test]
fn compares_math_and_diagram_snapshots() {
  let math = deterministic_contract(&md("katex-like-math"));
  assert_eq!(math["math"]["inline"].as_array().unwrap().len(), 1);
  assert_eq!(math["math"]["block"].as_array().unwrap().len(), 1);
  let math_html = math["math"]["html"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect::<Vec<_>>().join(" ");
  assert!(math_html.contains("math-inline"));
  assert!(math_html.contains("katex"));
  assert!(math_html.contains("katex-display"));

  let diagram = deterministic_contract(&md("diagram-contract"));
  assert_eq!(diagram["diagrams"]["items"].as_array().unwrap().len(), 1);
  assert_eq!(diagram["diagrams"]["items"][0]["attrs"]["language"], "mermaid");
  let diagram_html = diagram["diagrams"]["html"][0].as_str().unwrap();
  assert!(diagram_html.contains("diagram-block"));
  assert!(diagram_html.contains("diagram-mermaid"));
}

#[test]
fn compares_inline_and_table_snapshots() {
  let inline = deterministic_contract(&md("nested-inline-marks"));
  let nested = inline["inlineMarks"]["nestedExamples"].as_array().unwrap();
  assert!(nested.iter().any(|item| item["kind"] == "strong_emphasis"));
  assert!(nested.iter().any(|item| item["kind"] == "strike_strong"));
  assert!(nested.iter().any(|item| item["kind"] == "link_code"));

  let table = deterministic_contract(&md("table-alignment"));
  assert_eq!(table["tables"]["tables"][0]["alignments"][0], "left");
  assert_eq!(table["tables"]["tables"][0]["alignments"][1], "center");
  assert_eq!(table["tables"]["tables"][0]["alignments"][2], "right");
  assert_eq!(table["tables"]["tables"][0]["columns"], 3);
  assert_eq!(table["tables"]["tables"][0]["rows"], 1);
}
