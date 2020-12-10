struct TypeArena {
    arena: ArenaOfType,
    primitive_types: [ArenaIndex; 6],
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

impl TypeArena {
    fn get_index_of_primitive(&self, r#type: Type) -> ArenaIndex {
        match r#type {
            Type::Int => self.primitive_types[0],
            Type::Float => self.primitive_types[1],
            Type::Bool => self.primitive_types[2],
            Type::String => self.primitive_types[3],
            Type::Null => self.primitive_types[4],
            Type::Any => self.primitive_types[5],
            _ => panic!("Not a primitive type: {:?}", r#type)
        }
    }
}

struct TypeArenaWithDSU<'a> {
    arena: &'a mut TypeArena,

} 