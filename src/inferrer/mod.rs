mod adhoc;
mod heuristic;
mod unioner;

pub use adhoc::infer;
pub use heuristic::Optimizer;

#[cfg(test)]
mod tests;
