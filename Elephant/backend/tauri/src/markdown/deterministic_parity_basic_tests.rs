use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use super::muya_deterministic::deterministic_contract;

fn load_cases() -> Vec<Value> {
  let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let repo_root = manifest_dir.ancestors().nth(3).expect("repo root");
  let path = repo_root.join("agent/tauri-parity/elephant_tauri").join("parity").join("muya_deterministic_cases.json");
  let raw = fs::read_to_string(path).expect("fixtures");
  serde_json::from_str(&raw).expect("valid fixtures")
}

#[test]
fn validates_frontmatter_fixture() {
  let case = load_cases().into_iter().find(|case| case["name"] == "rich-frontmatter").expect("case");
  let contract = deterministic_contract(case["markdown"].as_str().unwrap());
  assert_eq!(contract["frontmatter"]["title"], "Demo");
  assert_eq!(contract["frontmatter"]["draft"], false);
  assert_eq!(contract["frontmatter"]["score"], 12);
  assert_eq!(contract["frontmatter"]["ratio"], 1.5);
  assert_eq!(contract["frontmatter"]["tags"][0], "alpha");
  assert_eq!(contract["frontmatter"]["author"]["name"], "Noam");
}

#[test]
fn validates_footnote_fixture() {
  let case = load_cases().into_iter().find(|case| case["name"] == "footnotes").expect("case");
  let contract = deterministic_contract(case["markdown"].as_str().unwrap());
  assert_eq!(contract["footnotes"]["definitions"].as_array().unwrap().len(), 1);
  assert_eq!(contract["footnotes"]["references"].as_array().unwrap().len(), 1);
  let html = contract["footnotes"]["html"].as_str().unwrap();
  assert!(html.contains("footnotes"));
  assert!(html.contains("Footnote text"));
}
