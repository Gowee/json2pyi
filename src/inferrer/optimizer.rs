use bidirectional_map::Bimap;
use disjoint_sets::UnionFind;

use std::{
    collections::{HashMap, HashSet},
    mem,
    ops::{Deref, DerefMut, Drop},
};

use super::unioner::union;
use crate::schema::{ArenaIndex, ITypeArena, Schema, Type, TypeArena};

/// A optimizer that merge similar `Map`s and/or same `Union`s as configured
pub struct Optimizer {
    pub to_merge_similar_datatypes: bool,
    pub to_merge_same_unions: bool,
}

impl Optimizer {
    pub fn new_default() -> Optimizer {
        Optimizer {
            to_merge_similar_datatypes: true,
            to_merge_same_unions: true,
        }
    }

    pub fn optimize(&self, schema: &mut Schema) {
        // <del>
        // Note: Merging maps and unions at the same time may have produced results different from
        // seperate merging (find map sets - merge - flatten - find union sets - merge - flatten).
        // For simplicity, just take the first way.
        // </del>
        // Merging maps and unions in one pass leads to the issue #8. The reason might be some
        // reentrancy issues in union (mem::replace?). TODO: figure out why
        // TODO: merge same array?
        schema.root = do_merge(
            schema,
            schema.arena.find_disjoint_sets(|a, b| {
                if let (Some(a), Some(b)) = (a.as_map(), b.as_map()) {
                    self.to_merge_similar_datatypes && a.is_similar_to(b)
                } else {
                    false
                }
            }),
        );
        schema.root = do_merge(
            schema,
            schema.arena.find_disjoint_sets(|a, b| {
                if let (Some(a), Some(b)) = (a.as_union(), b.as_union()) {
                    self.to_merge_same_unions && (a.types == b.types)
                } else {
                    false
                }
            }),
        );
    }
}

fn do_merge(schema: &mut Schema, sets: HashMap<ArenaIndex, HashSet<ArenaIndex>>) -> ArenaIndex {
    let mut ufarena = TypeArenaWithDSU::from_type_arena(&mut schema.arena);
    for (leader, mut set) in sets.into_iter() {
        set.insert(leader); // leader in disjoint set is now a follower
        if set.len() <= 1 {
            continue;
        }
        let compact_set = set
            .iter()
            .cloned()
            .filter(|&r#type| ufarena.contains(r#type))
            .collect::<Vec<ArenaIndex>>();
        // unioned is now the new leader
        let _leader = union(&mut ufarena, compact_set);
        // References to non-representative AreneIndex will be replaced automatically
        // when TypeArenaWithDSU is dropped
    }
    // Although unioner always keeps the first map slot intact, there is no guarantee that
    // root would always be the first map in types to be unioned. So update it if necessary.
    ufarena.find_representative(schema.root).unwrap()
    // arena.flatten();
}

/// A wrapper around `&mut TypeArena` with a Disjoint Set Union. `get` and `get_mut` are wrapped
/// with DSU find to be DSU-aware. `remove_in_favor_of` is wrapped with DSU union.
///
/// Upon dropping, all references to non-representative types are replaced according to the DSU.
#[derive(Debug)]
pub struct TypeArenaWithDSU<'a> {
    /// The inner arena
    arena: &'a mut TypeArena,
    /// The Disjoint Set Union structure
    dsu: UnionFind<usize>,
    /// The map from DSU index to ArenaIndex
    imap: Bimap<usize, ArenaIndex>,
}

impl<'a> TypeArenaWithDSU<'a> {
    fn from_type_arena(arena: &'a mut TypeArena) -> Self {
        let imap: Bimap<usize, ArenaIndex> =
            Bimap::from_hash_map(arena.iter().map(|(index, _)| index).enumerate().collect());

        let dsu = UnionFind::<usize>::new(imap.len());
        TypeArenaWithDSU { arena, dsu, imap }
    }

    /// Find the index of the representative `Type` which is the leader of the disjoint set to
    /// which `arni` belongs
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
        let mut dangling_types = HashSet::new();

        // There might be new types which internally references to non-representative and hence
        // non-existing types. They also need updating. So just iterate over the whole arena
        // instead of just imap which contains no newly inserted types.
        // let arnis: Vec<ArenaIndex> = self.imap.iter().map(|(_, &arni)| arni).collect();
        let arnis: Vec<ArenaIndex> = self.arena.iter().map(|(arni, _)| arni).collect();

