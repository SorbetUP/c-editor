use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use super::muya_edges::edge_contract;

fn load_cases() -> Vec<Value> {
  let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = manifest_dir.ancestors().nth(3).unwrap();
  let raw = fs::read_to_string(root.join("agent/tauri-parity/elephant_tauri").join("parity").join("muya_edge_cases.json")).unwrap();
  serde_json::from_str(&raw).unwrap()
}

#[test]
fn validates_autolink_and_escape_edge_cases() {
  let cases = load_cases();
  let autolinks = cases.iter().find(|case| case["name"] == "autolinks-email-and-url").unwrap();
  let contract = edge_contract(autolinks["markdown"].as_str().unwrap());
  assert_eq!(contract["autolinks"]["items"].as_array().unwrap().len(), 2);
  assert_eq!(contract["autolinks"]["items"][0]["href"], "https://example.com");
  assert_eq!(contract["autolinks"]["items"][1]["href"], "mailto:me@example.com");

  let escapes = cases.iter().find(|case| case["name"] == "escaped-markdown-punctuation").unwrap();
  let contract = edge_contract(escapes["markdown"].as_str().unwrap());
  assert_eq!(contract["escapes"]["items"].as_array().unwrap().len(), 5);
  let literal = contract["escapes"]["literal"].as_str().unwrap();
  assert!(literal.contains("*literal*"));
  assert!(literal.contains("# not heading"));
  assert!(literal.contains("[not link]"));
}

#[test]
fn validates_heading_attrs_and_code_fence_edge_cases() {
  let cases = load_cases();
  let heading = cases.iter().find(|case| case["name"] == "heading-attributes").unwrap();
  let contract = edge_contract(heading["markdown"].as_str().unwrap());
  assert_eq!(contract["headingAttrs"]["items"].as_array().unwrap().len(), 1);
  assert_eq!(contract["headingAttrs"]["items"][0]["level"], 2);
  assert_eq!(contract["headingAttrs"]["items"][0]["attrs"]["id"], "section");
  assert_eq!(contract["headingAttrs"]["items"][0]["attrs"]["classes"][0], "wide");

  let fences = cases.iter().find(|case| case["name"] == "tilde-and-backtick-code-fences").unwrap();
  let contract = edge_contract(fences["markdown"].as_str().unwrap());
  assert_eq!(contract["codeFences"]["items"].as_array().unwrap().len(), 2);
  assert_eq!(contract["codeFences"]["items"][0]["language"], "python");
  assert_eq!(contract["codeFences"]["items"][1]["language"], "rust");
}
