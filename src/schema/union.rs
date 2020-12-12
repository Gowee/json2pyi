use std::collections::HashSet;

use super::arena::ArenaIndex;

#[derive(Debug, PartialEq, Eq)]
pub struct Union {
    pub name_hints: HashSet<String>,
    pub types: HashSet<ArenaIndex>,
}
