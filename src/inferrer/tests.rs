use serde_json::Value;

use super::*;
use crate::generation::schema_to_dataclasses;
#[test]
fn test_jvilk_maketypes() {
    let data = include_str!("../../tests/data/githubstatus.json");
    let v: Value = serde_json::from_str(data).unwrap();

    let now = std::time::Instant::now();


    let mut schema = BasicInferrerClosure::new().infer(&v);
    dbg!(&schema);
    HeuristicInferrer {
        merging_similar_datatypes: true,
        merging_similar_unions: true,
    }
    .optimize(&mut schema);
    println!("{}", now.elapsed().as_millis());
    // dbg!(&schema);
    println!("{}", schema_to_dataclasses(&mut schema));
    println!("{}", now.elapsed().as_millis());
}
