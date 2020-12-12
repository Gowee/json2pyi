use bidirectional_map::Bimap;
use disjoint_sets::UnionFind;
use itertools::Itertools;

use std::collections::{HashMap, HashSet};
use std::mem;
use std::ops::{Deref, DerefMut, Drop};

use super::Unioner;
use crate::schema::{ArenaIndex, ArenaOfType, ITypeArena, Map, Schema, Type, TypeArena};

pub struct HeuristicInferrer {
    pub merging_similar_datatypes: bool,
    pub merging_similar_unions: bool,
}

impl HeuristicInferrer {
    pub fn optimize(&self, schema: &mut Schema) {
        let disjoint_sets = schema.arena.get_sets_of_similar_maps();
        let mut arena = TypeArenaWithDSU::from_type_arena(&mut schema.arena);
        // let mut to_replace = HashMap::<ArenaIndex, ArenaIndex>::new();
        {
            for (leader, mut set) in disjoint_sets.into_iter() {
                set.insert(leader); // leader in disjoint set is now a follower

                let compact_set = set
                    .iter()
                    .cloned()
                    .filter(|&r#type| arena.contains(r#type))
                    .collect::<Vec<ArenaIndex>>();
                // dbg!(&compact_set
                //     .iter()
                //     .map(|&arni| arena.get(arni).unwrap())
                //     .collect::<Vec<&Type>>());
                let mut unioner = Unioner::new(&mut arena);
                // unioned is now the new leader
                let leader = unioner.runion(compact_set);
                // for follower in set.into_iter() {
                //     to_replace.insert(follower, leader);
                // }

                // union set and update all reference
                // schema.arena.get_mut(primary)
            }
            // drop unioner to release arena and primitive_types
        }
        dbg!(&arena);
        arena.flatten();

        // // dbg!(&schema);
        // let mut ufnodes: HashMap<ArenaIndex, UnionFind<ArenaIndex>> = Default::default();
        // let arena_indices: Vec<ArenaIndex> = schema.arena.iter().map(|(index, _)| index).collect();
        // let mut dsu = UnionFind::<usize>::new(arena_indices.len());

        // for (dsui, arni) in arena_indices.iter().cloned().enumerate() {
        //     for (dsuj, arnj) in arena_indices.iter().skip(dsui + 1).cloned().enumerate() {
        //         let typei = schema.arena.get(arni).unwrap();
        //         let typej = schema.arena.get(arnj).unwrap();
        //         if typei.is_map() && typej.is_map() {
        //             if typei
        //                 .as_map()
        //                 .unwrap()
        //                 .is_similar_to(typej.as_map().unwrap())
        //             {
        //                 dsu.union(dsui, dsuj);
        //             }
        //         }
        //     }
        // }

        // let indices_arena: HashMap<ArenaIndex, usize> = arena_indices
        //     .iter()
        //     .cloned()
        //     .enumerate()
        //     .map(|(a, b)| (b, a))
        //     .collect();

        // let mut disjoint_sets = HashMap::<ArenaIndex, HashSet<ArenaIndex>>::new(); // disjoint sets
        // for (arni, r#type) in schema.arena.iter() {
        //     if r#type.is_map() {
        //         let p = dsu.find(indices_arena[&arni]);
        //         // if p == indices_arena[&arni] {
        //         disjoint_sets
        //             .entry(arena_indices[p])
        //             .or_default()
        //             .insert(arni);
        //         // }
        //         // types_to_drop.insert(ari, mem::take(r#type));
        //     }
        // }
        // // // dbg!("ds", &disjoint_sets);

        // let mut to_replace = HashMap::<ArenaIndex, ArenaIndex>::new();
        // {
        //     for (leader, mut set) in disjoint_sets.into_iter() {
        //         set.insert(leader); // leader in disjoint set is now a follower

        //         let compact_set = set
        //             .iter()
        //             .cloned()
        //             .filter(|&r#type| schema.arena.contains(r#type))
        //             .collect::<Vec<ArenaIndex>>();
        //         let mut unioner = Unioner::new(&mut schema.arena, &schema.primitive_types);
        //         // unioned is now the new leader
        //         // // dbg!("merging: ",&set,  &compact_set);
        //         let leader = unioner.runion(compact_set);
        //         for follower in set.into_iter() {
        //             to_replace.insert(follower, leader);
        //         }

        //         // union set and update all reference
        //         // schema.arena.get_mut(primary)
        //     }
        //     // drop unioner to release arena and primitive_types
        // }

        // for (_arni, r#type) in schema.arena.iter_mut() {
        //     match *r#type {
        //         Type::Map(ref mut map) => {
        //             for (_, r#type) in map.fields.iter_mut() {
        //                 if to_replace.contains_key(r#type) {
        //                     *r#type = to_replace[r#type];
        //                 } else {
        //                     // assert!(schema
        //                     //     .primitive_types
        //                     //     .iter()
        //                     //     .cloned()
        //                     //     .collect::<HashSet<ArenaIndex>>()
        //                     //     .contains(r#type));
        //                 }
        //             }
        //         }
        //         Type::Union(ref mut union) => {
        //             union.types = union
        //                 .types
        //                 .iter()
        //                 .map(|r#type| to_replace[r#type])
        //                 .collect();
        //         }
        //         // primitive types obviously requires no handling
        //         // array should have its inner union already handled by the above match arm
        //         _ => (),
        //     }
        // }

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

#[derive(Debug)]
pub struct TypeArenaWithDSU<'a> {
    arena: &'a mut TypeArena,
    dsu: UnionFind<usize>,
    imap: Bimap<usize, ArenaIndex>,
}

impl<'a> TypeArenaWithDSU<'a> {
    fn from_type_arena(arena: &'a mut TypeArena) -> Self {
        // map from DSU index to ArenaIndex
        let imap: Bimap<usize, ArenaIndex> =
            Bimap::from_hash_map(arena.iter().map(|(index, _)| index).enumerate().collect());

        let dsu = UnionFind::<usize>::new(imap.len());

        // map from ArenaIndex to DSU index
        // let indices_arena: HashMap<ArenaIndex, usize> = arena_indices
        //     .iter()
        //     .cloned()
        //     .enumerate()
        //     .map(|(a, b)| (b, a))
        //     .collect();
        TypeArenaWithDSU { arena, dsu, imap }
    }

    /// Find a representative `Type`, that is the leader of the disjoint set to which `arni` belongs
    fn find_representative(&self, arni: ArenaIndex) -> Option<ArenaIndex> {
        self.imap
            .get_rev(&arni)
            .and_then(|&dsui| self.imap.get_fwd(&self.dsu.find(dsui)))
            .cloned()
    }

    /// Replace all references to non-representative `ArenaIndex` in the `TypeArena` with the
    /// representative one in the DSU. This method is invoked automatically upon dropping to ensure
    /// the released `TypeArena` has all its references consistent.
    fn flatten(&mut self) {
        // dbg!(&self);
        let mut dangling_types = HashSet::new();

        // let arnis: Vec<ArenaIndex> = self.imap.iter().map(|(_, &arni)| arni).collect();
        let arnis: Vec<ArenaIndex> = self.arena.iter().map(|(arni, _)| arni).collect();

        // <del>Only check maps in DSU, as there are newly added types during unioning.</del>
        // Maps not
        for arni in arnis {
            let arnr = self.find_representative(arni);
            if arnr.is_some() && arnr.unwrap() != arni {
                // If it is not a new type (already in the DSU before) and it is non-representative.
                // TODO: shoule replace inner type references in a representative type? // FIX
                dangling_types.insert(arni);
            } 
                // Unions might be removed during unioning. So if a representative type is not
                // there anymore, just ignore it for now.
                if let Some(r#type) = self.get_mut(arni) {
                    if r#type.is_map() {
                        // Take the map out and put it back to circumvent borrow rule limitation
                        let mut map = mem::take(r#type).into_map().unwrap();
                        for (_, r#type) in map.fields.iter_mut() {
                            // If this field in in DSU, then replace it with the leader in the DS.
                            if let Some(arnr) = self.find_representative(*r#type) {
                                *r#type = arnr;
                            }
                            // O.W., it might be new a type during unioning, requiring no action.
                        }
                        *self.get_mut(arni).unwrap() = Type::Map(map);
                    // let r = arena.find_representative(arni).unwrap();
                    // if p == indices_arena[&arni] {
                    // disjoint_sets
                    //     .entry(r)
                    //     .or_default()
                    //     .insert(arni);
                    // }
                    // types_to_drop.insert(ari, mem::take(r#type));
                    } else if r#type.is_union() {
                        let mut union = mem::take(r#type).into_union().unwrap();
                        dbg!(&union);
                        union.types = union
                            .types
                            .into_iter()
                            .map(|arni| self.find_representative(arni).unwrap_or(arni))
                            .collect();
                        dbg!(&union);
                        *self.get_mut(arni).unwrap() = Type::Union(union);
                        // let mut union = mem::take(r#type).into_union().unwrap();
                        // *self.get_mut(arni).unwrap() = Type::Union(
                        //     union
                        //         .types
                        //         .into_iter()
                        //         .map(|r#type| self.find_representative(r#type).unwrap()),
                        // );
                    } else if r#type.is_array() {
                        let inner = mem::take(r#type).into_array().unwrap();
                        if self.get_mut(arni).is_none() {
                            unreachable!();
                        }
                        if self.find_representative(inner).is_none() {
                            // continue;
                        }
                        dbg!(inner);
                        dbg!( self.find_representative(inner));
                        dbg!(arni);
                        dbg!(self.get_mut(arni));
                        *self.get_mut(arni).unwrap() = Type::Array( self.find_representative(inner).unwrap());
                    }
                }
            
        }
        for r#type in dangling_types.into_iter() {
            // TODO: Should these all removed during unioning?
            println!("removed dangling: {:?}", r#type);
            assert!(self.arena.remove(r#type).is_none());
        }
    }

    #[inline(always)]
    fn contains(&self, i: ArenaIndex) -> bool {
        // TODO: where to pput?
        Deref::deref(self).contains(i)
    }
}

impl<'a> Deref for TypeArenaWithDSU<'a> {
    type Target = ArenaOfType;

    fn deref(&self) -> &Self::Target {
        &self.arena
    }
}

impl<'a> DerefMut for TypeArenaWithDSU<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.arena
    }
}

impl<'a> ITypeArena for TypeArenaWithDSU<'a> {
    #[inline(always)]
    fn get(&self, i: ArenaIndex) -> Option<&Type> {
        let t = self
            .find_representative(i) // if it is in the DSU
            .or(Some(i)) // O.W. it should be a newly added type during unioning
            .and_then(|arni| self.arena.get(arni));
        // dbg!(i, &t, self.find_representative(i));
        t // TODO: clean up
    }

    #[inline(always)]
    fn get_mut(&mut self, i: ArenaIndex) -> Option<&mut Type> {
        let t = self
            .find_representative(i) // if it is in the DSU
            .or(Some(i)) // O.W. it should be a newly added type during unioning
            .and_then(move |arni| self.arena.get_mut(arni));
        // dbg!(i, &t);
        t
        // FIX: borrowing issue
    }

    #[inline(always)]
    fn remove(&mut self, i: ArenaIndex) -> Option<Type> {
        // Note: It is not removed from DSU. So just ignore non-existing types when iterating DSU.
        //       As get/get_mut wraps DSU internally, unioner won't get panicked.
        DerefMut::deref_mut(self).remove(i)
    }

    fn remove_in_favor_of(&mut self, i: ArenaIndex, j: ArenaIndex) -> Option<Type> {
        // dbg!("rifo", i, j);
        self.dsu.union(
            *self.imap.get_rev(&i).unwrap(),
            *self.imap.get_rev(&j).unwrap(),
        );
        DerefMut::deref_mut(self).remove(i)
    }

    #[inline(always)]
    fn insert(&mut self, value: Type) -> ArenaIndex {
        // Note: The DSU is not updated.
        DerefMut::deref_mut(self).insert(value)
    }

    #[inline(always)]
    fn get_primitive_types(&self) -> &[ArenaIndex; 6] {
        self.arena.get_primitive_types()
    }
}

impl<'a> Drop for TypeArenaWithDSU<'a> {
    fn drop(&mut self) {
        self.flatten()
    }
}
