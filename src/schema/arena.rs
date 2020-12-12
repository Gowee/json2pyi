use generational_arena::Arena;
pub use generational_arena::Index as ArenaIndex;
pub type ArenaOfType = Arena<Type>;
use bidirectional_map::Bimap;
use disjoint_sets::UnionFind;
use itertools::Itertools;

use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

use super::Type;

#[derive(Debug)]
pub struct TypeArena {
    arena: ArenaOfType,
    primitive_types: [ArenaIndex; 6],
}

impl TypeArena {
    pub fn new() -> Self {
        let mut arena = ArenaOfType::new();
        let primitive_types = [
            arena.insert(Type::Int),
            arena.insert(Type::Float),
            arena.insert(Type::Bool),
            arena.insert(Type::String),
            arena.insert(Type::Null),
            arena.insert(Type::Any),
        ];
        TypeArena {
            arena,
            primitive_types,
        }
    }

    /// Get disjoint sets of similar types.
    pub fn get_sets_of_similar_maps(&self) -> HashMap<ArenaIndex, HashSet<ArenaIndex>> {
        let imap: Bimap<usize, ArenaIndex> = Bimap::from_hash_map(
            self.arena
                .iter()
                .map(|(index, _)| index)
                .enumerate()
                .collect(),
        );
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
            if typei.is_map()
                && typej.is_map()
                && typei
                    .as_map()
                    .unwrap()
                    .is_similar_to(typej.as_map().unwrap())
            {
                Some((dsui, dsuj))
            } else {
                None
            }
        })
        .for_each(|(dsui, dsuj)| {
            dsu.union(dsui, dsuj);
        });

        let mut disjoint_sets = HashMap::<ArenaIndex, HashSet<ArenaIndex>>::new(); // disjoint sets
        for (arni, r#type) in self.arena.iter() {
            if r#type.is_map() {
                let r = imap
                    .get_rev(&arni)
                    .and_then(|&dsui| imap.get_fwd(&dsu.find(dsui)))
                    .cloned()
                    .unwrap();
                // if p == indices_arena[&arni] {
                disjoint_sets.entry(r).or_default().insert(arni);
                // }
                // types_to_drop.insert(ari, mem::take(r#type));
            }
        }
        // dbg!("ds", &disjoint_sets);
        disjoint_sets
    }
}

impl Deref for TypeArena {
    type Target = ArenaOfType;

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
    fn get_primitive_types(&self) -> &[ArenaIndex; 6];

    fn get_index_of_primitive(&self, r#type: Type) -> ArenaIndex {
        let primitive_types = self.get_primitive_types();
        match r#type {
            Type::Int => primitive_types[0],
            Type::Float => primitive_types[1],
            Type::Bool => primitive_types[2],
            Type::String => primitive_types[3],
            Type::Null => primitive_types[4],
            Type::Any => primitive_types[5],
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
    fn get_primitive_types(&self) -> &[ArenaIndex; 6] {
        &self.primitive_types
    }
}