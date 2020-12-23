use itertools::Itertools;

use std::{
    collections::HashSet,
    fmt,
    ops::{Deref, DerefMut},
};

/// Name hints in [`super::Type::Map`] or [`super::Type::Union`], a wrapper around `HashSet<String>`
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct NameHints(HashSet<String>);

impl NameHints {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn into_inner(self) -> HashSet<String> {
        self.into()
    }
}

impl fmt::Display for NameHints {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.iter()
            .map(String::as_str)
            .intersperse("Or")
            .map(|s| write!(fmt, "{}", s))
            .collect()
    }
}

impl Deref for NameHints {
    type Target = HashSet<String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NameHints {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<HashSet<String>> for NameHints {
    fn from(h: HashSet<String>) -> Self {
        NameHints(h)
    }
}

impl From<NameHints> for HashSet<String> {
    fn from(h: NameHints) -> Self {
        h.0
    }
}
