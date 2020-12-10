use indexmap::IndexMap;

use std::collections::HashSet;

use super::arena::{ArenaIndex};

#[derive(Debug, PartialEq, Eq)]
pub struct Map {
    pub name: String,
    pub fields: IndexMap<String, ArenaIndex>,
}

impl Map {
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
