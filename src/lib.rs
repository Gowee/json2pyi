macro_rules! for_wasm {
    ($($item:item)*) => {$(
        #[cfg(target_arch = "wasm32")]
        $item
    )*}
}
// https://github.com/seanmonstar/reqwest/blob/29b15cb1d2ed59db3b57d6a5ff98236435efc9cd/src/lib.rs#L204

pub mod inferrer;
pub mod schema;
pub mod target;

for_wasm! {
    mod wasm;
}

#[cfg(test)]
mod tests;
