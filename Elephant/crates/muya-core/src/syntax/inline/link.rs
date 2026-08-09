#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinkKind {
  Link,
  Image,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkMatch<'a> {
  pub consumed: usize,
  pub label: &'a str,
  pub destination: &'a str,
  pub title: Option<&'a str>,
  pub kind: LinkKind,
}

pub fn parse_image(source: &str) -> Option<LinkMatch<'_>> {
  parse_with_prefix(source, "![", LinkKind::Image)
}

pub fn parse_link(source: &str) -> Option<LinkMatch<'_>> {
  parse_with_prefix(source, "[", LinkKind::Link)
}

fn parse_with_prefix<'a>(source: &'a str, prefix: &str, kind: LinkKind) -> Option<LinkMatch<'a>> {
  let rest = source.strip_prefix(prefix)?;
  let label_end = rest.find("](")?;
  let label = &rest[..label_end];
  let target_start = label_end + 2;
  let target_rest = &rest[target_start..];
  let target_end = target_rest.find(')')?;
  let target = target_rest[..target_end].trim();
  let (destination, title) = parse_target(target)?;

  Some(LinkMatch {
    consumed: prefix.len() + target_start + target_end + 1,
    label,
    destination,
    title,
    kind,
  })
}

fn parse_target(target: &str) -> Option<(&str, Option<&str>)> {
  if target.is_empty() {
    return Some(("", None));
  }

  for quote in ['"', '\''] {
    let marker = format!(" {quote}");
    if let Some(index) = target.find(&marker) {
      let title = target.get(index + 2..)?.strip_suffix(quote)?;
      return Some((target[..index].trim(), Some(title)));
    }
  }

  Some((target, None))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_links_and_images() {
    assert_eq!(
      parse_link("[OpenAI](https://openai.com \"Home\")"),
      Some(LinkMatch {
        consumed: 35,
        label: "OpenAI",
        destination: "https://openai.com",
        title: Some("Home"),
        kind: LinkKind::Link,
      })
    );
    assert_eq!(
      parse_image("![alt](image.png)"),
      Some(LinkMatch {
        consumed: 17,
        label: "alt",
        destination: "image.png",
        title: None,
        kind: LinkKind::Image,
      })
    );
  }
}
