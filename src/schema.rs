use crate::mapset_impl::Map;

#[derive(Debug)]
pub enum Schema {
    Map(Map<String, Schema>),
    Array(Box<Schema>),
    Union(Vec<Schema>),
    Int,   // TODO: range
    Float, // TODO: range
    Bool,
    String,
    Null,
    Any,
}

impl Schema {
    pub fn as_map(&self) -> Option<&Map<String, Schema>> {
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

    pub fn as_array(&self) -> Option<&Schema> {
        match *self {
            Self::Array(ref schema) => Some(schema.as_ref()),
            _ => None,
        }
    }

    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    pub fn as_union(&self) -> Option<&[Schema]> {
        match *self {
            Self::Union(ref schemas) => Some(schemas.as_slice()),
            _ => None,
        }
    }

    pub fn is_union(&self) -> bool {
        self.as_union().is_some()
    }
}

impl PartialEq for Schema {
    fn eq(&self, other: &Self) -> bool {
        fn canon<'a>(
            union: impl IntoIterator<Item = &'a Schema>,
        ) -> (
            [bool; 6],
            Option<&'a Map<String, Schema>>,
            Option<&'a Schema>,
        ) {
            let mut primitive_types = [false; 6];
            let mut map = None;
            let mut array: Option<&Schema> = None;

            for schema in union.into_iter() {
                match *schema {
                    Schema::Map(ref m) => {
                        if map.is_none() {
                            map = Some(m);
                        } else {
                            panic!("Union should not have multiple Maps inside");
                        }
                    }
                    Schema::Array(ref a) => {
                        if array.is_none() {
                            array = Some(&*a);
                        } else {
                            panic!("Union should not have multiple Arrays inside")
                        }
                    }
                    Schema::Union(_) => {
                        panic!("Union should not have Union as direct child inside")
                    }
                    Schema::Int => primitive_types[0] = true,
                    Schema::Float => primitive_types[1] = true,
                    Schema::Bool => primitive_types[2] = true,
                    Schema::String => primitive_types[3] = true,
                    Schema::Null => primitive_types[4] = true,
                    Schema::Any => primitive_types[5] = true,
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
            Schema::Int => other.is_int(),
            Schema::Float => other.is_float(),
            Schema::Bool => other.is_bool(),
            Schema::String => other.is_string(),
            Schema::Null => other.is_null(),
            Schema::Any => other.is_any(),
        }
    }
}

impl Eq for Schema {}

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
