use std::collections::HashSet;

use super::{arena::ArenaIndex, name_hints::NameHints};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Union {
    pub name_hints: NameHints,
    pub types: HashSet<ArenaIndex>,
}
