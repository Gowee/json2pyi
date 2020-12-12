use inflector::Inflector;
use itertools::Itertools;

use std::collections::HashSet;
// use crate::mapset_impl::Map;
use crate::schema::{ArenaIndex, Map, Schema, Type, Union};

const ROOT_NAME: &'static str = "UnnamedObject";

pub fn schema_to_dataclasses(schema: &mut Schema) -> String {
    DataclassesGeneratorClosure::new(schema).generate()
}

#[derive(Debug)]
pub struct DataclassesGeneratorClosure<'a> {
    schema: &'a Schema,
    types_to_import: HashSet<String>,
    datatypes: Vec<String>,
}

impl<'a> DataclassesGeneratorClosure<'a> {
    pub fn new(schema: &'a Schema) -> Self {
        DataclassesGeneratorClosure {
            schema,
            types_to_import: HashSet::new(),
            datatypes: Vec::new(),
        }
    }

    pub fn generate(mut self) -> String {
        for (_, r#type) in self.schema.arena.iter() {
            match *r#type {
                Type::Map(Map {
                    ref name_hints,
                    ref fields,
                }) => {
                    let mut def = String::new();
                    def.push_str(&format!("class {}\n", name_hints.iter().join("Or")));

                    for (key, &r#type) in fields.iter() {
                        def.push_str("    ");
                        def.push_str(key);
                        def.push_str(": ");
                        def.push_str(&self.get_type_name_by_index(r#type).unwrap());
                        def.push_str("\n");
                    }
                    self.datatypes.push(def);
                }
                _ => {}
            }
        }
        self.datatypes.join("\n\n")
    }

    pub fn get_type_name_by_index(&self, i: ArenaIndex) -> Option<String>{
        self.schema.arena.get(i).map(|r#type| self.get_type_name(r#type))
    }

    pub fn get_type_name(&self, r#type: &Type) -> String {
        // match self.schema.arena.
        match *r#type {
            Type::Map(Map {
                ref name_hints,
                fields: _,
            }) => name_hints.iter().join("Or"),
            Type::Union(Union { ref name_hints, ref types }) => {
                name_hints.iter().join("Or")
                // FIX: the below line will panic as typeref in unions are not updated after optimizing so far 
                // types.iter().cloned().map(|r#type| self.get_type_name_by_index(r#type).unwrap()).join(", ")
            },
            Type::Array(r#type) => {
                dbg!(r#type);
                format!("List[{}]", self.get_type_name_by_index(r#type).unwrap())
            },
            Type::Int => String::from("int"),
            Type::Float => String::from("float"),
            Type::Bool => String::from("bool"),
            Type::String => String::from("str"),
            Type::Null => String::from("None"),
            Type::Any => String::from("Any"),
        }
    }
}

// impl Schema {
//     pub fn to_dataclasses(&self) -> String {
//         unimplemented!();
// let named_schemas: Vec<(&str, Map<String, Schema>)>  = vec![];
// let mut stack: Vec<&Schema> = vec![self];
// let mut output = vec![];

//         fn traverse(
//             schema: &Type,
//             outer_name: Option<String>,
//             output: &mut Vec<String>,
//         ) -> String {
//             // dbg!(schema, &outer_name);
//             match &schema {
//                 Type::Map(ref map) => {
//                     let class_name = String::from(outer_name.unwrap_or(String::from(ROOT_NAME))); // TODO: convert case and suffix
//                     // dbg!(&class_name);
//                     let fields: Vec<String> = map
//                         .iter()
//                         .map(|(key, schema)| {
//                             let type_name = if schema.is_array() {
//                                 if &key.to_singular() == key && &key.to_plural() != key{
//                                     format!("{}Item", key.to_pascal_case())
//                                 } else {
//                                     key.to_pascal_case()
//                                 }
//                             } else {
//                                 key.to_pascal_case()
//                             };
//                             format!("{} = {}", key, traverse(schema, Some(type_name), output))
//                         })
//                         .collect();
//                     output.push(format!(
//                         "\
// @dataclass
// class {}:
//     {}",
//                         class_name,
//                         fields.join("\n    ")
//                     ));
//                     String::from(class_name)
//                 }
//                 Type::Array(ref array) => {
//                     format!("List[{}]", traverse(array, outer_name, output))
//                 }
//                 Type::Union(ref union) => {
//                     let mut optional = false;
//                     let t = union
//                         .iter()
//                         .filter(|schema| {
//                             if schema.is_null() {
//                                 optional = true;
//                                 false
//                             } else {
//                                 true
//                             }
//                         })
//                         .map(|schema| traverse(schema, outer_name.clone(), output))
//                         .join(" | ");
//                     if t.is_empty() {
//                         if optional {
//                             String::from("None")
//                         } else {
//                             panic!("Union should not be empty") // empty union
//                         }
//                     } else {
//                         format!("Optional[{}]", t)
//                     }
//                 }
//                 Type::Int => String::from("int"),
//                 Type::Float => String::from("float"),
//                 Type::Bool => String::from("bool"),
//                 Type::String => String::from("str"),
//                 // TODO: treat `* | null` as `Optional[*]`
//                 Type::Null => String::from("None"), // unreachable!()
//                 Type::Any => String::from("Any"),
//             }
//         }

//         // while let Some(&schema) = stack.last() {
//         //     match schema
//         // }

//         traverse(self, Some(String::from(root_name)), &mut output);

//         output.join("\n\n")
//     }
// }

// #[cfg(test)]
// mod tests {
//     use crate::inferrer::infer;

//     #[test]
//     fn test_to_dataclasses() {
//         let data = include_str!("../../tests/data/jvilk-maketypes.json");
//         let s = infer(&serde_json::from_str(data).unwrap());
//         println!("Redered: {}", s.to_dataclasses("RootObject"));
//     }
// }
