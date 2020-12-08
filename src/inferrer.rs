use indexmap::IndexMap;
use inflector::Inflector;
/// Infer a schema from a given JSONValue
use serde_json::Value as JSONValue;

// use crate::mapset_impl::Map;
use crate::schema::{ArenaIndex, Map, Schema, Type, TypeArena};

/// An inferrer to infer Schema from `JSONValue`
struct SchemaInferrer {
    arena: Option<TypeArena>,
    ///
    primitive_types: Option<[ArenaIndex; 6]>,
}

impl SchemaInferrer {
    pub fn new() -> Self {
        SchemaInferrer {
            arena: None,
            primitive_types: None,
        }
    }

    pub fn infer(&mut self, json: &JSONValue) -> Schema {
        let mut arena = TypeArena::new();
        let mut ptypes = [
            arena.insert(Type::Int),
            arena.insert(Type::Float),
            arena.insert(Type::Bool),
            arena.insert(Type::String),
            arena.insert(Type::Null),
            arena.insert(Type::Any),
        ];

        self.arena = Some(arena);
        self.primitive_types = Some(ptypes);

        let root = self.rinfer(json, None);

        let arena = self.arena.take().unwrap();
        let _ = self.primitive_types.take();
        Schema {
            arena: arena,
            root: root,
        }
    }

    fn rinfer(&mut self, json: &JSONValue, outer_name: Option<String>) -> ArenaIndex {
        let primitive_types = self.primitive_types.unwrap();
        match *json {
            JSONValue::Number(ref number) => {
                if number.is_f64() {
                    primitive_types[1]
                } else {
                    primitive_types[0]
                }
            }
            JSONValue::Bool(_) => primitive_types[2],
            JSONValue::String(_) => primitive_types[3],
            JSONValue::Null => primitive_types[4],
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
                let inner = self.union(types);
                self.arena.as_mut().unwrap().insert(Type::Array(inner))
            }
            JSONValue::Object(ref map) => {
                let mut fields = IndexMap::new();
                for (key, value) in map.iter() {
                    fields.insert(key.to_owned(), self.rinfer(value, Some(key.to_owned())));
                }
                self.arena.as_mut().unwrap().insert(Type::Map(Map {
                    name: outer_name.unwrap_or_else(|| String::from("UnnamedType")),
                    fields,
                }))
            }
        }
    }

    fn union(&mut self, types: impl IntoIterator<Item = ArenaIndex>) -> Type {
        // All Maps are collected at first and then merged into one unioned Map, field by field.
        let mut maps: Option<Map<String, Vec<Type>>> = None;
        let mut map_count = 0; // Used to determine whether a field is present in all Maps.
                               // All Arrays are collected at first. Then their inner types are unioned recursively.
                               // e.g. `int[], (int | bool)[], string[]` -> (int | bool | string)[]
        let mut arrays = vec![];

        for r#type in types.into_iter().flat_map(|r#type| match self.arena.unwrap().get(r#type) {
            Type::Union(schemas) => schemas, // expand union
            _ => vec![schema],               // TODO: avoid unnecessary Vec
        }) {
            match schema {
                Type::Map(map) => {
                    let maps = maps.get_or_insert_with(|| Map::new());
                    for (key, schema) in map.into_iter() {
                        maps.entry(key).or_default().push(schema);
                    }
                    map_count += 1;
                }
                Type::Array(array) => {
                    arrays.push(*array);
                }
                Type::Union(_) => unreachable!(), // union should have been expanded above
                Type::Int => primitive_types[0] = true,
                Type::Float => primitive_types[1] = true,
                Type::Bool => primitive_types[2] = true,
                Type::String => primitive_types[3] = true,
                Type::Null => primitive_types[4] = true,
                Type::Any => primitive_types[5] = true,
            }
        }

        let mut schemas = vec![];

        if let Some(maps) = maps {
            // merge maps recursively by unioning every possible fields
            let unioned_map: Map<String, Type> = maps
                .into_iter()
                .map(|(key, mut schemas)| {
                    // The field is nullable if not present in every Map.
                    if schemas.len() < map_count {
                        schemas.push(Type::Null); // Null
                    }
                    (key, union(schemas))
                })
                .collect();
            if unioned_map.is_empty() {
                // every map is empty (no field at all)
                // TODO: Any or unit type?
                primitive_types[5] = true; // Any
            } else {
                schemas.push(Type::Map(unioned_map));
            }
        }
        if !arrays.is_empty() {
            schemas.push(Type::Array(Box::new(union(arrays))));
        }
        if primitive_types[1] {
            schemas.push(Type::Float);
        } else if primitive_types[0] {
            // In JS(ON), int and float are both number, which implies 1.0 is serialized as 1.
            // So if both int and float present in the union, just treat it as float.
            schemas.push(Type::Int);
        }
        if primitive_types[2] {
            schemas.push(Type::Bool);
        }
        if primitive_types[3] {
            schemas.push(Type::String);
        }
        if primitive_types[4] {
            schemas.push(Type::Null);
        }
        if schemas.is_empty() && primitive_types[5] {
            // Any implies undetermined (e.g. [] or {}). So set it only if there are no concrete type.
            schemas.push(Type::Any);
        }
        match schemas.len() {
            0 => Type::Any,
            1 => schemas.pop().unwrap(),
            _ => Type::Union(schemas),
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

// #[cfg(test)]
// mod tests {
//     use serde_json::Value;

//     use crate::schema::Type;

//     use super::*;

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

//     #[test]
//     fn test_jvilk_maketypes() {
//         let data = include_str!("../tests/data/jvilk-maketypes.json");
//         let v: Value = serde_json::from_str(data).unwrap();
//         dbg!(infer(&v));
//     }
// }
