pub mod block;
pub mod inline;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SyntaxLayer {
  Block,
  Inline,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FeatureSpec {
  pub id: &'static str,
  pub layer: SyntaxLayer,
  pub precedence: u16,
  pub fallback: bool,
}

pub fn all_features() -> impl Iterator<Item = &'static FeatureSpec> {
  block::FEATURES.iter().chain(inline::FEATURES.iter())
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeSet;

  use super::*;

  #[test]
  fn feature_ids_are_unique() {
    let features = all_features().collect::<Vec<_>>();
    let ids = features
      .iter()
      .map(|feature| feature.id)
      .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), features.len());
  }

  #[test]
  fn each_layer_has_exactly_one_fallback() {
    assert_eq!(
      block::FEATURES
        .iter()
        .filter(|feature| feature.fallback)
        .count(),
      1
    );
    assert_eq!(
      inline::FEATURES
        .iter()
        .filter(|feature| feature.fallback)
        .count(),
      1
    );
  }
}
