use indexmap::IndexMap;
use inflector::Inflector;
/// Infer a schema from a given JSONValue
use serde_json::Value as JSONValue;

use std::collections::HashSet;
use std::mem;

// use crate::mapset_impl::Map;
use crate::schema::{ArenaIndex, ITypeArena, Map, Schema, Type, TypeArena, Union};

pub fn union(arena: &mut TypeArena, types: impl IntoIterator<Item = ArenaIndex>) -> ArenaIndex {
    Unioner::new(arena).union(types)
}

pub struct Unioner<'a, T: ITypeArena> {
    // Unioner is pub, so it is not named UnionerClosure.
    arena: &'a mut T,
}

impl<'a, T: ITypeArena> Unioner<'a, T> {
    pub fn new(arena: &'a mut T) -> Self {
        Self { arena }
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
        let mut map_name_hints = HashSet::new();
        let mut arrays = vec![];

        let types: Vec<ArenaIndex> = types
            .into_iter()
            .flat_map(|r#type| {
                // dbg!(r#type);
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
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        // dbg!(&types);
        let tys = types.clone();
        for r#type in types {
            dbg!(r#type, self.arena.get(r#type));
            match *self.arena.get(r#type).unwrap() {
                Type::Map(_) => {
                    // {// dbg!(self.arena.get(r#type).unwrap());}
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
                        map = self
                            .arena
                            .remove_in_favor_of(r#type, first_map.unwrap())
                            .unwrap()
                            .into_map()
                            .unwrap();
                    }
                    let maps = maps.get_or_insert_with(|| Default::default());
                    for (key, r#type) in map.fields.into_iter() {
                        maps.entry(key).or_default().push(r#type);
                    }
                    map_count += 1;
                    dbg!(&map.name_hints);
                    map_name_hints.extend(map.name_hints); // NOTE: HashSet.union is NOT in-place.
                }
                Type::Array(array) => {
                    arrays.push(array);
                }
                Type::Union(_) => unreachable!(), // union should have been expanded above
                _ => {
                    // O.W. it is a primitive type. Then just add it to the union as is.
                    // Note: SPECIAL CASE:
                    // first_map in a union is taken out with a Any left there. In a recursive
                    // sub-call, if Any is explicitly matched out and put into the union with
                    // get_index_of_primitive, then it might cause problem. So in the current
                    // implementation, the ArenaIndex for Any has to be put into the union as is.
                    // See the linked-list or tree-recursion test case.
                    unioned.insert(r#type);
                } // Type::Int => {
                                                   //     unioned.insert(self.arena.get_index_of_primitive(Type::Int));
                                                   // }
                                                   // Type::Float => {
                                                   //     unioned.insert(self.arena.get_index_of_primitive(Type::Float));
                                                   // }
                                                   // Type::Bool => {
                                                   //     unioned.insert(self.arena.get_index_of_primitive(Type::Bool));
                                                   // }
                                                   // Type::String => {
                                                   //     unioned.insert(self.arena.get_index_of_primitive(Type::String));
                                                   // }
                                                   // Type::Null => {
                                                   //     unioned.insert(self.arena.get_index_of_primitive(Type::Null));
                                                   // }
                                                   // Type::Any => {
                                                   //     unioned.insert(self.arena.get_index_of_primitive(Type::Any));
                                                   // }
            }
        }

        // let mut schemas = vec![];
        dbg!(&first_map, &maps);
        if let Some(maps) = maps {
            // merge maps recursively by unioning every possible fields
            // dbg!(&maps);
            let unioned_map: IndexMap<String, ArenaIndex> = maps
                .into_iter()
                .map(|(key, mut types)| {
                    // The field is nullable if not present in every Map.
                    if types.len() < map_count {
                        types.push(self.arena.get_index_of_primitive(Type::Null));
                        // Null
                    }
                    (key, self.runion(types))
                })
                .collect();
            dbg!(&unioned_map);
            if unioned_map.is_empty() {
                // every map is empty (no field at all)
                // TODO: Any or unit type?
                // TODO: should slot be removed from arena here?
                unioned.insert(self.arena.get_index_of_primitive(Type::Any)); // Any
            } else {
                let slot = first_map.unwrap();
                *self.arena.get_mut(slot).unwrap() = Type::Map(Map {
                    name_hints: map_name_hints,
                    fields: unioned_map,
                });
                unioned.insert(slot);
            }
        }
        if !arrays.is_empty() {
            let inner = self.runion(arrays);
            unioned.insert(self.arena.insert(Type::Array(inner)));
        }
        if unioned.contains(&self.arena.get_index_of_primitive(Type::Int))
            && unioned.contains(&self.arena.get_index_of_primitive(Type::Float))
        {
            // In JS(ON), int and float are both number, which implies 1.0 is serialized as 1.
            // So if both int and float present in the union, just treat it as float.
            unioned.remove(&self.arena.get_index_of_primitive(Type::Int));
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
        dbg!(&tys, unioned.iter().collect::<Vec<_>>());
        match unioned.len() {
            0 => self.arena.get_index_of_primitive(Type::Any), // Any
            1 => unioned.drain().nth(0).unwrap(),
            _ => self.arena.insert(Type::Union(Union {
                name_hints: {
                    let mut hints = HashSet::new();
                    hints.insert(String::from("UnnamedUnion"));
                    hints
                },
                types: unioned,
            })),
        }
    }
}
