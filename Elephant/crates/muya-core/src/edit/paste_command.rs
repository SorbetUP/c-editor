use crate::model::Document;
use crate::selection::Selection;

use super::{
  paste::build_paste_markdown, paste_nested::build_nested_paste,
  paste_nested_structured::build_nested_structured_paste, EditError, Transaction,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PasteCommand {
  markdown: String,
}

impl PasteCommand {
  pub fn new(markdown: impl Into<String>) -> Self {
    Self {
      markdown: markdown.into(),
    }
  }

  pub fn build(&self, document: &Document, selection: Selection) -> Result<Transaction, EditError> {
    if let Some(transaction) = build_nested_structured_paste(document, selection, &self.markdown)? {
      return Ok(transaction);
    }
    if let Some(transaction) = build_nested_paste(document, selection, &self.markdown)? {
      return Ok(transaction);
    }
    build_paste_markdown(document, selection, &self.markdown)
  }
}
