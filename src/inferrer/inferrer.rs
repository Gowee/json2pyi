use indexmap::IndexMap;
use inflector::Inflector;
/// Infer a schema from a given JSONValue
use serde_json::Value as JSONValue;

use std::collections::HashSet;

// use crate::mapset_impl::Map;
use crate::schema::{ArenaIndex, Map, Schema, Type, TypeArena, Union};
use super::union;

/// infer Schema from `JSONValue`
struct SchemaInferer {/* ... */}

/// An closure for the inferrer to work
pub struct BasicInferrerClosure {
    arena: TypeArena,
    primitive_types: [ArenaIndex; 6],
}

impl BasicInferrerClosure {
    pub fn new() -> Self {
        let mut arena = TypeArena::new();
        let primitive_types = [
            arena.insert(Type::Int),
            arena.insert(Type::Float),
            arena.insert(Type::Bool),
            arena.insert(Type::String),
            arena.insert(Type::Null),
            arena.insert(Type::Any),
        ];
        BasicInferrerClosure {
            arena,
            primitive_types,
        }
    }

    pub fn infer(mut self, json: &JSONValue) -> Schema {
        let root = self.rinfer(json, None);

        let arena = self.arena;
        let primitive_types = self.primitive_types;
        Schema {
            arena,
            primitive_types,
            root,
        }
    }

    fn rinfer(&mut self, json: &JSONValue, outer_name: Option<String>) -> ArenaIndex {
        match *json {
            JSONValue::Number(ref number) => {
                if number.is_f64() {
                    self.primitive_types[1]
                } else {
                    self.primitive_types[0]
                }
            }
            JSONValue::Bool(_) => self.primitive_types[2],
            JSONValue::String(_) => self.primitive_types[3],
            JSONValue::Null => self.primitive_types[4],
            JSONValue::Array(ref array) => {
                let mut types = vec![];
                let outer_name = outer_name.unwrap_or_else(|| String::from("UnnamedType"));
                for value in array.iter() {
                    let type_name = if &outer_name.to_singular() == &outer_name
                        && &outer_name.to_plural() != &outer_name
                    {
                        format!("{}Item", outer_name.to_pascal_case())
                    } else {
                        outer_name.to_singular()
                    };
                    types.push(self.rinfer(value, Some(type_name)))
                }
                let inner = union(&mut self.arena, &mut self.primitive_types, types);
                self.arena.insert(Type::Array(inner))
            }
            JSONValue::Object(ref map) => {
                let mut fields = IndexMap::new();
                for (key, value) in map.iter() {
                    fields.insert(key.to_owned(), self.rinfer(value, Some(key.to_owned())));
                }
                self.arena.insert(Type::Map(Map {
                    name: outer_name.unwrap_or_else(|| String::from("UnnamedType")),
                    fields,
                }))
            }
        }
    }


}

