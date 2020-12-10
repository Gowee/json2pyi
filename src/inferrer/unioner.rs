use indexmap::IndexMap;
use inflector::Inflector;
/// Infer a schema from a given JSONValue
use serde_json::Value as JSONValue;

use std::collections::HashSet;
use std::mem;

// use crate::mapset_impl::Map;
use crate::schema::{ArenaIndex, Map, Schema, Type, TypeArena, Union};

pub fn union(
    arena: &mut TypeArena,
    primitive_types: &[ArenaIndex; 6],
    types: impl IntoIterator<Item = ArenaIndex>,
) -> ArenaIndex {
    Unioner::new(arena, primitive_types).union(types)
}

pub struct Unioner<'a> {
    // Unioner is pub, so it is not named UnionerClosure.
    arena: &'a mut TypeArena,
    primitive_types: &'a [ArenaIndex; 6],
}

impl<'a> Unioner<'a> {
    pub fn new(arena: &'a mut TypeArena, primitive_types: &'a [ArenaIndex; 6]) -> Self {
        Self {
            arena,
            primitive_types,
        }
    }

    pub fn union(mut self, types: impl IntoIterator<Item = ArenaIndex>) -> ArenaIndex {
        self.runion(types)
    }

    pub fn runion(&mut self, types: impl IntoIterator<Item = ArenaIndex>) -> ArenaIndex {
        let mut unioned = HashSet::new();
        // The first Type::Map is kept to be unioned into.
        let mut first_map: Option<ArenaIndex> = None;
        // All Maps are collected at first and then merged into one unioned Map, field by field.
        let mut maps: Option<IndexMap<String, Vec<ArenaIndex>>> = None;
        let mut map_count = 0; // Used to determine whether a field is present in all Maps.
                               // All Arrays are collected at first. Then their inner types are unioned recursively.
                               // e.g. `int[], (int | bool)[], string[]` -> (int | bool | string)[]
        let mut arrays = vec![];

        let types: Vec<ArenaIndex> = types
            .into_iter()
            .flat_map(|r#type| {
                match self
                    .arena
                    .get(r#type)
                    .expect("It should be there during recusive inferring/unioning")
                {
                    Type::Union(_) => {
                        self.arena
                            .remove(r#type)
                            .unwrap()
                            .into_union()
                            .unwrap()
                            .types
                            .into_iter()
                            .collect::<Vec<ArenaIndex>>() // remove & expand the union
                    }
                    _ => vec![r#type], // TODO: avoid unnecessary Vec
                }
            })
            .collect();
        for r#type in types {
            match *self.arena.get(r#type).unwrap() {
                Type::Map(_) => {
                    let map;
                    if first_map.is_none() {
                        // If this is the first map in the union, just take its inner out so that
                        // its slot can be reused again with ArenaIndex left intact.
                        first_map = Some(r#type);
                        map = mem::take(self.arena.get_mut(r#type).unwrap())
                            .into_map()
                            .unwrap();
                    } else {
                        // O.W., just remove the type from the arena.
                        map = self.arena.remove(r#type).unwrap().into_map().unwrap();
                    }
                    let maps = maps.get_or_insert_with(|| Default::default());
                    for (key, schema) in map.fields.into_iter() {
                        maps.entry(key).or_default().push(schema);
                    }
                    map_count += 1;
                }
                Type::Array(array) => {
                    arrays.push(array);
                }
                Type::Union(_) => unreachable!(), // union should have been expanded above
                Type::Int => {
                    unioned.insert(self.primitive_types[0]);
                }
                Type::Float => {
                    unioned.insert(self.primitive_types[1]);
                }
                Type::Bool => {
                    unioned.insert(self.primitive_types[2]);
                }
                Type::String => {
                    unioned.insert(self.primitive_types[3]);
                }
                Type::Null => {
                    unioned.insert(self.primitive_types[4]);
                }
                Type::Any => {
                    unioned.insert(self.primitive_types[5]);
                }
            }
        }

        // let mut schemas = vec![];

        if let Some(maps) = maps {
            // merge maps recursively by unioning every possible fields
            let unioned_map: IndexMap<String, ArenaIndex> = maps
                .into_iter()
                .map(|(key, mut types)| {
                    // The field is nullable if not present in every Map.
                    if types.len() < map_count {
                        types.push(self.primitive_types[4]); // Null
                    }
                    (key, self.runion(types))
                })
                .collect();
            if unioned_map.is_empty() {
                // every map is empty (no field at all)
                // TODO: Any or unit type?
                // TODO: should slot be removed from arena here?
                unioned.insert(self.primitive_types[5]); // Any
            } else {
                let slot = first_map.unwrap();
                *self.arena.get_mut(slot).unwrap() = Type::Map(Map {
                    name: String::from("aa"),
                    fields: unioned_map,
                });
                unioned.insert(slot);
            }
        }
        if !arrays.is_empty() {
            let inner = self.runion(arrays);
            unioned.insert(self.arena.insert(Type::Array(inner)));
        }
        if unioned.contains(&self.primitive_types[0]) && unioned.contains(&self.primitive_types[1])
        {
            // In JS(ON), int and float are both number, which implies 1.0 is serialized as 1.
            // So if both int and float present in the union, just treat it as float.
            unioned.remove(&self.primitive_types[0]);
        }
        // if primitive_types[1] {
        //     schemas.push(Type::Float);
        // } else if primitive_types[0] {
        //     // In JS(ON), int and float are both number, which implies 1.0 is serialized as 1.
        //     // So if both int and float present in the union, just treat it as float.
        //     schemas.push(Type::Int);
        // }
        // if primitive_types[2] {
        //     schemas.push(Type::Bool);
        // }
        // if primitive_types[3] {
        //     schemas.push(Type::String);
        // }
        // if primitive_types[4] {
        //     schemas.push(Type::Null);
        // }
        // if schemas.is_empty() && primitive_types[5] {
        //     // Any implies undetermined (e.g. [] or {}). So set it only if there are no concrete type.
        //     schemas.push(Type::Any);
        // }
        match unioned.len() {
            0 => self.primitive_types[5], // Any
            1 => unioned.drain().nth(0).unwrap(),
            _ => self.arena.insert(Type::Union(Union {
                name: String::from("UnnamedUnion"),
                types: unioned,
            })),
        }
    }
}
