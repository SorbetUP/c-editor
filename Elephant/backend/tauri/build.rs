use std::env;
use std::fs;
use std::path::PathBuf;

fn push_test(out: &mut String, name: &str, body: &str) {
  out.push_str("#[test]\nfn ");
  out.push_str(name);
  out.push_str("() {\n");
  out.push_str(body);
  out.push_str("\n}\n\n");
}

fn generate_tauri_parity_tests() {
  let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is required"));
  let mut out = String::new();

  out.push_str("use crate::markdown_engine::{excerpt, heading_title, parse_markdown, parse_tags, render_note};\n");
  out.push_str("use crate::note_domain::{create_note_markdown, note_filename_from_title, rename_note_markdown};\n");
  out.push_str("use crate::path_utils::{clean_relative_path, join_path, with_markdown_extension};\n");
  out.push_str("use crate::search_logic::{normalize_query, score_text};\n");
  out.push_str("use crate::vault::types::{active_vault, next_vault_id, slug_id, VaultConfig, VaultDescriptor};\n");
  out.push_str("use crate::vault_layout::{is_hidden_vault_path, is_visible_vault_path, required_hidden_dirs, HIDDEN_ROOT};\n\n");
  out.push_str("fn vault_descriptor(id: &str, name: &str, path: &str) -> VaultDescriptor { VaultDescriptor { id: id.to_string(), name: name.to_string(), path: path.to_string(), icon: String::new(), last_opened_at: String::from(\"0\") } }\n\n");

  for index in 0..90 {
    push_test(&mut out, &format!("generated_slug_case_{index:03}"), &format!("assert_eq!(slug_id(\"Vault {index} Project\"), \"vault-{index}-project\");"));
  }

  for index in 0..80 {
    push_test(&mut out, &format!("generated_markdown_extension_case_{index:03}"), &format!("assert_eq!(with_markdown_extension(\"Note {index}\"), \"Note {index}.md\"); assert_eq!(with_markdown_extension(\"Already {index}.md\"), \"Already {index}.md\");"));
  }

  for index in 0..90 {
    push_test(&mut out, &format!("generated_clean_path_case_{index:03}"), &format!("assert_eq!(clean_relative_path(\"folder//sub/./note-{index}.md\"), \"folder/sub/note-{index}.md\"); let joined = join_path(\"/vault\", \"folder/note-{index}.md\"); assert!(joined.ends_with(\"folder/note-{index}.md\"));"));
  }

  for index in 0..70 {
    push_test(&mut out, &format!("generated_hidden_path_case_{index:03}"), &format!("assert!(is_hidden_vault_path(\".elephantnote/config/file-{index}.json\")); assert!(!is_visible_vault_path(\".elephantnote/config/file-{index}.json\")); assert!(!is_hidden_vault_path(\"folder/note-{index}.md\")); assert!(is_visible_vault_path(\"folder/note-{index}.md\"));"));
  }

  for index in 0..80 {
    push_test(&mut out, &format!("generated_note_filename_case_{index:03}"), &format!("assert_eq!(note_filename_from_title(\"Title {index}/Slash\"), \"Title {index}-Slash.md\"); assert_eq!(note_filename_from_title(\"\"), \"Untitled.md\");"));
  }

  for index in 0..70 {
    push_test(&mut out, &format!("generated_search_case_{index:03}"), &format!("assert_eq!(normalize_query(\"  Alpha   {index}  \"), \"alpha {index}\"); assert!(score_text(\"alpha {index}\", \"Alpha {index} title\", \"body\") >= 10); assert!(score_text(\"alpha {index}\", \"title\", \"alpha {index} body\") >= 3); assert_eq!(score_text(\"missing {index}\", \"title\", \"body\"), 0);"));
  }

  for index in 0..80 {
    push_test(&mut out, &format!("generated_markdown_roundtrip_case_{index:03}"), &format!("let markdown = render_note(\"Title {index}\", \"note\", &[String::from(\"tag-{index}\")], \"# Title {index}\\n\\nBody {index}\"); let parsed = parse_markdown(&markdown, \"fallback.md\"); assert_eq!(parsed.title, \"Title {index}\"); assert_eq!(parsed.note_type, \"note\"); assert_eq!(parsed.tags, vec![String::from(\"tag-{index}\")]); assert!(parsed.body.contains(\"Body {index}\")); assert!(excerpt(&parsed.body, 2).contains(\"Title {index}\"));"));
  }

  for index in 0..60 {
    push_test(&mut out, &format!("generated_rename_case_{index:03}"), &format!("let markdown = create_note_markdown(\"Old {index}\", &[String::from(\"tag\")]); let renamed = rename_note_markdown(&markdown, \"New {index}\", \"fallback.md\"); let parsed = parse_markdown(&renamed, \"fallback.md\"); assert_eq!(parsed.title, \"New {index}\"); assert!(renamed.contains(\"title: \\\"New {index}\\\"\"));"));
  }

  for index in 0..70 {
    push_test(&mut out, &format!("generated_tag_parse_case_{index:03}"), &format!(r#"let tags = parse_tags(" [alpha-{index}, #beta-{index}, gamma-{index}] "); assert_eq!(tags, vec![String::from("alpha-{index}"), String::from("beta-{index}"), String::from("gamma-{index}")]);"#));
  }

  for index in 0..70 {
    push_test(&mut out, &format!("generated_heading_excerpt_case_{index:03}"), &format!("let body = \"# Heading {index}\\n\\nFirst line {index}\\nSecond line {index}\"; assert_eq!(heading_title(body).unwrap(), \"Heading {index}\"); assert_eq!(excerpt(body, 2), \"Heading {index} First line {index}\");"));
  }

  for index in 0..60 {
    push_test(&mut out, &format!("generated_vault_collision_case_{index:03}"), &format!("let existing = vec![vault_descriptor(\"team-{index}\", \"Team\", \"/vault/team\"), vault_descriptor(\"team-{index}-2\", \"Team 2\", \"/vault/team2\")]; assert_eq!(next_vault_id(&existing, \"Team {index}\"), \"team-{index}-3\");"));
  }

  for index in 0..60 {
    push_test(&mut out, &format!("generated_active_vault_none_case_{index:03}"), &format!("let config = VaultConfig {{ vaults: vec![vault_descriptor(\"a-{index}\", \"A\", \"/vault/a\")], active_vault_id: Some(String::from(\"missing-{index}\")) }}; assert!(active_vault(&config).is_none()); let empty = VaultConfig {{ vaults: vec![], active_vault_id: None }}; assert!(active_vault(&empty).is_none());"));
  }

  for index in 0..60 {
    push_test(&mut out, &format!("generated_case_whitespace_query_case_{index:03}"), &format!("assert_eq!(normalize_query(\"\\tMixed   CASE {index}\\n\"), \"mixed case {index}\"); assert!(score_text(\"mixed case {index}\", \"MIXED CASE {index}\", \"\") > 0);"));
  }

  push_test(&mut out, "generated_required_hidden_dirs_contract", "let dirs = required_hidden_dirs(); assert!(dirs.len() >= 7); assert!(dirs.contains(&\"config\")); assert!(!dirs.contains(&\"wiki\")); assert!(dirs.contains(&\"sync\"));");
  push_test(&mut out, "generated_active_vault_contract", "let config = VaultConfig { vaults: vec![vault_descriptor(\"main\", \"Main\", \"/vault/main\")], active_vault_id: Some(String::from(\"main\")) }; let active = active_vault(&config).unwrap(); assert_eq!(active.name, \"Main\"); assert_eq!(active.path, \"/vault/main\");");
  push_test(&mut out, "generated_next_vault_id_contract", "let existing = vec![vault_descriptor(\"work\", \"Work\", \"/vault/work\"), vault_descriptor(\"work-2\", \"Work 2\", \"/vault/work2\"), vault_descriptor(\"work-3\", \"Work 3\", \"/vault/work3\")]; assert_eq!(next_vault_id(&existing, \"Work\"), \"work-4\");");
  push_test(&mut out, "generated_hidden_root_constant_contract", "assert_eq!(HIDDEN_ROOT, \".elephantnote\");");

  fs::write(out_dir.join("generated_tauri_parity_tests.rs"), out).expect("write generated Tauri parity tests");
}

fn main() {
  generate_tauri_parity_tests();
  tauri_build::build()
}
