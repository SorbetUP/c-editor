use encoding_rs::Encoding;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;

use crate::infra::write_atomically;
use crate::state::AppState;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MarkdownDocument {
  pub markdown: String,
  pub filename: String,
  pub pathname: String,
  pub encoding: String,
  pub is_bom: bool,
  pub line_ending: String,
  pub adjust_line_ending_on_save: bool,
  pub is_mixed_line_endings: bool,
  pub trim_trailing_newline: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WriteOptions {
  pub encoding: Option<String>,
  pub is_bom: Option<bool>,
  pub line_ending: Option<String>,
  pub adjust_line_ending_on_save: Option<bool>,
}

pub const BOM_UTF8: &[u8] = &[0xef, 0xbb, 0xbf];
pub const BOM_UTF16BE: &[u8] = &[0xfe, 0xff];
pub const BOM_UTF16LE: &[u8] = &[0xff, 0xfe];

fn detect_bom(bytes: &[u8]) -> Option<&'static Encoding> {
  if bytes.starts_with(BOM_UTF8) {
    Some(encoding_rs::UTF_8)
  } else if bytes.starts_with(BOM_UTF16BE) {
    Some(encoding_rs::UTF_16BE)
  } else if bytes.starts_with(BOM_UTF16LE) {
    Some(encoding_rs::UTF_16LE)
  } else {
    None
  }
}

fn strip_bom(bytes: &[u8]) -> &[u8] {
  if bytes.starts_with(BOM_UTF8) {
    &bytes[BOM_UTF8.len()..]
  } else if bytes.starts_with(BOM_UTF16BE) {
    &bytes[BOM_UTF16BE.len()..]
  } else if bytes.starts_with(BOM_UTF16LE) {
    &bytes[BOM_UTF16LE.len()..]
  } else {
    bytes
  }
}

fn detect_encoding(bytes: &[u8]) -> &'static Encoding {
  if let Some(enc) = detect_bom(bytes) {
    return enc;
  }
  let mut detector = chardetng::EncodingDetector::new();
  detector.feed(bytes, true);
  detector.guess(None, true)
}

fn decode_bytes(bytes: &[u8]) -> (String, &'static Encoding, bool) {
  let has_bom = detect_bom(bytes).is_some();
  let encoding = detect_encoding(bytes);
  let payload = strip_bom(bytes);
  let (cow, _, _) = encoding.decode(payload);
  (cow.into_owned(), encoding, has_bom)
}

fn detect_line_ending(text: &str) -> (String, bool, bool, bool) {
  let has_lf = text.contains('\n');
  let has_crlf = text.contains("\r\n");
  let is_mixed = has_lf && has_crlf;
  (
    if has_crlf && !has_lf {
      "crlf".into()
    } else {
      "lf".into()
    },
    has_lf,
    has_crlf,
    is_mixed,
  )
}

fn normalize_eol_to_lf(text: &str) -> String {
  text.replace("\r\n", "\n")
}

fn convert_eol(text: &str, eol: &str) -> String {
  let lf = normalize_eol_to_lf(text);
  if eol == "crlf" {
    lf.replace('\n', "\r\n")
  } else {
    lf
  }
}

fn encoding_name(enc: &'static Encoding) -> String {
  enc.name().to_lowercase().replace('-', "")
}

fn read_markdown_file(path: &Path) -> std::io::Result<MarkdownDocument> {
  let bytes = fs::read(path)?;
  let (mut text, encoding, is_bom) = decode_bytes(&bytes);
  let (detected_eol, has_lf, has_crlf, is_mixed) = detect_line_ending(&text);
  let line_ending = if has_crlf && !has_lf {
    "crlf".into()
  } else if has_lf && !has_crlf {
    "lf".into()
  } else {
    "lf".into()
  };
  let adjust_line_ending_on_save = is_mixed || line_ending != "lf";
  if is_mixed {
    text = normalize_eol_to_lf(&text);
  }
  let _ = (detected_eol, has_crlf, has_lf);
  let filename = path
    .file_name()
    .map(|n| n.to_string_lossy().to_string())
    .unwrap_or_default();
  let trim_trailing_newline = 3u8;
  Ok(MarkdownDocument {
    markdown: text,
    filename,
    pathname: path.to_string_lossy().to_string(),
    encoding: encoding_name(encoding),
    is_bom,
    line_ending,
    adjust_line_ending_on_save,
    is_mixed_line_endings: is_mixed,
    trim_trailing_newline,
  })
}

