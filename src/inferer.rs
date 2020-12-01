/// Infer a schema from a given JSON HashMap
use serde_json::Value as JSONValue;

use crate::mapset_impl::Map;
use crate::schema::Schema;

// const ROOT_NAME: &'static str = "SomeJSON";

pub fn infer(json: &JSONValue) -> Schema {
    match *json {
        JSONValue::Null => Schema::Null,
        JSONValue::Bool(_) => Schema::Bool,
        JSONValue::Number(ref number) => {
            if number.is_f64() {
                Schema::Float
            } else {
                Schema::Int
            }
        }
        JSONValue::String(_) => Schema::String,
        JSONValue::Array(ref array) => {
            let inner = match array.len() {
                0 => Schema::Any,
                1 => infer(array.first().unwrap()),
                _ => union(array.into_iter().map(|value| infer(value))),
            };
            Schema::Array(Box::new(inner))
        }
        JSONValue::Object(ref map) => Schema::Map(
            map.iter()
                .map(|(key, value)| (key.to_owned(), infer(value)))
                .collect(),
        ),
    }
}

fn union(schemas: impl IntoIterator<Item = Schema>) -> Schema {
    // Int, Float, Bool, String, Null, Any
    let mut primitive_types = [false, false, false, false, false, false];
    let mut maps: Option<Map<String, Vec<Schema>>> = None;
    let mut arrays = vec![];

    for schema in schemas.into_iter().flat_map(|schema| match schema {
        Schema::Union(schemas) => schemas, // expand union
        _ => vec![schema],
    }) {
        match schema {
            Schema::Map(map) => {
                // maps.push(map);
                let maps = maps.get_or_insert_with(|| Map::new());
                for (key, schema) in map.into_iter() {
                    maps.entry(key).or_default().push(schema);
                }
                // unioned_map = match unioned_map {
                //     Some(mut unioned_map) => {
                //         for (key, schema) in map.into_iter() {
                //             if unioned_map.contains_key(&key) {
                //                 let schemas = unioned_map.remove(&key).unwrap();
                //                 unioned_map
                //                     .insert(key, Schema::Union(union(vec![schemas, schema])));
                //             } else {
                //                 unioned_map.insert(key, schema);
                //             }
                //         }
                //         Some(unioned_map)
                //     }
                //     None => Some(map),
                // }
            }
            Schema::Array(array) => {
                arrays.push(*array);
            }
            Schema::Union(_) => unreachable!(),
            Schema::Int => primitive_types[0] = true,
            Schema::Float => primitive_types[1] = true,
            Schema::Bool => primitive_types[2] = true,
            Schema::String => primitive_types[3] = true,
            Schema::Null => primitive_types[4] = true,
            Schema::Any => primitive_types[5] = true,
        }
    }

    let mut schemas = vec![];

    if let Some(maps) = maps {
        let unioned_map: Map<String, Schema> = maps
            .into_iter()
            .map(|(key, schemas)| (key, union(schemas)))
            .collect();
        schemas.push(if unioned_map.is_empty() {
            Schema::Any
        } else {
            Schema::Map(unioned_map)
        })
    }
    if !arrays.is_empty() {
        let inner = if arrays.len() == 1 {
            arrays.pop().unwrap()
        } else {
            union(
                arrays
                    .into_iter()
                    .flat_map(|array| match array {
                        Schema::Union(union) => union,
                        schema => vec![schema],
                    })
                    .collect::<Vec<Schema>>(),
            )
        };
        schemas.push(Schema::Array(Box::new(inner)));
    }
    // if let Some(map) = unioned_map {
    //     schemas.push(Schema::Map(map));
    // }
    if primitive_types[0] {
        schemas.push(Schema::Int);
    }
    if primitive_types[1] {
        schemas.push(Schema::Float);
    }
    if primitive_types[2] {
        schemas.push(Schema::Bool);
    }
    if primitive_types[3] {
        schemas.push(Schema::String);
    }
    if primitive_types[4] {
        schemas.push(Schema::Null);
    }
    if schemas.is_empty() && primitive_types[5] {
        schemas.push(Schema::Any);
    }
    match schemas.len() {
        0 => Schema::Any,
        1 => schemas.pop().unwrap(),
        _ => Schema::Union(schemas),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::schema::Schema;

    use super::*;

    #[test]
    fn test_primitives() {
        let data = r#"
        {
            "null": null,
            "bool": true,
            "int": 123,
            "negint": -456,
            "float": 1.0123,
            "string": "hwllo"
        }
        "#;
        let v: Value = serde_json::from_str(data).unwrap();

        let s = infer(&v);
        assert!(s.is_map());
        let map = s.as_map().unwrap();

        assert_eq!(
            map.iter().map(|(key, _)| key).collect::<Vec<&String>>(),
            vec!["null", "bool", "int", "negint", "float", "string"]
        ); // order preserving

        assert!(map.get("null").unwrap().is_null());
        assert!(map.get("bool").unwrap().is_bool());
        assert!(map.get("int").unwrap().is_int());
        assert!(map.get("negint").unwrap().is_int());
        assert!(map.get("string").unwrap().is_string());
        assert!(map.get("float").unwrap().is_float());
    }

    #[test]
    fn test_array_one() {
        let data = r#"
            {
                "array": [1]
            }
        "#;
        let v: Value = serde_json::from_str(data).unwrap();
        let s = infer(&v);
        assert!(s.is_map());
        let map = s.as_map().unwrap();
        let array = map.get("array").unwrap();
        assert!(array.is_array());
        assert!(array.as_array().unwrap().is_int());
    }

    #[test]
    fn test_array_empty() {
        let data = r#"
            {
                "anys": []
            }
        "#;
        let v: Value = serde_json::from_str(data).unwrap();
        let s = infer(&v);
        assert!(s.is_map());
        let map = s.as_map().unwrap();
        let array = map.get("anys").unwrap();
        assert!(array.is_array());
        assert!(array.as_array().unwrap().is_any());
    }

    #[test]
    fn test_union() {
        let data = r#"
            {
                "unions": [1, "bo", true]
            }
        "#;
        let v: Value = serde_json::from_str(data).unwrap();
        let s = infer(&v);
        assert!(s.is_map());
        let map = s.as_map().unwrap();
        assert!(map.get("unions").unwrap().is_array());
        let array = map.get("unions").unwrap().as_array().unwrap();
        assert!(array.is_union());
        let union = array.as_union().unwrap();
        // assert_eq!(union, &[Schema::Int, Schema::String, Schema::Bool]);
        assert!(union.len() == 3);
        let mut present = [false; 3];
        for schema in union.iter() {
            if schema.is_int() {
                present[0] = true;
            } else if schema.is_bool() {
                present[1] = true;
            } else if schema.is_string() {
                present[2] = true;
            } else {
                panic!("Unexpected schema: {:?}", schema);
            }
        }
        assert_eq!(present, [true; 3]);
    }

    #[test]
    fn test_any() {
        let data = include_str!("../tests/data/empty-array.json");
        let v: Value = serde_json::from_str(data).unwrap();
        let v = infer(&v);

        assert_eq!(
            v,
            Schema::Map(
                vec![(
                    String::from("emptyarray"),
                    Schema::Array(Box::new(Schema::Any))
                )]
                .into_iter()
                .collect()
            )
        );
    }

    #[test]
    fn test_union_with_any() {
        let data = include_str!("../tests/data/union-with-any.json");
        let v: Value = serde_json::from_str(data).unwrap();
        let v = infer(&v);

        assert_eq!(
            v,
            Schema::Array(Box::new(Schema::Map(
                vec![(
                    String::from("field1"),
                    Schema::Array(Box::new(Schema::Union(vec![Schema::String, Schema::Int])))
                )]
                .into_iter()
                .collect()
            )))
        );
    }
    #[test]
    fn test_union_of_array() {
        let data = include_str!("../tests/data/union-of-array.json");
        let v: Value = serde_json::from_str(data).unwrap();
        let v = infer(&v);

        assert_eq!(
            v,
            Schema::Array(Box::new(Schema::Array(Box::new(Schema::Union(vec![
                Schema::Int,
                Schema::String,
                Schema::Bool
            ])))))
        );
    }

    #[test]
    fn test_union_of_array_with_any() {
        let data = include_str!("../tests/data/union-of-array-with-any.json");
        let v: Value = serde_json::from_str(data).unwrap();
        let v = infer(&v);

        assert_eq!(
            v,
            Schema::Array(Box::new(Schema::Array(Box::new(Schema::Union(vec![
                Schema::Int,
                Schema::String,
                Schema::Bool
            ])))))
        );
    }
    #[test]
    fn test_quicktype() {
        let data = include_str!("../tests/data/quicktype.json");
        let v: Value = serde_json::from_str(data).unwrap();
        dbg!(infer(&v));
    }

    #[test]
    fn test_jvilk_maketypes() {
        let data = include_str!("../tests/data/jvilk-maketypes.json");
        let v: Value = serde_json::from_str(data).unwrap();
        dbg!(infer(&v));
    }
}
