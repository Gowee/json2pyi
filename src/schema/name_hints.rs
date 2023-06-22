use indexmap::IndexSet;
use itertools::Itertools;

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

/// Name hints in [`super::Type::Map`] or [`super::Type::Union`], a wrapper around `IndexSet<String>`
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NameHints(IndexSet<String>);

impl NameHints {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn into_inner(self) -> IndexSet<String> {
        self.into()
    }
}

impl fmt::Display for NameHints {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        Itertools::intersperse(self.iter().map(String::as_str), "Or")
            .try_for_each(|s| write!(fmt, "{}", s))
    }
}

impl Deref for NameHints {
    type Target = IndexSet<String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NameHints {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<IndexSet<String>> for NameHints {
    fn from(h: IndexSet<String>) -> Self {
        NameHints(h)
    }
}

impl From<NameHints> for IndexSet<String> {
    fn from(h: NameHints) -> Self {
        h.0
    }
}