// pub fn infer(json: &JSONValue) -> Type {
//     match *json {
//         JSONValue::Null => Type::Null,
//         JSONValue::Bool(_) => Type::Bool,
//         JSONValue::Number(ref number) => {
//             if number.is_f64() {
//                 Type::Float
//             } else {
//                 Type::Int
//             }
//         }
//         JSONValue::String(_) => Type::String,
//         JSONValue::Array(ref array) => {
//             let inner = union(array.into_iter().map(|value| infer(value)));
//             Type::Array(Box::new(inner))
//         }
//         JSONValue::Object(ref map) => Type::Map(
//             map.iter()
//                 .map(|(key, value)| (key.to_owned(), infer(value)))
//                 .collect(),
//         ),
//     }
// }

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::schema::Type;
    // use serde_json::Value;

    //     #[test]
    //     fn test_primitives() {
    //         let data = r#"
    //         {
    //             "null": null,
    //             "bool": true,
    //             "int": 123,
    //             "negint": -456,
    //             "float": 1.0123,
    //             "string": "hwllo"
    //         }
    //         "#;
    //         let v: Value = serde_json::from_str(data).unwrap();

    //         let s = infer(&v);
    //         assert!(s.is_map());
    //         let map = s.as_map().unwrap();

    //         assert_eq!(
    //             map.iter().map(|(key, _)| key).collect::<Vec<&String>>(),
    //             vec!["null", "bool", "int", "negint", "float", "string"]
    //         ); // order preserving

    //         assert!(map.get("null").unwrap().is_null());
    //         assert!(map.get("bool").unwrap().is_bool());
    //         assert!(map.get("int").unwrap().is_int());
    //         assert!(map.get("negint").unwrap().is_int());
    //         assert!(map.get("string").unwrap().is_string());
    //         assert!(map.get("float").unwrap().is_float());
    //     }

    //     #[test]
    //     fn test_array_one() {
    //         let data = r#"
    //             {
    //                 "array": [1]
    //             }
    //         "#;
    //         let v: Value = serde_json::from_str(data).unwrap();
    //         let s = infer(&v);
    //         assert!(s.is_map());
    //         let map = s.as_map().unwrap();
    //         let array = map.get("array").unwrap();
    //         assert!(array.is_array());
    //         assert!(array.as_array().unwrap().is_int());
    //     }

    //     #[test]
    //     fn test_array_empty() {
    //         let data = r#"
    //             {
    //                 "anys": []
    //             }
    //         "#;
    //         let v: Value = serde_json::from_str(data).unwrap();
    //         let s = infer(&v);
    //         assert!(s.is_map());
    //         let map = s.as_map().unwrap();
    //         let array = map.get("anys").unwrap();
    //         assert!(array.is_array());
    //         assert!(array.as_array().unwrap().is_any());
    //     }

    //     #[test]
    //     fn test_union() {
    //         let data = r#"
    //             {
    //                 "unions": [1, "bo", true]
    //             }
    //         "#;
    //         let v: Value = serde_json::from_str(data).unwrap();
    //         let s = infer(&v);
    //         assert!(s.is_map());
    //         let map = s.as_map().unwrap();
    //         assert!(map.get("unions").unwrap().is_array());
    //         let array = map.get("unions").unwrap().as_array().unwrap();
    //         assert!(array.is_union());
    //         let union = array.as_union().unwrap();
    //         // assert_eq!(union, &[Schema::Int, Schema::String, Schema::Bool]);
    //         assert!(union.len() == 3);
    //         let mut present = [false; 3];
    //         for schema in union.iter() {
    //             if schema.is_int() {
    //                 present[0] = true;
    //             } else if schema.is_bool() {
    //                 present[1] = true;
    //             } else if schema.is_string() {
    //                 present[2] = true;
    //             } else {
    //                 panic!("Unexpected schema: {:?}", schema);
    //             }
    //         }
    //         assert_eq!(present, [true; 3]);
    //     }

    //     #[test]
    //     fn test_any() {
    //         let data = include_str!("../tests/data/empty-array.json");
    //         let v: Value = serde_json::from_str(data).unwrap();
    //         let v = infer(&v);

    //         assert_eq!(
    //             v,
    //             Type::Map(
    //                 vec![(String::from("emptyarray"), Type::Array(Box::new(Type::Any)))]
    //                     .into_iter()
    //                     .collect()
    //             )
    //         );
    //     }

    //     #[test]
    //     fn test_union_of_map_with_any() {
    //         let data = include_str!("../tests/data/union-of-map-with-any.json");
    //         let v: Value = serde_json::from_str(data).unwrap();
    //         let v = infer(&v);

    //         assert_eq!(
    //             v,
    //             Type::Array(Box::new(Type::Map(
    //                 vec![(
    //                     String::from("field1"),
    //                     Type::Array(Box::new(Type::Union(vec![Type::String, Type::Int])))
    //                 )]
    //                 .into_iter()
    //                 .collect()
    //             )))
    //         );
    //     }

    //     #[test]
    //     fn test_union_of_array() {
    //         let data = include_str!("../tests/data/union-of-array.json");
    //         let v: Value = serde_json::from_str(data).unwrap();
    //         let v = infer(&v);

    //         assert_eq!(
    //             v,
    //             Type::Array(Box::new(Type::Array(Box::new(Type::Union(vec![
    //                 Type::Int,
    //                 Type::String,
    //                 Type::Bool
    //             ])))))
    //         );
    //     }

    //     #[test]
    //     fn test_union_of_map_and_others() {
    //         let data = include_str!("../tests/data/union-of-map-and-others.json");
    //         let v: Value = serde_json::from_str(data).unwrap();
    //         let v = infer(&v);

    //         assert_eq!(
    //             v,
    //             Type::Array(Box::new(Type::Union(vec![
    //                 Type::Map(
    //                     vec![(String::from("field1"), Type::Float)]
    //                         .into_iter()
    //                         .collect()
    //                 ),
    //                 Type::Null
    //             ])))
    //         );
    //     }

    //     #[test]
    //     fn test_union_of_array_with_any() {
    //         let data = include_str!("../tests/data/union-of-array-with-any.json");
    //         let v: Value = serde_json::from_str(data).unwrap();
    //         let s = infer(&v);

    //         assert_eq!(
    //             s,
    //             Type::Array(Box::new(Type::Array(Box::new(Type::Union(vec![
    //                 Type::Int,
    //                 Type::String,
    //                 Type::Bool
    //             ])))))
    //         );
    //     }

    //     #[test]
    //     fn test_union_of_map_with_optional_field() {
    //         let data = include_str!("../tests/data/union-of-map-with-optional-field.json");
    //         let v: Value = serde_json::from_str(data).unwrap();
    //         let s = infer(&v);

    //         assert_eq!(
    //             s,
    //             Type::Array(Box::new(Type::Map(
    //                 vec![
    //                     (String::from("name"), Type::String),
    //                     (
    //                         String::from("address"),
    //                         Type::Union(vec![Type::String, Type::Null])
    //                     )
    //                 ]
    //                 .into_iter()
    //                 .collect()
    //             )))
    //         );
    //     }

    //     #[test]
    //     fn test_quicktype() {
    //         let data = include_str!("../tests/data/quicktype.json");
    //         let v: Value = serde_json::from_str(data).unwrap();
    //         dbg!(infer(&v));
    //     }

    // #[test]
    // fn test_jvilk_maketypes() {
    //     let data = include_str!("../tests/data/jvilk-maketypes.json");
    //     let v: Value = serde_json::from_str(data).unwrap();
    //     let schema = InferrerClosure::new().infer(&v);
    //     dbg!(schema);
    // }
}
