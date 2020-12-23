pub use generational_arena::Arena;
pub use generational_arena::Index as ArenaIndex;
// pub type Arena<Type> = Arena<Type>;
use bidirectional_map::Bimap;
use disjoint_sets::UnionFind;
use itertools::Itertools;

use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
};

use super::Type;

#[derive(Debug)]
pub struct TypeArena {
    arena: Arena<Type>,
    primitive_types: [ArenaIndex; 8],
}

impl TypeArena {
    pub fn new() -> Self {
        let mut arena = Arena::<Type>::new();
        let primitive_types = [
            arena.insert(Type::Int),
            arena.insert(Type::Float),
            arena.insert(Type::Bool),
            arena.insert(Type::String),
            arena.insert(Type::Date),
            arena.insert(Type::UUID),
            arena.insert(Type::Null),
            arena.insert(Type::Any),
        ];
        TypeArena {
            arena,
            primitive_types,
        }
    }

    /// Get disjoint sets of similar types.
    pub fn find_disjoint_sets<F>(
        &self,
        should_union_fn: F,
    ) -> HashMap<ArenaIndex, HashSet<ArenaIndex>>
    where
        F: Fn(&Type, &Type) -> bool,
    {
        // The map between ArenaIndex and its index of type usize in the DSU
        let imap: Bimap<usize, ArenaIndex> = Bimap::from_hash_map(
            self.arena
                .iter()
                .map(|(index, _)| index)
                .enumerate()
                .collect(),
        );
        // Disjoint set union
        let mut dsu = UnionFind::<usize>::new(imap.len());
        {
            let iter1 = imap.fwd().iter().map(|(&a, &b)| (a, b));
            let iter2 = iter1.clone();
            iter1.cartesian_product(iter2)
        }
        .filter(|(left, right)| left != right)
        .filter_map(|((dsui, arni), (dsuj, arnj))| {
            let typei = self.arena.get(arni).unwrap();
            let typej = self.arena.get(arnj).unwrap();

            if should_union_fn(typei, typej) {
                Some((dsui, dsuj))
            } else {
                None
            }
        })
        .for_each(|(dsui, dsuj)| {
            // For every pair of different types in the arena, union it if should_union_fn gives true.
            dsu.union(dsui, dsuj);
        });

        // Result sets
        // TODO: Or just return HashSet here?
        let mut disjoint_sets = HashMap::<ArenaIndex, HashSet<ArenaIndex>>::new();
        for (arni, _type) in self.arena.iter() {
            let r = imap
                .get_rev(&arni)
                .and_then(|&dsui| imap.get_fwd(&dsu.find(dsui)))
                .cloned()
                .unwrap();
            disjoint_sets.entry(r).or_default().insert(arni);
        }
        disjoint_sets
    }
}

impl Deref for TypeArena {
    type Target = Arena<Type>;

    fn deref(&self) -> &Self::Target {
        &self.arena
    }
}

impl DerefMut for TypeArena {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.arena
    }
}

pub trait ITypeArena {
    fn get(&self, i: ArenaIndex) -> Option<&Type>;
    fn get_mut(&mut self, i: ArenaIndex) -> Option<&mut Type>;
    fn remove(&mut self, i: ArenaIndex) -> Option<Type>;
    fn remove_in_favor_of(&mut self, i: ArenaIndex, j: ArenaIndex) -> Option<Type>;
    fn insert(&mut self, value: Type) -> ArenaIndex;
    fn get_primitive_types(&self) -> &[ArenaIndex; 8];

    fn get_index_of_primitive(&self, r#type: Type) -> ArenaIndex {
        let primitive_types = self.get_primitive_types();
        match r#type {
            Type::Int => primitive_types[0],
            Type::Float => primitive_types[1],
            Type::Bool => primitive_types[2],
            Type::String => primitive_types[3],
            Type::Date => primitive_types[4],
            Type::UUID => primitive_types[5],
            Type::Null => primitive_types[6],
            Type::Any => primitive_types[7],
            _ => panic!("Not a primitive type: {:?}", r#type),
        }
    }
}

impl ITypeArena for TypeArena {
    #[inline(always)]
    fn get(&self, i: ArenaIndex) -> Option<&Type> {
        Deref::deref(self).get(i)
    }

    #[inline(always)]
    fn get_mut(&mut self, i: ArenaIndex) -> Option<&mut Type> {
        DerefMut::deref_mut(self).get_mut(i)
    }

    #[inline(always)]
    fn insert(&mut self, value: Type) -> ArenaIndex {
        DerefMut::deref_mut(self).insert(value)
    }

    #[inline(always)]
    fn remove(&mut self, i: ArenaIndex) -> Option<Type> {
        DerefMut::deref_mut(self).remove(i)
    }

    #[inline(always)]
    fn remove_in_favor_of(&mut self, i: ArenaIndex, j: ArenaIndex) -> Option<Type> {
        let _ = j;
        self.remove(i)
    }

    #[inline(always)]
    fn get_primitive_types(&self) -> &[ArenaIndex; 8] {
        &self.primitive_types
    }
}
