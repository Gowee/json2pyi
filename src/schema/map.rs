use indexmap::IndexMap;

use std::{
    collections::HashSet,
    fmt::{self, Display},
};

use super::{arena::ArenaIndex, name_hints::NameHints};

/// A collection of field names and their corresponding types, with hints for its name
///
/// Generally, it is inferred from [`serde_json::JSONValue::Object`]
/// (i.e. `{ "key": "value", ... }`). It usually generated as `class` (as in Python), `struct`
/// (as in Rust) or key-value style `interface` (as in TypeScript).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Map {
    pub name_hints: NameHints, // FIX: IndexMap to ensure name generation is the same all the time
    pub fields: IndexMap<String, ArenaIndex>,
}

impl Map {
    /// Compare the structure of two `Map`s to determine if they are similar enough to be merged
    pub fn is_similar_to(&self, other: &Self) -> bool {
        // TODO: take value type into consideration
        let a: HashSet<_> = self.fields.iter().map(|(name, _)| name).collect();
        let b: HashSet<_> = other.fields.iter().map(|(name, _)| name).collect();

        let a_intsec_b = a.intersection(&b).count();
        let a_diff_b = a.difference(&b).count();
        let b_diff_a = b.difference(&a).count();
        let tversky_index =
            a_intsec_b as f64 / (a_intsec_b as f64 + 1.0 * a_diff_b as f64 + 1.0 * b_diff_a as f64);

        // https://en.wikipedia.org/wiki/Tversky_index
        tversky_index > 0.8
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.name_hints.is_empty() {
            // NOTE: the type wrapper of map should be stored in Arena for persistent memory address.
            write!(f, "UnnammedType{:X}", self as *const Map as usize)
        } else {
            self.name_hints.fmt(f)
        }
    }
}
