use disjoint_sets::UnionFind;

use std::collections::{HashMap, HashSet};
use std::mem;

use super::Unioner;
use crate::schema::{ArenaIndex, Map, Schema, Type};

pub struct HeuristicInferrer {
    pub merging_similar_datatypes: bool,
    pub merging_similar_unions: bool,
}

impl HeuristicInferrer {
    pub fn optimize(&self, schema: &mut Schema) {
        dbg!(&schema);
        // let mut ufnodes: HashMap<ArenaIndex, UnionFind<ArenaIndex>> = Default::default();
        let arena_indices: Vec<ArenaIndex> = schema.arena.iter().map(|(index, _)| index).collect();
        let mut dsu = UnionFind::<usize>::new(arena_indices.len());

        for (dsui, arni) in arena_indices.iter().cloned().enumerate() {
            for (dsuj, arnj) in arena_indices.iter().skip(dsui + 1).cloned().enumerate() {
                let typei = schema.arena.get(arni).unwrap();
                let typej = schema.arena.get(arnj).unwrap();
                if typei.is_map() && typej.is_map() {
                    if typei
                        .as_map()
                        .unwrap()
                        .is_similar_to(typej.as_map().unwrap())
                    {
                        dsu.union(dsui, dsuj);
                    }
                }
            }
        }

        let indices_arena: HashMap<ArenaIndex, usize> = arena_indices
            .iter()
            .cloned()
            .enumerate()
            .map(|(a, b)| (b, a))
            .collect();

        let mut disjoint_sets = HashMap::<ArenaIndex, HashSet<ArenaIndex>>::new(); // disjoint sets
        for (arni, r#type) in schema.arena.iter() {
            if r#type.is_map() {
                let p = dsu.find(indices_arena[&arni]);
                // if p == indices_arena[&arni] {
                    disjoint_sets
                        .entry(arena_indices[p])
                        .or_default()
                        .insert(arni);
                // }
                // types_to_drop.insert(ari, mem::take(r#type));
            }
        }
        dbg!("ds", &disjoint_sets);

        let mut to_replace = HashMap::<ArenaIndex, ArenaIndex>::new();
        {
            for (leader, mut set) in disjoint_sets.into_iter() {
                set.insert(leader); // leader in disjoint set is now a follower

                let compact_set = set
                    .iter()
                    .cloned()
                    .filter(|&r#type| schema.arena.contains(r#type))
                    .collect::<Vec<ArenaIndex>>();
                let mut unioner = Unioner::new(&mut schema.arena, &schema.primitive_types);
                // unioned is now the new leader
                dbg!("merging: ",&set,  &compact_set);
                let leader = unioner.runion(compact_set);
                for follower in set.into_iter() {
                    to_replace.insert(follower, leader);
                }

                // union set and update all reference
                // schema.arena.get_mut(primary)
            }
            // drop unioner to release arena and primitive_types
        }

        for (_arni, r#type) in schema.arena.iter_mut() {
            match *r#type {
                Type::Map(ref mut map) => {
                    for (_, r#type) in map.fields.iter_mut() {
                        if to_replace.contains_key(r#type) {
                            *r#type = to_replace[r#type];
                        } else {
                            // assert!(schema
                            //     .primitive_types
                            //     .iter()
                            //     .cloned()
                            //     .collect::<HashSet<ArenaIndex>>()
                            //     .contains(r#type));
                        }
                    }
                }
                Type::Union(ref mut union) => {
                    union.types = union
                        .types
                        .iter()
                        .map(|r#type| to_replace[r#type])
                        .collect();
                }
                // primitive types obviously requires no handling
                // array should have its inner union already handled by the above match arm
                _ => (),
            }
        }

        // for (ari, r#type) in schema.arena.iter_mut() {
        //     if r#type.is_map() {

        //     }
        // }

        // for
        // for (index1, type1) in schema.arena.iter().skip(schema.primitive_types.len()) {
        //     let node1 = ufnodes.entry(index1).or_insert_with(|| UnionFind::new(index1));
        //     for (index2, type2) in schema.arena.iter().skip(schema.primitive_types.len()) {
        //         if index1 != index2 && type1.is_map() && type2.is_map() {
        //             if type1
        //                 .as_map()
        //                 .unwrap()
        //                 .is_similar_to(type2.as_map().unwrap())
        //             {
        //                 let node2 = ufnodes.entry(index1).or_insert_with(|| UnionFind::new(index1));
        //                 node1.union(node2);
        //             }
        //         }
        //     }
        // }
    }
}
