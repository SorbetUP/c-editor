use serde::{Deserialize, Serialize};

/// Observable source range expressed in UTF-16 code units, matching JavaScript.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceRange {
  pub start: u32,
  pub end: u32,
}

impl SourceRange {
  pub fn new(start: u32, end: u32) -> Self {
    Self { start, end }
  }

  pub fn len(self) -> u32 {
    self.end.saturating_sub(self.start)
  }

  pub fn is_empty(self) -> bool {
    self.start == self.end
  }
}
