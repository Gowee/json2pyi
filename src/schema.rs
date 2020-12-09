use indexmap::IndexMap;

use std::collections::HashSet;

use generational_arena::Arena;
pub use generational_arena::Index as ArenaIndex;
pub type TypeArena = Arena<Type>;

#[derive(Debug)]
pub struct Schema {
    pub arena: TypeArena,
    pub primitive_types: [ArenaIndex; 6],
    pub root: ArenaIndex,
}

#[derive(Debug)]
pub enum Type {
    Map(Map),
    Array(ArenaIndex),
    Union(Union),
    Int,
    Float,
    Bool,
    String,
    Null,
    Any,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Map {
    pub name: String,
    pub fields: IndexMap<String, ArenaIndex>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Union {
    pub name: String,
    pub types: HashSet<ArenaIndex>,
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

    pub fn is_union(&self) -> bool {
        self.as_union().is_some()
    }
}

impl Default for Type {
    fn default() -> Self {
        Type::Any
    }
}

impl Map {
    pub fn is_similar_to(&self, other: &Self) -> bool {
        // TODO: take value type into consideration
        let a: HashSet<_> = self.fields.iter().map(|(name, _)| name).collect();
        let b: HashSet<_> = other.fields.iter().map(|(name, _)| name).collect();
        
        let a_intsec_b = a.intersection(&b).count();
        let a_diff_b = a.difference(&b).count();
        let b_diff_a = b.difference(&a).count();
        let tversky_index = a_intsec_b as f64 / (a_intsec_b as f64 + 1.0 * a_diff_b as f64 +  1.0 * b_diff_a as f64);

        // https://en.wikipedia.org/wiki/Tversky_index
        tversky_index > 0.8
    }
}

// impl PartialEq for Type {
//     fn eq(&self, other: &Self) -> bool {
//         fn canon<'a>(
//             union: impl IntoIterator<Item = &'a Type>,
//         ) -> (
//             [bool; 6],
//             Option<&'a Map<String, Type>>,
//             Option<&'a Type>,
//         ) {
//             let mut primitive_types = [false; 6];
//             let mut map = None;
//             let mut array: Option<&Type> = None;

//             for schema in union.into_iter() {
//                 match *schema {
//                     Type::Map(ref m) => {
//                         if map.is_none() {
//                             map = Some(m);
//                         } else {
//                             panic!("Union should not have multiple Maps inside");
//                         }
//                     }
//                     Type::Array(ref a) => {
//                         if array.is_none() {
//                             array = Some(&*a);
//                         } else {
//                             panic!("Union should not have multiple Arrays inside")
//                         }
//                     }
//                     Type::Union(_) => {
//                         panic!("Union should not have Union as direct child inside")
//                     }
//                     Type::Int => primitive_types[0] = true,
//                     Type::Float => primitive_types[1] = true,
//                     Type::Bool => primitive_types[2] = true,
//                     Type::String => primitive_types[3] = true,
//                     Type::Null => primitive_types[4] = true,
//                     Type::Any => primitive_types[5] = true,
//                 }
//             }
//             (primitive_types, map, array)
//         }

//         match *self {
//             Self::Map(ref self_map) => {
//                 if let Some(other_map) = other.as_map() {
//                     self_map == other_map
//                 } else {
//                     false
//                 }
//             }
//             Self::Array(ref self_array) => {
//                 if let Some(other_array) = other.as_array() {
//                     self_array.as_ref() == other_array
//                 } else {
//                     false
//                 }
//             }
//             Self::Union(ref self_union) => {
//                 if let Some(other_union) = other.as_union() {
//                     dbg!(self);
//                     dbg!(other);
//                     canon(self_union) == canon(other_union)
//                 } else {
//                     false
//                 }
//             }
//             Type::Int => other.is_int(),
//             Type::Float => other.is_float(),
//             Type::Bool => other.is_bool(),
//             Type::String => other.is_string(),
//             Type::Null => other.is_null(),
//             Type::Any => other.is_any(),
//         }
//     }
// }

// impl Eq for Type {}

// pub trait ExpandUnion: IntoIterator<Item = Schema> {
//     fn expand_union(self) -> impl Iterator<Item = Schema>;
// }

// impl<T: IntoIterator<Item = Schema>> T {
//     pub fn expand_union() {}
// }

// impl From<Vec<Schema>> for Schema {
//     fn from(vec: Vec<Schema>) -> Schema {

//     }
// }

// macro_rules! impl_is_variant {
//     ($enum:ident, $variant:ident, $name:ident) => {
//         impl $enum {
//             pub fn is_$name(&self) -> bool {
//                 if let Self::$variant =
//             }
//         }
//     }
// }
