use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use super::muya_clipboard::{backspace, clipboard_contract, paste_text, EditState};
use super::muya_interactions::{commit_composition, image_selection, table_insert_column, update_composition, CompositionState};
use super::muya_navigation::{detect_input_rule, move_cursor};

#[test]
fn validates_muya_editor_parity_fixtures() {
  let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let repo_root = manifest_dir.ancestors().nth(3).expect("repo root");
  let fixture_path = repo_root.join("agent/tauri-parity/elephant_tauri").join("parity").join("muya_editor_cases.json");
  let raw = fs::read_to_string(fixture_path).expect("muya editor parity fixtures");
  let cases: Vec<Value> = serde_json::from_str(&raw).expect("valid muya editor parity fixtures");

  for case in cases {
    let name = case["name"].as_str().unwrap_or("unnamed");
    match case["kind"].as_str().unwrap_or_default() {
      "clipboard" => {
        let markdown = case["markdown"].as_str().unwrap();
        let selection = serde_json::from_value(case["selection"].clone()).ok();
        let result = clipboard_contract(markdown, selection);
        assert_eq!(result["markdown"], case["expect"]["markdown"], "case {name}");
        for needle in case["expect"]["htmlContains"].as_array().unwrap().iter().filter_map(Value::as_str) {
          assert!(result["html"].as_str().unwrap().contains(needle), "case {name} missing {needle}");
        }
      }
      "paste" => {
        let state: EditState = serde_json::from_value(case["state"].clone()).unwrap();
        let result = paste_text(state, case["text"].as_str().unwrap());
        assert_eq!(result.markdown, case["expect"]["markdown"].as_str().unwrap(), "case {name}");
        assert_eq!(result.cursor as u64, case["expect"]["cursor"].as_u64().unwrap(), "case {name}");
        assert_eq!(!result.undo_stack.is_empty(), case["expect"]["canUndo"].as_bool().unwrap(), "case {name}");
      }
      "backspace" => {
        let state: EditState = serde_json::from_value(case["state"].clone()).unwrap();
        let result = backspace(state);
        assert_eq!(result.markdown, case["expect"]["markdown"].as_str().unwrap(), "case {name}");
        assert_eq!(result.cursor as u64, case["expect"]["cursor"].as_u64().unwrap(), "case {name}");
      }
      "moveCursor" => {
        let result = move_cursor(
          case["markdown"].as_str().unwrap(),
          case["cursor"].as_u64().unwrap() as usize,
          case["direction"].as_str().unwrap(),
          case["extend"].as_bool().unwrap_or(false),
          case["anchor"].as_u64().map(|value| value as usize),
        );
        assert_eq!(result["cursor"], case["expect"]["cursor"], "case {name}");
      }
      "inputRule" => {
        let rule = detect_input_rule(case["lineBeforeCursor"].as_str().unwrap()).expect("rule");
        assert_eq!(rule["kind"], case["expect"]["kind"], "case {name}");
        if let Some(expected_checked) = case["expect"].get("checked") {
          assert_eq!(rule["checked"], *expected_checked, "case {name}");
        }
      }
      "tableInsertColumn" => {
        let result = table_insert_column(case["markdown"].as_str().unwrap(), case["columnIndex"].as_u64().unwrap() as usize);
        assert!(result.contains(case["expect"]["contains"].as_str().unwrap()), "case {name}");
      }
      "imageSelection" => {
        let result = image_selection(case["markdown"].as_str().unwrap(), case["cursor"].as_u64().unwrap() as usize).unwrap();
        assert_eq!(result["alt"], case["expect"]["alt"], "case {name}");
        assert_eq!(result["url"], case["expect"]["url"], "case {name}");
      }
      "ime" => {
        let state: CompositionState = serde_json::from_value(case["state"].clone()).unwrap();
        let state = update_composition(state, case["text"].as_str().unwrap());
        let result = commit_composition(state);
        assert_eq!(result.markdown, case["expect"]["markdown"].as_str().unwrap(), "case {name}");
        assert_eq!(result.cursor as u64, case["expect"]["cursor"].as_u64().unwrap(), "case {name}");
        assert_eq!(result.composing, case["expect"]["composing"].as_bool().unwrap(), "case {name}");
      }
      other => panic!("unknown editor parity fixture kind {other}"),
    }
  }
}
