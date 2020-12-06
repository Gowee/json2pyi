use typed_arena::Arena;
use indexmap::IndexMap;

// use crate::mapset_impl::Map;

pub struct Map<'a> {
    name: Option<String>,
    fields: IndexMap<String, SchemaType<'a>>
}

pub struct Union<'a> {
    name: Option<String>,
    primitive_types: [bool; 6],
    map: Option<&'a Map<'a>>,
    array: Option<&'a SchemaType<'a>>,
}

#[derive(Debug)]
pub enum SchemaType<'a> {
    Map(&'a Map<'a>>),
    Array(&'a SchemaType<'a>),
    Union(Vec<&'a SchemaType<'a>>),
    Int,
    Float,
    Bool,
    String,
    Null,
    Any,
}

impl<'a> SchemaType<'a> {
    pub fn as_map(&'a self) -> Option<&'a Map<String, &'a SchemaType<'a>>> {
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

    pub fn as_array(&self) -> Option<&SchemaType> {
        match *self {
            Self::Array(schema) => Some(schema),
            _ => None,
        }
    }

    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    pub fn as_union(&'a self) -> Option<&'a [&'a SchemaType<'a>]> {
        match *self {
            Self::Union(ref schemas) => Some(schemas.as_slice()),
            _ => None,
        }
    }

    pub fn is_union(&self) -> bool {
        self.as_union().is_some()
    }
}

impl<'a> PartialEq for &'a SchemaType<'a> {
    fn eq(&self, other: &Self) -> bool {
        fn canon<'a>(
            union: impl IntoIterator<Item = &'a SchemaType<'a>>,
        ) -> (
            [bool; 6],
            Option<&'a Map<String, SchemaType<'a>>>,
            Option<&'a SchemaType<'a>>,
        ) {
            let mut primitive_types = [false; 6];
            let mut map = None;
            let mut array: Option<&SchemaType> = None;

            for schema in union.into_iter() {
                match *schema {
                    SchemaType::Map(ref m) => {
                        if map.is_none() {
                            map = Some(m);
                        } else {
                            panic!("Union should not have multiple Maps inside");
                        }
                    }
                    SchemaType::Array(ref a) => {
                        if array.is_none() {
                            array = Some(&*a);
                        } else {
                            panic!("Union should not have multiple Arrays inside")
                        }
                    }
                    SchemaType::Union(_) => {
                        panic!("Union should not have Union as direct child inside")
                    }
                    SchemaType::Int => primitive_types[0] = true,
                    SchemaType::Float => primitive_types[1] = true,
                    SchemaType::Bool => primitive_types[2] = true,
                    SchemaType::String => primitive_types[3] = true,
                    SchemaType::Null => primitive_types[4] = true,
                    SchemaType::Any => primitive_types[5] = true,
                }
            }
            (primitive_types, map, array)
        }

        match *self {
            Self::Map(ref self_map) => {
                if let Some(other_map) = other.as_map() {
                    self_map == other_map
                } else {
                    false
                }
            }
            Self::Array(ref self_array) => {
                if let Some(other_array) = other.as_array() {
                    self_array.as_ref() == other_array
                } else {
                    false
                }
            }
            Self::Union(ref self_union) => {
                if let Some(other_union) = other.as_union() {
                    dbg!(self);
                    dbg!(other);    
                    canon(self_union) == canon(other_union)
                } else {
                        false
                }
            }
            SchemaType::Int => other.is_int(),
            SchemaType::Float => other.is_float(),
            SchemaType::Bool => other.is_bool(),
            SchemaType::String => other.is_string(),
            SchemaType::Null => other.is_null(),
            SchemaType::Any => other.is_any(),
        }
    }
}

impl<'a> Eq for &'a SchemaType<'a> {}

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
