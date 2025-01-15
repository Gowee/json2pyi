use indexmap::IndexMap;
// /// Infer a schema from a given JSONValue
// use serde_json::Value as JSONValue;

use std::{collections::HashSet, mem};

use crate::schema::{ArenaIndex, ITypeArena, Map, NameHints, Type, Union};

/// Union a sequence of `types` into a single [`Type`] in the given `arena`
pub fn union(
    arena: &mut impl ITypeArena,
    types: impl IntoIterator<Item = ArenaIndex>,
) -> ArenaIndex {
    UnionerClosure::new(arena).union(types)
}

/// A unioner with a reference to some a arena associated
pub struct UnionerClosure<'a, T: ITypeArena> {
    arena: &'a mut T,
}

impl<'a, T: ITypeArena> UnionerClosure<'a, T> {
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
        let mut map_name_hints = NameHints::new();
        let mut first_union: Option<ArenaIndex> = None;
        let mut union_name_hints = NameHints::new();
        // All Arrays are collected at first. Then their inner types are unioned recursively.
        // e.g. `int[], (int | bool)[], string[]` -> (int | bool | string)[]
        let mut arrays = vec![];
        // TODO: keep first_array?

        // Expand any nested unions. Due to borrow issues, collecting is inevitable
        let mut types: Vec<ArenaIndex> = types.into_iter().collect();
        let mut changed = true;
        while changed {
            changed = false;
            types = types
            .into_iter()
            .flat_map(|r#type| {
                // dbg!(r#type);
                match self
                    .arena
                    .get(r#type)
                    .expect("The type should be present in the arena during unioning")
                {
                    Type::Union(_) => {
                        changed = true;
                        let Union { name_hints, types } = if let Some(first_union) = first_union {
                            dbg!(r#type, first_union);
                            dbg!(self.arena
                                .remove_in_favor_of(r#type, first_union)
                                .unwrap())
                                .into_union()
                                .unwrap() // remove & expand the union
                        } else {
                            first_union = Some(r#type);
                            mem::take(self.arena.get_mut(r#type).unwrap())
                                .into_union()
                                .unwrap()
                        };
                        union_name_hints.extend(name_hints.into_inner());
                        types.into_iter().collect::<Vec<_>>()
                    }
                    _ => vec![r#type], // TODO: avoid unnecessary Vec
                }
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        }
        for r#type in types {
            // dbg!(r#type, self.arena.get(r#type));
            match *self.arena.get(r#type).unwrap() {
                Type::Map(_) => {
                    let map;
                    if let Some(first_map) = first_map {
                        // If it is not the first map in the union, just remove the type from the
                        // arena.
                        map = self
                            .arena
                            .remove_in_favor_of(r#type, first_map)
                            .unwrap()
                            .into_map()
                            .unwrap();
                    } else {
                        // O.W., just take the inner of the first map out so that its slot can be
                        // reused again with ArenaIndex left intact.
                        first_map = Some(r#type);
                        map = mem::take(self.arena.get_mut(r#type).unwrap())
                            .into_map()
                            .unwrap();
                    }
                    let maps = maps.get_or_insert_with(Default::default);
                    for (key, r#type) in map.fields.into_iter() {
                        maps.entry(key).or_default().push(r#type);
                    }
                    map_count += 1;
                    // NOTE: For in-place HashSet union, `.extend` is needed instead of `.union`.
                    map_name_hints.extend(map.name_hints.into_inner());
                }
                Type::Array(_) => {
                    // TODO: FIX : in favor of?
                    let inner = self.arena.remove(r#type).unwrap().into_array().unwrap();
                    arrays.push(inner);
                }
                Type::Union(_) => unreachable!(), // union should have been expanded above
                Type::Undetermined => unreachable!(),
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

        // dbg!(&first_map, &maps);
        if let Some(maps) = maps {
            // merge maps recursively by unioning every possible fields
            let unioned_map: IndexMap<String, ArenaIndex> = maps
                .into_iter()
                .map(|(key, mut types)| {
                    // The field is not required if not present in every Map.
                    if types.len() < map_count {
                        types.push(self.arena.get_index_of_primitive(Type::Missing));
                        // TypeScript: ?/undefined; Python: Missing/NotRequired
                    }
                    (key, self.runion(types))
                })
                .collect();
            // dbg!(&unioned_map);
            if unioned_map.is_empty() {
                // every map is empty (no field at all)
                // TODO: Any or unit type?
                // TODO: should slot be removed from arena here?
                unioned.insert(self.arena.get_index_of_primitive(Type::Any)); // Any
            } else {
                dbg!(map_count);
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
        {
            // Mix of string-like types is treated as string
            let uuid = unioned.contains(&self.arena.get_index_of_primitive(Type::UUID));
            let datetime = unioned.contains(&self.arena.get_index_of_primitive(Type::Date));
            let string = unioned.contains(&self.arena.get_index_of_primitive(Type::String));

            if (uuid & datetime) | (string & (uuid ^ datetime)) {
                // at least two are true
                // https://stackoverflow.com/a/3090404/5488616
                unioned.remove(&self.arena.get_index_of_primitive(Type::Date));
                unioned.remove(&self.arena.get_index_of_primitive(Type::UUID));
                unioned.insert(self.arena.get_index_of_primitive(Type::String));
            }
        }

        // dbg!(&tys, unioned.iter().collect::<Vec<_>>());
        // if first_union.is_some() {
        //     //
        //     assert!(unioned.len() > 1);
        // }
        match unioned.len() {
            0 => self.arena.get_index_of_primitive(Type::Any), // Any
            1 => unioned.drain().next().unwrap(),
            _ => {
                let union = Type::Union(Union {
                    name_hints: union_name_hints,
                    types: unioned,
                });
                if let Some(slot) = first_union {
                    *self.arena.get_mut(slot).unwrap() = union;
                    slot
                } else {
                    self.arena.insert(union)
                }
            }
        }
    }
}
