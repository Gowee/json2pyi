use std::collections::HashSet;

use super::arena::ArenaIndex;

#[derive(Debug, PartialEq, Eq)]
pub struct Union {
    pub name: String,
    pub types: HashSet<ArenaIndex>,
}
