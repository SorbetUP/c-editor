use serde_json::{json, Map, Number, Value};

pub fn split_yaml_frontmatter(markdown: &str) -> (Value, &str) {
  let Some(rest) = markdown.strip_prefix("---\n") else { return (json!({}), markdown); };
  let Some(end) = rest.find("\n---") else { return (json!({}), markdown); };
  let raw = &rest[..end];
  let body = rest[end + 4..].trim_start_matches('\n');
  (parse_yaml_like_frontmatter(raw), body)
}

pub fn parse_yaml_like_frontmatter(raw: &str) -> Value {
  let lines = raw.lines().collect::<Vec<_>>();
  let mut map = Map::new();
  let mut i = 0usize;

  while i < lines.len() {
    let line = lines[i];
    if line.trim().is_empty() || line.trim_start().starts_with('#') || leading_spaces(line) > 0 {
      i += 1;
      continue;
    }

    let Some((key, value)) = line.split_once(':') else { i += 1; continue; };
    let key = key.trim();
    if key.is_empty() { i += 1; continue; }
    let value = value.trim();

    if value.is_empty() {
      let (nested, consumed) = parse_nested_block(&lines[i + 1..]);
      map.insert(key.to_string(), nested);
      i += consumed + 1;
    } else {
      map.insert(key.to_string(), parse_scalar_or_inline_array(value));
      i += 1;
    }
  }

  Value::Object(map)
}

fn parse_nested_block(lines: &[&str]) -> (Value, usize) {
  let mut consumed = 0usize;
  let mut array = Vec::new();
  let mut object = Map::new();
  let mut saw_array = false;
  let mut saw_object = false;

  for line in lines {
    if line.trim().is_empty() {
      consumed += 1;
      continue;
    }
    if leading_spaces(line) == 0 {
      break;
    }
    let trimmed = line.trim();
    if let Some(item) = trimmed.strip_prefix("- ") {
      saw_array = true;
      array.push(parse_scalar_or_inline_array(item.trim()));
    } else if let Some((key, value)) = trimmed.split_once(':') {
      saw_object = true;
      object.insert(key.trim().to_string(), parse_scalar_or_inline_array(value.trim()));
    }
    consumed += 1;
  }

  if saw_array && !saw_object { (Value::Array(array), consumed) }
  else if saw_object { (Value::Object(object), consumed) }
  else { (json!({}), consumed) }
}

pub fn parse_scalar_or_inline_array(value: &str) -> Value {
  let value = value.trim();
  if value.starts_with('[') && value.ends_with(']') {
    return Value::Array(value.trim_start_matches('[').trim_end_matches(']').split(',')
      .map(|item| parse_scalar(item.trim()))
      .filter(|item| !item.is_null())
      .collect());
  }
  parse_scalar(value)
}

fn parse_scalar(value: &str) -> Value {
  let value = value.trim();
  if value.is_empty() { return Value::Null; }
  if matches!(value, "null" | "~") { return Value::Null; }
  if value == "true" { return Value::Bool(true); }
  if value == "false" { return Value::Bool(false); }
  if let Ok(number) = value.parse::<i64>() { return Value::Number(Number::from(number)); }
  if let Ok(number) = value.parse::<f64>() {
    if let Some(number) = Number::from_f64(number) { return Value::Number(number); }
  }
  Value::String(unquote(value).trim_start_matches('#').to_string())
}

fn unquote(value: &str) -> String {
  let value = value.trim();
  if (value.starts_with('"') && value.ends_with('"')) || (value.starts_with('\'') && value.ends_with('\'')) {
    value[1..value.len() - 1].to_string()
  } else {
    value.to_string()
  }
}

fn leading_spaces(line: &str) -> usize {
  line.chars().take_while(|ch| *ch == ' ' || *ch == '\t').map(|ch| if ch == '\t' { 2 } else { 1 }).sum()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_scalars_inline_arrays_and_body() {
    let (frontmatter, body) = split_yaml_frontmatter("---\ntitle: \"Demo\"\ndraft: true\nscore: 12\nratio: 1.5\ntags: [one, #two]\nempty: null\n---\n# Body");
    assert_eq!(frontmatter["title"], "Demo");
    assert_eq!(frontmatter["draft"], true);
    assert_eq!(frontmatter["score"], 12);
    assert_eq!(frontmatter["ratio"], 1.5);
    assert_eq!(frontmatter["tags"][1], "two");
    assert!(frontmatter["empty"].is_null());
    assert_eq!(body, "# Body");
  }

  #[test]
  fn parses_block_arrays_and_objects() {
    let frontmatter = parse_yaml_like_frontmatter("tags:\n  - alpha\n  - beta\nauthor:\n  name: Noam\n  role: engineer");
    assert_eq!(frontmatter["tags"][0], "alpha");
    assert_eq!(frontmatter["tags"][1], "beta");
    assert_eq!(frontmatter["author"]["name"], "Noam");
    assert_eq!(frontmatter["author"]["role"], "engineer");
  }

  #[test]
  fn ignores_missing_closing_marker() {
    let (frontmatter, body) = split_yaml_frontmatter("---\ntitle: Broken\n# Body");
    assert_eq!(frontmatter, json!({}));
    assert!(body.contains("Broken"));
  }
}