        // <del>Only check maps in DSU, as there are newly added types during unioning.</del>
        // Maps not
        for arni in arnis {
            let arnr = self.find_representative(arni);
            if arnr.is_some() && arnr.unwrap() != arni {
                // If it is not a new type (already in the DSU before) and it is non-representative.
                //// TODO: shoule replace inner type references in a representative type? // FIX
                dangling_types.insert(arni);
            }
            //// Unions might be removed during unioning. So if a representative type is not
            //// there anymore, just ignore it for now.
            if let Some(r#type) = self.get_mut(arni) {
                if r#type.is_map() {
                    // Take the map out and put it back to circumvent borrow rule limitation
                    let mut map = mem::take(r#type).into_map().unwrap();
                    for (_, r#type) in map.fields.iter_mut() {
                        // If this field is in DSU, then replace it with the leader in the DS.
                        if let Some(arnr) = self.find_representative(*r#type) {
                            *r#type = arnr;
                        }
                        // O.W., it might be new a type during unioning, requiring no action.
                    }
                    *self.get_mut(arni).unwrap() = Type::Map(map);
                } else if r#type.is_union() {
                    let mut union = mem::take(r#type).into_union().unwrap();
                    union.types = union
                        .types
                        .into_iter()
                        .map(|arni| self.find_representative(arni).unwrap_or(arni))
                        .collect();
                    *self.get_mut(arni).unwrap() = Type::Union(union);
                } else if r#type.is_array() {
                    let inner = mem::take(r#type).into_array().unwrap();
                    *self.get_mut(arni).unwrap() =
                        Type::Array(self.find_representative(inner).unwrap_or(inner));
                }
            }
        }
        for r#type in dangling_types.into_iter() {
            // TODO: Should these all removed during unioning?
            println!("removed dangling: {:?}", r#type);
            // FIXME: temporary fix for #8
            // assert!(self.arena.remove(r#type).is_none());
        }
    }

    #[inline(always)]
    fn contains(&self, i: ArenaIndex) -> bool {
        // TODO: where to pput?
        Deref::deref(self).contains(i)
    }
}

impl<'a> Deref for TypeArenaWithDSU<'a> {
    type Target = TypeArena;

    fn deref(&self) -> &Self::Target {
        self.arena
    }
}

impl<'a> DerefMut for TypeArenaWithDSU<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.arena
    }
}

impl<'a> ITypeArena for TypeArenaWithDSU<'a> {
    #[inline(always)]
    fn get(&self, i: ArenaIndex) -> Option<&Type> {
        // dbg!(i, &t, self.find_representative(i));
        self.find_representative(i) // if it is in the DSU
            .or(Some(i)) // O.W. it should be a newly added type during unioning
            .and_then(|arni| self.arena.get(arni)) // TODO: clean up
    }

    #[inline(always)]
    fn get_mut(&mut self, i: ArenaIndex) -> Option<&mut Type> {
        // dbg!(i, &t);
        self.find_representative(i) // if it is in the DSU
            .or(Some(i)) // O.W. it should be a newly added type during unioning
            .and_then(move |arni| self.arena.get_mut(arni))
        // FIX: borrowing issue
    }

    #[inline(always)]
    fn remove(&mut self, i: ArenaIndex) -> Option<Type> {
        // Note: It is not removed from DSU. So just ignore non-existing types when iterating DSU.
        //       As get/get_mut wraps DSU internally, unioner won't get panicked.
        DerefMut::deref_mut(self).remove(i)
    }

    /// Remove the type denoted by the index i and union i into j in the DSU
    fn remove_in_favor_of(&mut self, i: ArenaIndex, j: ArenaIndex) -> Option<Type> {
        self.dsu.union(
            *self.imap.get_rev(&i).unwrap(),
            *self.imap.get_rev(&j).unwrap(),
        );
        // FIXME: temporary fix for #8
        // DerefMut::deref_mut(self).remove(i)
        DerefMut::deref_mut(self).get(i).cloned()
    }

    #[inline(always)]
    fn insert(&mut self, value: Type) -> ArenaIndex {
        debug_assert_eq!(self.dsu.len(), self.imap.len());
        let i = self.dsu.alloc();
        let arni = DerefMut::deref_mut(self).insert(value);
        self.imap.insert(i, arni);
        arni
    }

    #[inline(always)]
    fn get_primitive_types(&self) -> &[ArenaIndex; 9] {
        self.arena.get_primitive_types()
    }
}

impl<'a> Drop for TypeArenaWithDSU<'a> {
    fn drop(&mut self) {
        self.flatten()
    }
}
