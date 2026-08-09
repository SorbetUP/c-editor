use serde::{Deserialize, Serialize};

#[derive(
  Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct NodeId(pub u64);
