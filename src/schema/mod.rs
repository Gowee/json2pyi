use std::collections::HashSet;

mod arena;
mod map;
mod name_hints;
mod union;

pub use self::{
    arena::{ArenaIndex, ITypeArena, TypeArena, Arena},
    map::Map,
    name_hints::NameHints,
    union::Union,
};

/// A schema inferred from a sample JSON
///
/// It is a wrapper around [`TypeArena`] with a additional `root` field pointing to the root type.
#[derive(Debug)]
pub struct Schema {
    pub arena: TypeArena,
    pub root: ArenaIndex,
}

#[derive(Debug, Clone)]
pub enum Type { // TODO: doc
    Map(Map),
    Array(ArenaIndex),
    Union(Union),
    Int,
    Float,
    Bool,
    String,
    Date,
    UUID,
    Null,
    Any,
}

pub struct TopdownIter<'a> {
    arena: &'a TypeArena,
    stack: Vec<ArenaIndex>,
    seen: HashSet<ArenaIndex>,
}

impl<'a> Iterator for TopdownIter<'a> {
    type Item = &'a Type;
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            arena,
            ref mut stack,
            ref mut seen,
        } = self;
        if let Some(curr) = stack.pop() {
            let r#type = arena.get(curr).unwrap();
            match *r#type {
                Type::Map(ref map) => {
                    for (_, &r#type) in map.fields.iter().rev() {
                        if !seen.contains(&r#type) {
                            stack.push(r#type);
                            seen.insert(r#type);
                        }
                    }
                }
                Type::Array(inner) => {
                    if !seen.contains(&inner) {
                        stack.push(inner);
                        seen.insert(inner);
                    }
                }
                Type::Union(ref union) => {
                    for &r#type in union.types.iter() {
                        if !seen.contains(&r#type) {
                            stack.push(r#type);
                            seen.insert(r#type);
                        }
                    }
                }
                _ => (),
            }
            Some(r#type)
        } else {
            None
        }
    }
}

impl Schema {
    /// Iterate over all types in the schema from its `root`
    pub fn iter_topdown(&self) -> TopdownIter {
        // TODO: iterate in topological order by BFS
        //       which needs a predicate fn to determine whether to flat a union/map in its level
        TopdownIter {
            arena: &self.arena,
            stack: vec![self.root],
            seen: Default::default(),
        }
    }
}

impl Type {
    pub fn into_map(self) -> Option<Map> {
        match self {
            Self::Map(map) => Some(map),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&Map> {
        match *self {
            Self::Map(ref map) => Some(map),
            _ => None,
        }
    }

    pub fn as_map_mut(&mut self) -> Option<&mut Map> {
        match *self {
            Self::Map(ref mut map) => Some(map),
            _ => None,
        }
    }

    pub fn is_map(&self) -> bool {
        self.as_map().is_some()
    }

    pub fn is_null(&self) -> bool {
        match *self {
            Self::Null => true,
            _ => false,
        }
    }
    pub fn is_bool(&self) -> bool {
        match *self {
            Self::Bool => true,
            _ => false,
        }
    }
    pub fn is_int(&self) -> bool {
        match *self {
            Self::Int => true,
            _ => false,
        }
    }
    pub fn is_float(&self) -> bool {
        match *self {
            Self::Float => true,
            _ => false,
        }
    }
    pub fn is_string(&self) -> bool {
        match *self {
            Self::String => true,
            _ => false,
        }
    }

    pub fn is_any(&self) -> bool {
        match *self {
            Self::Any => true,
            _ => false,
        }
    }

    pub fn into_array(self) -> Option<ArenaIndex> {
        match self {
            Self::Array(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<ArenaIndex> {
        match *self {
            Self::Array(r#type) => Some(r#type),
            _ => None,
        }
    }

    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    pub fn into_union(self) -> Option<Union> {
        match self {
            Self::Union(types) => Some(types),
            _ => None,
        }
    }

    pub fn as_union(&self) -> Option<&Union> {
        match *self {
            Self::Union(ref types) => Some(types),
            _ => None,
        }
    }

    pub fn as_union_mut(&mut self) -> Option<&mut Union> {
        match *self {
            Self::Union(ref mut types) => Some(types),
            _ => None,
        }
    }

    pub fn is_union(&self) -> bool {
        self.as_union().is_some()
    }
}

impl Default for Type {
    fn default() -> Self {
        Type::Any
    }
}
