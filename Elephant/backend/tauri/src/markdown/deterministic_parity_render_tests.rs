use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use super::muya_deterministic::deterministic_contract;

fn load_case(name: &str) -> Value {
  let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let repo_root = manifest_dir.ancestors().nth(3).expect("repo root");
  let path = repo_root.join("agent/tauri-parity/elephant_tauri").join("parity").join("muya_deterministic_cases.json");
  let raw = fs::read_to_string(path).expect("fixtures");
  let cases: Vec<Value> = serde_json::from_str(&raw).expect("valid fixtures");
  cases.into_iter().find(|case| case["name"] == name).expect("case")
}

#[test]
fn validates_math_and_diagram_fixtures() {
  let math_case = load_case("katex-like-math");
  let math = deterministic_contract(math_case["markdown"].as_str().unwrap());
  assert_eq!(math["math"]["inline"].as_array().unwrap().len(), 1);
  assert_eq!(math["math"]["block"].as_array().unwrap().len(), 1);
  let math_html = math["math"]["html"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect::<Vec<_>>().join("\n");
  assert!(math_html.contains("katex"));
  assert!(math_html.contains("katex-display"));
  assert!(math_html.contains("data-latex"));

  let diagram_case = load_case("diagram-contract");
  let diagram = deterministic_contract(diagram_case["markdown"].as_str().unwrap());
  assert_eq!(diagram["diagrams"]["items"].as_array().unwrap().len(), 1);
  let diagram_html = diagram["diagrams"]["html"][0].as_str().unwrap();
  assert!(diagram_html.contains("diagram-block"));
  assert!(diagram_html.contains("diagram-mermaid"));
  assert!(diagram_html.contains("data-diagram-language"));
}

#[test]
fn validates_nested_inline_and_table_fixtures() {
  let inline_case = load_case("nested-inline-marks");
  let inline = deterministic_contract(inline_case["markdown"].as_str().unwrap());
  let nested = inline["inlineMarks"]["nestedExamples"].as_array().unwrap();
  assert!(nested.iter().any(|item| item["kind"] == "strong_emphasis"));
  assert!(nested.iter().any(|item| item["kind"] == "strike_strong"));
  assert!(nested.iter().any(|item| item["kind"] == "link_code"));

  let table_case = load_case("table-alignment");
  let table = deterministic_contract(table_case["markdown"].as_str().unwrap());
  assert_eq!(table["tables"]["tables"][0]["alignments"][0], "left");
  assert_eq!(table["tables"]["tables"][0]["alignments"][1], "center");
  assert_eq!(table["tables"]["tables"][0]["alignments"][2], "right");
  assert_eq!(table["tables"]["tables"][0]["columns"], 3);
  assert_eq!(table["tables"]["tables"][0]["rows"], 1);
}