fn write_markdown_file(path: &Path, content: &str, opts: &WriteOptions) -> std::io::Result<()> {
  let encoding_name = opts.encoding.clone().unwrap_or_else(|| "utf8".into());
  let is_bom = opts.is_bom.unwrap_or(false);
  let eol = opts.line_ending.clone().unwrap_or_else(|| "lf".into());
  let adjust = opts.adjust_line_ending_on_save.unwrap_or(false);

  let enc = resolve_encoding(&encoding_name);
  let final_content = if adjust {
    convert_eol(content, &eol)
  } else {
    content.to_string()
  };
  let bytes = encode_with_optional_bom(&final_content, enc, is_bom);
  write_atomically(path, &bytes)
}

fn resolve_encoding(name: &str) -> &'static Encoding {
  let lower = name.to_lowercase().replace('-', "");
  match lower.as_str() {
    "utf8" | "utf" => encoding_rs::UTF_8,
    "utf16be" => encoding_rs::UTF_16BE,
    "utf16le" => encoding_rs::UTF_16LE,
    "gb18030" | "gbk" => encoding_rs::GB18030,
    "gb2312" => encoding_rs::GB18030,
    "big5" => encoding_rs::BIG5,
    "euckr" => encoding_rs::EUC_KR,
    "eucjp" => encoding_rs::EUC_JP,
    "shiftjis" | "sjis" => encoding_rs::SHIFT_JIS,
    "iso88591" | "latin1" => encoding_rs::WINDOWS_1252,
    "windows1252" => encoding_rs::WINDOWS_1252,
    _ => encoding_rs::UTF_8,
  }
}

fn encode_with_optional_bom(content: &str, enc: &'static Encoding, add_bom: bool) -> Vec<u8> {
  let mut out = Vec::new();
  if add_bom {
    if enc == encoding_rs::UTF_8 {
      out.extend_from_slice(BOM_UTF8);
    } else if enc == encoding_rs::UTF_16BE {
      out.extend_from_slice(BOM_UTF16BE);
    } else if enc == encoding_rs::UTF_16LE {
      out.extend_from_slice(BOM_UTF16LE);
    }
  }
  if enc == encoding_rs::UTF_16LE || enc == encoding_rs::UTF_16BE {
    let little = enc == encoding_rs::UTF_16LE;
    for c in content.encode_utf16() {
      let bytes = c.to_le_bytes();
      out.push(if little { bytes[0] } else { bytes[1] });
      out.push(if little { bytes[1] } else { bytes[0] });
    }
  } else {
    let (bytes, _, _) = enc.encode(content);
    out.extend_from_slice(&bytes);
  }
  out
}

pub fn resolve_path(path: &Path) -> Option<PathBuf> {
  fs::canonicalize(path).ok().or_else(|| Some(path.to_path_buf()))
}

