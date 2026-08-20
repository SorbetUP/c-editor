use crate::model::ReferenceStyle;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedReference<'a> {
  pub label: &'a str,
  pub reference: &'a str,
  pub style: ReferenceStyle,
  pub consumed: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedBareAutoLink<'a> {
  pub text: &'a str,
  pub destination: String,
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

pub fn autolink_destination(value: &str) -> String {
  if valid_email(value) {
    format!("mailto:{value}")
  } else {
    value.to_string()
  }
}

pub fn parse_bare_autolink(source: &str) -> Option<ParsedBareAutoLink<'_>> {
  let end = source
    .char_indices()
    .find_map(|(index, character)| character.is_whitespace().then_some(index))
    .unwrap_or(source.len());
  let token = source.get(..end)?;
  if token.is_empty() {
    return None;
  }

  let destination = if valid_www_autolink(token) {
    format!("http://{token}")
  } else if valid_http_autolink(token) {
    token.to_string()
  } else if valid_email(token) {
    format!("mailto:{token}")
  } else {
    return None;
  };

  Some(ParsedBareAutoLink {
    text: token,
    destination,
    consumed: token.len(),
  })
}

pub fn parse_reference_link(source: &str) -> Option<ParsedReference<'_>> {
  parse_reference(source, false)
}

pub fn parse_reference_image(source: &str) -> Option<ParsedReference<'_>> {
  parse_reference(source, true)
}

fn parse_reference(source: &str, image: bool) -> Option<ParsedReference<'_>> {
  let offset = if image {
    source.strip_prefix("![")?;
    1
  } else {
    source.strip_prefix('[')?;
    0
  };
  let label_open = offset;
  let (label, label_close) = bracket_content(source, label_open)?;
  if label.is_empty() || label.trim().is_empty() || (!image && label.starts_with('^')) {
    return None;
  }

  let after_label = label_close + 1;
  if source[after_label..].starts_with("[]") {
    return Some(ParsedReference {
      label,
      reference: label,
      style: ReferenceStyle::Collapsed,
      consumed: after_label + 2,
    });
  }

  if source[after_label..].starts_with('[') {
    let (reference, reference_close) = bracket_content(source, after_label)?;
    if reference.trim().is_empty() {
      return None;
    }
    return Some(ParsedReference {
      label,
      reference,
      style: ReferenceStyle::Full,
      consumed: reference_close + 1,
    });
  }

  Some(ParsedReference {
    label,
    reference: label,
    style: ReferenceStyle::Shortcut,
    consumed: after_label,
  })
}

fn bracket_content(source: &str, opening: usize) -> Option<(&str, usize)> {
  if source.as_bytes().get(opening) != Some(&b'[') {
    return None;
  }
  let content_start = opening + 1;
  let mut depth = 1usize;
  let mut escaped = false;
  for (relative, character) in source[content_start..].char_indices() {
    let index = content_start + relative;
    if escaped {
      escaped = false;
      continue;
    }
    if character == '\\' {
      escaped = true;
      continue;
    }
    match character {
      '[' => depth += 1,
      ']' => {
        depth -= 1;
        if depth == 0 {
          return Some((&source[content_start..index], index));
        }
      }
      _ => {}
    }
  }
  None
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
  if local.is_empty() || domain.is_empty() || !local.chars().all(is_email_local_character) {
    return false;
  }
  let mut parts = domain.split('.');
  let Some(first) = parts.next() else {
    return false;
  };
  valid_domain_part(first) && parts.all(valid_domain_part)
}

fn valid_domain_part(part: &str) -> bool {
  !part.is_empty()
    && part.len() <= 63
    && part.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
    && part.as_bytes().last().is_some_and(u8::is_ascii_alphanumeric)
    && part
      .chars()
      .all(|character| character.is_ascii_alphanumeric() || character == '-')
}

fn is_email_local_character(character: char) -> bool {
  character.is_ascii_alphanumeric()
    || matches!(
      character,
      '.' | '!' | '#' | '$' | '%' | '&' | '\'' | '*' | '+' | '-' | '/' | '=' | '?' | '^' | '_'
        | '`' | '{' | '|' | '}' | '~'
    )
}

fn valid_www_autolink(value: &str) -> bool {
  let Some(rest) = value.strip_prefix("www.") else {
    return false;
  };
  let (authority, path) = split_path(rest);
  if path.is_some_and(|path| !path.starts_with('/')) {
    return false;
  }
  let (host, port) = split_port(authority);
  if !valid_optional_port(port) {
    return false;
  }
  let Some((name, tld)) = host.rsplit_once('.') else {
    return false;
  };
  !name.is_empty()
    && name
      .chars()
      .all(|character| character.is_ascii_lowercase() || matches!(character, '_' | '-'))
    && tld.len() >= 2
    && tld.chars().all(|character| character.is_ascii_lowercase())
}

