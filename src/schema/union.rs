use std::{
    collections::HashSet,
    fmt::{self, Display},
};

use super::{arena::ArenaIndex, name_hints::NameHints};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Union {
    pub name_hints: NameHints,
    pub types: HashSet<ArenaIndex>,
}

impl Display for Union {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.name_hints.is_empty() {
            // NOTE: the type wrapper of map should be stored in Arena for persistent memory address.
            write!(f, "UnnammedUnion{:X}", self as *const Union as usize)
        } else {
            self.name_hints.fmt(f)
        }
    }
}