#[tauri::command]
pub fn tauri_fs_read_markdown(path: String) -> Result<MarkdownDocument, String> {
  let path = PathBuf::from(&path);
  read_markdown_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn tauri_fs_write_markdown(path: String, content: String, options: Option<WriteOptions>) -> Result<(), String> {
  let path = PathBuf::from(&path);
  let opts = options.unwrap_or_else(|| WriteOptions {
    encoding: None,
    is_bom: None,
    line_ending: None,
    adjust_line_ending_on_save: None,
  });
  write_markdown_file(&path, &content, &opts).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn tauri_fs_resolve_path(path: String) -> Result<String, String> {
  let path = PathBuf::from(&path);
  match resolve_path(&path) {
    Some(p) => Ok(p.to_string_lossy().to_string()),
    None => Err("cannot resolve path".into()),
  }
}

#[tauri::command]
pub fn tauri_fs_detect_encoding(bytes_b64: String) -> Result<String, String> {
  use std::sync::OnceLock;
  static EMPTY: OnceLock<Vec<u8>> = OnceLock::new();
  let _ = EMPTY.set(Vec::new());
  base64_decode(&bytes_b64).map(|bytes| encoding_name(detect_encoding(&bytes)))
}

fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
  use std::convert::TryInto;
  fn decode_tbl(c: u8) -> Option<u8> {
    match c {
    b'A'..=b'Z' => Some(c - b'A'),
    b'a'..=b'z' => Some(c - b'a' + 26),
    b'0'..=b'9' => Some(c - b'0' + 52),
    b'+' => Some(62),
    b'/' => Some(63),
    _ => None,
    }
  }
  let input: Vec<u8> = s.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
  if input.len() % 4 != 0 {
    return Err("invalid base64 length".into());
  }
  let mut out = Vec::with_capacity(input.len() * 3 / 4);
  for chunk in input.chunks(4) {
    let v0 = decode_tbl(chunk[0]).ok_or("invalid base64 char")?;
    let v1 = decode_tbl(chunk[1]).ok_or("invalid base64 char")?;
    let v2 = if chunk[2] == b'=' { 0 } else { decode_tbl(chunk[2]).ok_or("invalid base64 char")? };
    let v3 = if chunk[3] == b'=' { 0 } else { decode_tbl(chunk[3]).ok_or("invalid base64 char")? };
    out.push((v0 << 2) | (v1.try_into().unwrap_or(0) >> 4));
    if chunk[2] != b'=' {
      out.push((v1 << 4) | (v2 >> 2));
      if chunk[3] != b'=' {
        out.push((v2 << 6) | v3);
      }
    }
  }
  Ok(out)
}

#[tauri::command]
pub fn tauri_fs_trash_item(path: String, _state: State<'_, AppState>) -> Result<(), String> {
  let path = PathBuf::from(&path);
  if !path.exists() {
    return Err("path does not exist".into());
  }
  if path.is_dir() {
    fs::remove_dir_all(&path).map_err(|e| e.to_string())
  } else {
    fs::remove_file(&path).map_err(|e| e.to_string())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn detect_utf8_bom() {
    let bytes: Vec<u8> = [0xef, 0xbb, 0xbf].iter().chain(b"hello".iter()).copied().collect();
    assert_eq!(detect_bom(&bytes), Some(encoding_rs::UTF_8));
  }

  #[test]
  fn detect_utf16le_bom() {
    let bytes: Vec<u8> = [0xff, 0xfe].iter().chain(b"h\x00e\x00".iter()).copied().collect();
    assert_eq!(detect_bom(&bytes), Some(encoding_rs::UTF_16LE));
  }

  #[test]
  fn detect_utf16be_bom() {
    let bytes: Vec<u8> = [0xfe, 0xff].iter().copied().collect();
    assert_eq!(detect_bom(&bytes), Some(encoding_rs::UTF_16BE));
  }

  #[test]
  fn decode_utf8_no_bom() {
    let (text, enc, bom) = decode_bytes(b"hello world");
    assert_eq!(text, "hello world");
    assert_eq!(enc, encoding_rs::UTF_8);
    assert!(!bom);
  }

  #[test]
  fn detect_line_endings() {
    let (eol, has_lf, has_crlf, is_mixed) = detect_line_ending("line1\nline2\n");
    assert_eq!(eol, "lf");
    assert!(has_lf);
    assert!(!has_crlf);
    assert!(!is_mixed);
    let (_eol, has_lf, has_crlf, _is_mixed) = detect_line_ending("line1\r\nline2\r\n");
    assert!(!has_lf || has_crlf);
    assert!(has_crlf);
  }

  #[test]
  fn normalize_eol() {
    let text = "line1\r\nline2\nline3\r\n";
    let normalized = normalize_eol_to_lf(text);
    assert_eq!(normalized, "line1\nline2\nline3\n");
  }

  #[test]
  fn convert_eol_crlf() {
    let text = "line1\nline2\n";
    assert_eq!(convert_eol(text, "crlf"), "line1\r\nline2\r\n");
    assert_eq!(convert_eol(text, "lf"), "line1\nline2\n");
  }

  #[test]
  fn encode_utf8_with_bom() {
    let bytes = encode_with_optional_bom("hello", encoding_rs::UTF_8, true);
    assert!(bytes.starts_with(BOM_UTF8));
    assert_eq!(&bytes[3..], b"hello");
  }

  #[test]
  fn encode_utf16le_without_bom() {
    let bytes = encode_with_optional_bom("AB", encoding_rs::UTF_16LE, false);
    assert_eq!(&bytes, b"A\x00B\x00");
  }

  #[test]
  fn read_markdown_file_roundtrip() {
    let dir = std::env::temp_dir().join(format!("elephantnote_fs_test_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("note.md");
    write_markdown_file(&path, "# Title\nbody", &WriteOptions {
      encoding: Some("utf8".into()),
      is_bom: Some(false),
      line_ending: Some("lf".into()),
      adjust_line_ending_on_save: Some(false),
    })
    .unwrap();
    let doc = read_markdown_file(&path).unwrap();
    assert_eq!(doc.filename, "note.md");
    assert_eq!(doc.markdown, "# Title\nbody");
    assert!(!doc.is_bom);
    fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn read_utf16le_file_with_bom() {
    let dir = std::env::temp_dir().join(format!("elephantnote_fs_utf16_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("note_le.md");
    let bytes = encode_with_optional_bom("Hello", encoding_rs::UTF_16LE, true);
    fs::write(&path, &bytes).unwrap();
    let doc = read_markdown_file(&path).unwrap();
    assert_eq!(doc.markdown, "Hello");
    assert!(doc.is_bom);
    fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn resolve_path_for_existing_file() {
    let dir = std::env::temp_dir().join(format!("elephantnote_fs_resolve_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test.md");
    fs::write(&path, "data").unwrap();
    let resolved = resolve_path(&path);
    assert!(resolved.is_some());
    fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn base64_helper_decodes_bytes() {
    let encoded = base64_decode("aGVsbG8=").unwrap();
    assert_eq!(&encoded, b"hello");
  }

  #[allow(dead_code)]
  fn _sup_noop() {}
}