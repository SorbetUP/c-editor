use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use super::muya_compat::parse_muya_document;
use super::parse_markdown_document;

#[test]
fn validates_markdown_parity_fixtures() {
  let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let repo_root = manifest_dir.ancestors().nth(3).expect("repo root");
  let fixture_path = repo_root.join("agent/tauri-parity/elephant_tauri").join("parity").join("markdown_cases.json");
  let raw = fs::read_to_string(fixture_path).expect("markdown parity fixtures");
  let cases: Vec<Value> = serde_json::from_str(&raw).expect("valid markdown parity fixtures");

  for case in cases {
    let name = case["name"].as_str().unwrap_or("unnamed");
    let markdown = case["markdown"].as_str().expect("case markdown");
    let expect = &case["expect"];
    let doc = parse_markdown_document(markdown);
    let muya_doc = parse_muya_document(markdown);

    if let Some(title) = expect.get("title").and_then(Value::as_str) {
      assert_eq!(doc.frontmatter["title"], title, "case {name}");
    }
    if let Some(count) = expect.get("outlineCount").and_then(Value::as_u64) {
      assert_eq!(doc.outline.len() as u64, count, "case {name}");
    }
    if let Some(count) = expect.get("taskCount").and_then(Value::as_u64) {
      assert_eq!(doc.tasks.len() as u64, count, "case {name}");
    }
    if let Some(count) = expect.get("checkedTaskCount").and_then(Value::as_u64) {
      let checked = doc.tasks.iter().filter(|task| task.checked).count() as u64;
      assert_eq!(checked, count, "case {name}");
    }
    if let Some(count) = expect.get("linkCount").and_then(Value::as_u64) {
      assert_eq!(doc.links.len() as u64, count, "case {name}");
    }
    if let Some(count) = expect.get("codeBlockCount").and_then(Value::as_u64) {
      let code_blocks = doc.blocks.iter().filter(|block| block.kind == "code").count() as u64;
      assert_eq!(code_blocks, count, "case {name}");
    }
    if let Some(kinds) = expect.get("extraKinds").and_then(Value::as_array) {
      let extras = muya_doc["extras"].as_array().expect("extras array");
      for kind in kinds.iter().filter_map(Value::as_str) {
        assert!(extras.iter().any(|extra| extra["kind"] == kind), "case {name} missing extra kind {kind}");
      }
    }
    if let Some(needles) = expect.get("htmlContains").and_then(Value::as_array) {
      for needle in needles.iter().filter_map(Value::as_str) {
        let html = muya_doc["html"].as_str().unwrap_or(&doc.html);
        assert!(html.contains(needle), "case {name} missing html fragment {needle}");
      }
    }
  }
}
