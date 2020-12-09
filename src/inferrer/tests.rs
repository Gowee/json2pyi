use serde_json::Value;

use super::*;
#[test]
fn test_jvilk_maketypes() {
    let data = include_str!("../../tests/data/jvilk-maketypes.json");
    let v: Value = serde_json::from_str(data).unwrap();
    let mut schema = BasicInferrerClosure::new().infer(&v);

    HeuristicInferrer {
        merging_similar_datatypes: true,
        merging_similar_unions: true,
    }
    .optimize(&mut schema);

    println!("{:?}", schema);
}
