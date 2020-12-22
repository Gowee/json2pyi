use inflector::Inflector;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use std::{
    collections::HashSet,
    fmt::{self, Write},
    write,
};
// use crate::mapset_impl::Map;
use crate::schema::{ArenaIndex, ITypeArena, Map, Schema, Type, Union};

use super::{GenOutput, Indentation, TargetGenerator};

#[derive(Debug, Serialize, Deserialize)]
pub struct PythonDataclasses {
    pub generate_type_alias_for_union: bool,
    pub indentation: Indentation,
}

#[typetag::serde]
impl TargetGenerator for PythonDataclasses {
    fn generate(&self, schema: &Schema) -> GenOutput {
        let closure = GeneratorClosure::new(schema, self);

        closure.run()
    }
}

impl PythonDataclasses {
    fn write_indentation(&self, s: &mut String) -> fmt::Result {
        match self.indentation {
            Indentation::Space(len) => {
                for _ in 0..len {
                    write!(s, " ")?;
                }
            }
            Indentation::Tab => {
                write!(s, "\t")?;
            }
        }
        Ok(())
    }
}

// pub fn schema_to_dataclasses(schema: &mut Schema) -> String {
//     DataclassesGeneratorClosure::new(schema).generate()
// }

#[derive(Debug)]
pub struct GeneratorClosure<'a> {
    schema: &'a Schema,
    options: &'a PythonDataclasses,
    header: String,
    body: String,
}

impl<'a> GeneratorClosure<'a> {
    pub fn new(schema: &'a Schema, options: &'a PythonDataclasses) -> Self {
        GeneratorClosure {
            schema,
            options,
            header: String::new(),
            body: String::new(),
        }
    }

    pub fn run(mut self) -> GenOutput {
        for r#type in self.schema.iter_topdown() {
            match *r#type {
                Type::Map(Map {
                    ref name_hints,
                    ref fields,
                }) => {
                    write!(self.body, "class ").unwrap();
                    if name_hints.is_empty() {
                        write!(
                            self.body,
                            "UnnammedType{:X}",
                            r#type as *const Type as usize
                        )
                        .unwrap();
                    } else {
                        write!(self.body, "{}", name_hints.iter().join("Or")).unwrap();
                    }
                    write!(self.body, ":\n").unwrap();
                    for (key, &r#type) in fields.iter() {
                        self.options.write_indentation(&mut self.body).unwrap();
                        write!(
                            self.body,
                            "{}: {}\n",
                            key,
                            self.get_type_name_by_index(r#type).unwrap()
                        )
                        .unwrap();
                    }
                    write!(self.body, "\n").unwrap();
                }
                Type::Union(Union {
                    ref name_hints,
                    ref types,
                }) => {
                    if self.options.generate_type_alias_for_union {
                        let is_non_trivial = (types.len()
                            - types.contains(&self.schema.arena.get_index_of_primitive(Type::Null))
                                as usize)
                            > 1;
                        if is_non_trivial {
                            if name_hints.is_empty() {
                                write!(
                                    self.body,
                                    "UnnammedUnion{:X}",
                                    r#type as *const Type as usize
                                )
                                .unwrap();
                            } else {
                                write!(self.body, "{}", name_hints).unwrap();
                            }

                            write!(
                                self.body,
                                "{}Union = Union[{}]",
                                name_hints.iter().join("Or"),
                                types
                                    .iter()
                                    .cloned()
                                    .map(|r#type| self.get_type_name_by_index(r#type).unwrap())
                                    .join(", ")
                            )
                            .unwrap();
                            write!(self.body, "\n").unwrap();
                        }
                    }
                }
                _ => {}
            }
        }
        GenOutput {
            header: self.header,
            body: self.body,
            additional: String::new(),
        }
    }

    pub fn get_type_name_by_index(&self, i: ArenaIndex) -> Option<String> {
        self.schema
            .arena
            .get(i)
            .map(|r#type| self.get_type_name(r#type))
    }

    pub fn get_type_name(&self, r#type: &Type) -> String {
        // match self.schema.arena.
        match *r#type {
            Type::Map(Map {
                ref name_hints,
                fields: _,
            }) => name_hints.iter().join("Or"),
            Type::Union(Union {
                ref name_hints,
                ref types,
            }) => {
                // name_hints.iter().join("Or")
                // FIX: the below line will panic as typeref in unions are not updated after optimizing so far
                if self.options.generate_type_alias_for_union && {
                    let is_non_trivial = (types.len()
                        - types.contains(&self.schema.arena.get_index_of_primitive(Type::Null))
                            as usize)
                        > 1;
                    is_non_trivial
                } {
                    name_hints.iter().join("Or")
                } else {
                    let mut optional = false;
                    let inner = types
                        .iter()
                        .cloned()
                        .map(|r#type| self.schema.arena.get(r#type).unwrap())
                        .filter(|&r#type| {
                            if r#type.is_null() {
                                optional = true;
                                false
                            } else {
                                true
                            }
                        })
                        .map(|r#type| self.get_type_name(r#type))
                        .join(", ");
                    if optional {
                        format!("Optional[{}]", inner)
                    } else {
                        inner
                    }
                }
            }
            Type::Array(r#type) => {
                dbg!(r#type);
                format!("List[{}]", self.get_type_name_by_index(r#type).unwrap())
            }
            Type::Int => String::from("int"),
            Type::Float => String::from("float"),
            Type::Bool => String::from("bool"),
            Type::String => String::from("str"),
            Type::Date => String::from("datetime"),
            Type::UUID => String::from("UUID"),
            Type::Null => String::from("None"),
            Type::Any => String::from("Any"),
        }
    }

    // pub fn get_union_as_variants(&self, union: &Union) -> String {
    // let mut optional = false;
    // let types =
    // let inner = types
    //     .iter()
    //     .cloned()
    //     .map(|r#type| self.schema.arena.get(r#type).unwrap())
    //     .filter(|&r#type| {
    //         if r#type.is_null() {
    //             optional = true;
    //             false
    //         } else {
    //             true
    //         }
    //     })
    //     .map(|r#type| self.get_type_name(r#type))
    //     .join(", ");
    // if optional {
    //     format!("Optional[{}]", inner)
    // } else {
    //     inner
    // }
    // }
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
