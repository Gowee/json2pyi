use generational_arena::{Arena, Index};
// trait ArenaExt {
//     fn take(&mut self) -> Option
// }

impl<T: Default> ArenaExt<T> for Arena<T> {
    fn take(&mut self, i: Index) -> T {
        self.get_mut(i)
    }
}