fn valid_http_autolink(value: &str) -> bool {
  let rest = value
    .strip_prefix("https://")
    .or_else(|| value.strip_prefix("http://"));
  let Some(rest) = rest else {
    return false;
  };
  let (authority, path) = split_path(rest);
  if authority.is_empty() || path.is_some_and(|path| !path.starts_with('/')) {
    return false;
  }
  let (host, port) = split_port(authority);
  valid_optional_port(port) && valid_http_host(host)
}

fn valid_http_host(host: &str) -> bool {
  if host == "localhost" {
    return true;
  }
  if host.starts_with('[') && host.ends_with(']') {
    let inner = &host[1..host.len() - 1];
    return !inner.is_empty()
      && inner
        .chars()
        .all(|character| character.is_ascii_hexdigit() || matches!(character, '.' | ':'));
  }
  if host.chars().all(|character| character.is_ascii_digit() || character == '.') {
    return !host.is_empty();
  }
  let Some((prefix, tld)) = host.rsplit_once('.') else {
    return false;
  };
  !prefix.is_empty()
    && prefix.chars().all(|character| {
      character.is_ascii_lowercase() || character.is_ascii_digit() || matches!(character, '-' | '.' | '_' | '~')
    })
    && tld.len() >= 2
    && tld.chars().all(|character| character.is_ascii_lowercase())
}

fn split_path(value: &str) -> (&str, Option<&str>) {
  value
    .find('/')
    .map_or((value, None), |index| (&value[..index], Some(&value[index..])))
}

fn split_port(authority: &str) -> (&str, Option<&str>) {
  if authority.starts_with('[') {
    if let Some(end) = authority.find(']') {
      let host_end = end + 1;
      if authority.as_bytes().get(host_end) == Some(&b':') {
        return (&authority[..host_end], Some(&authority[host_end + 1..]));
      }
      return (authority, None);
    }
  }
  authority
    .rsplit_once(':')
    .filter(|(_, port)| port.chars().all(|character| character.is_ascii_digit()))
    .map_or((authority, None), |(host, port)| (host, Some(port)))
}

fn valid_optional_port(port: Option<&str>) -> bool {
  port.is_none_or(|port| {
    !port.is_empty() && port.len() <= 5 && port.chars().all(|character| character.is_ascii_digit())
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mirrors_active_tauri_angle_autolink_rule() {
    assert_eq!(parse_autolink("<https://example.com> tail").unwrap().value, "https://example.com");
    let email = parse_autolink("<dev-team@example.com>").unwrap();
    assert_eq!(email.value, "dev-team@example.com");
    assert_eq!(autolink_destination(email.value), "mailto:dev-team@example.com");
    assert!(parse_autolink("<not a link>").is_none());
  }

  #[test]
  fn mirrors_active_tauri_bare_autolink_rule() {
    let https = parse_bare_autolink("https://example.com/a(b) tail").unwrap();
    assert_eq!(https.text, "https://example.com/a(b)");
    assert_eq!(https.destination, https.text);

    let www = parse_bare_autolink("www.example.com tail").unwrap();
    assert_eq!(www.destination, "http://www.example.com");

    let email = parse_bare_autolink("dev-team@example tail").unwrap();
    assert_eq!(email.destination, "mailto:dev-team@example");

    assert!(parse_bare_autolink("ftp://example.com tail").is_none());
    assert!(parse_bare_autolink("www.example.com, tail").is_none());
  }

  #[test]
  fn mirrors_tauri_reference_link_shapes() {
    let full = parse_reference_link("[label][Target ID] tail").unwrap();
    assert_eq!(full.label, "label");
    assert_eq!(full.reference, "Target ID");
    assert_eq!(full.style, ReferenceStyle::Full);

    let collapsed = parse_reference_link("[label][] tail").unwrap();
    assert_eq!(collapsed.reference, "label");
    assert_eq!(collapsed.style, ReferenceStyle::Collapsed);

    let shortcut = parse_reference_image("![logo] tail").unwrap();
    assert_eq!(shortcut.label, "logo");
    assert_eq!(shortcut.style, ReferenceStyle::Shortcut);

    assert!(parse_reference_link("[^footnote]").is_none());
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
