#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedLeaf<'a> {
  pub value: &'a str,
  pub consumed: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedWrapped<'a> {
  pub content: &'a str,
  pub consumed: usize,
}

pub fn parse_autolink(source: &str) -> Option<ParsedLeaf<'_>> {
  let rest = source.strip_prefix('<')?;
  let end = rest.find('>')?;
  let value = &rest[..end];
  if value.is_empty() || value.chars().any(char::is_whitespace) {
    return None;
  }
  if !(valid_scheme_link(value) || valid_email(value)) {
    return None;
  }
  Some(ParsedLeaf {
    value,
    consumed: end + 2,
  })
}

pub fn parse_inline_html(source: &str) -> Option<ParsedLeaf<'_>> {
  if let Some(comment) = source.strip_prefix("<!--") {
    let end = comment.find("-->")?;
    let consumed = 4 + end + 3;
    return Some(ParsedLeaf {
      value: &source[..consumed],
      consumed,
    });
  }

  let after_lt = source.strip_prefix('<')?;
  let first = after_lt.chars().next()?;
  if !first.is_ascii_alphabetic() {
    return None;
  }
  let open_end = after_lt.find('>')? + 1;
  let opening = &after_lt[..open_end];
  if opening.contains('\n') || opening[..opening.len() - 1].contains('<') {
    return None;
  }
  let tag_end = opening
    .char_indices()
    .take_while(|(_, character)| character.is_ascii_alphanumeric() || *character == '-')
    .last()
    .map(|(index, character)| index + character.len_utf8())?;
  let tag = &opening[..tag_end];
  let open_consumed = 1 + open_end;
  if opening.trim_end().ends_with("/>") {
    return Some(ParsedLeaf {
      value: &source[..open_consumed],
      consumed: open_consumed,
    });
  }

  let closing = format!("</{tag}>");
  if let Some(relative) = source[open_consumed..].find(&closing) {
    let consumed = open_consumed + relative + closing.len();
    return Some(ParsedLeaf {
      value: &source[..consumed],
      consumed,
    });
  }

  Some(ParsedLeaf {
    value: &source[..open_consumed],
    consumed: open_consumed,
  })
}

pub fn parse_inline_math(source: &str) -> Option<ParsedLeaf<'_>> {
  if !source.starts_with('$') || source.starts_with("$$") {
    return None;
  }
  let close = find_unescaped(source, '$', 1)?;
  if close <= 1 || source[close + 1..].starts_with('$') {
    return None;
  }
  let value = &source[1..close];
  if value.ends_with('\\') {
    return None;
  }
  Some(ParsedLeaf {
    value,
    consumed: close + 1,
  })
}

pub fn parse_superscript(source: &str) -> Option<ParsedWrapped<'_>> {
  parse_single_delimiter(source, '^')
}

pub fn parse_subscript(source: &str) -> Option<ParsedWrapped<'_>> {
  if source.starts_with("~~") {
    return None;
  }
  parse_single_delimiter(source, '~')
}

pub fn parse_footnote_reference(source: &str) -> Option<ParsedLeaf<'_>> {
  let rest = source.strip_prefix("[^")?;
  let end = rest.find(']')?;
  let label = &rest[..end];
  if label.is_empty()
    || label
      .chars()
      .any(|character| character.is_whitespace() || matches!(character, '^' | '[' | ']'))
  {
    return None;
  }
  Some(ParsedLeaf {
    value: label,
    consumed: 2 + end + 1,
  })
}

fn parse_single_delimiter(source: &str, delimiter: char) -> Option<ParsedWrapped<'_>> {
  if !source.starts_with(delimiter) {
    return None;
  }
  let start = delimiter.len_utf8();
  let close = find_unescaped(source, delimiter, start)?;
  let content = &source[start..close];
  if content.is_empty() || contains_unescaped_whitespace(content) {
    return None;
  }
  Some(ParsedWrapped {
    content,
    consumed: close + delimiter.len_utf8(),
  })
}

fn find_unescaped(source: &str, target: char, from: usize) -> Option<usize> {
  let mut escaped = false;
  for (relative, character) in source[from..].char_indices() {
    if escaped {
      escaped = false;
      continue;
    }
    if character == '\\' {
      escaped = true;
      continue;
    }
    if character == target {
      return Some(from + relative);
    }
  }
  None
}

fn contains_unescaped_whitespace(value: &str) -> bool {
  let mut escaped = false;
  for character in value.chars() {
    if escaped {
      escaped = false;
      continue;
    }
    if character == '\\' {
      escaped = true;
      continue;
    }
    if character.is_whitespace() {
      return true;
    }
  }
  false
}

fn valid_scheme_link(value: &str) -> bool {
  let Some((scheme, destination)) = value.split_once(':') else {
    return false;
  };
  if scheme.len() < 2 || scheme.len() > 32 || destination.is_empty() {
    return false;
  }
  let mut chars = scheme.chars();
  chars.next().is_some_and(|character| character.is_ascii_alphabetic())
    && chars.all(|character| character.is_ascii_alphanumeric() || matches!(character, '+' | '.' | '-'))
    && !destination.chars().any(|character| matches!(character, ' ' | '<' | '>'))
}

fn valid_email(value: &str) -> bool {
  let Some((local, domain)) = value.split_once('@') else {
    return false;
  };
  !local.is_empty()
    && !domain.is_empty()
    && domain.split('.').all(|part| {
      !part.is_empty()
        && !part.starts_with('-')
        && !part.ends_with('-')
        && part.chars().all(|character| character.is_ascii_alphanumeric() || character == '-')
    })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mirrors_tauri_autolink_rule() {
    assert_eq!(parse_autolink("<https://example.com> tail").unwrap().value, "https://example.com");
    assert_eq!(parse_autolink("<dev@example.com>").unwrap().value, "dev@example.com");
    assert!(parse_autolink("<not a link>").is_none());
  }

  #[test]
  fn mirrors_tauri_html_math_and_extension_markers() {
    assert_eq!(parse_inline_html("<kbd>Ctrl</kbd> tail").unwrap().value, "<kbd>Ctrl</kbd>");
    assert_eq!(parse_inline_math("$x+y$ tail").unwrap().value, "x+y");
    assert_eq!(parse_superscript("^2^ tail").unwrap().content, "2");
    assert_eq!(parse_subscript("~n~ tail").unwrap().content, "n");
    assert_eq!(parse_footnote_reference("[^source-1] tail").unwrap().value, "source-1");
  }
}