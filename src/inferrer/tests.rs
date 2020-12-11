use serde_json::Value;

use super::*;
#[test]
fn test_jvilk_maketypes() {
    let data = include_str!("../../tests/data/quicktype.json");
    let v: Value = serde_json::from_str(data).unwrap();
    let mut schema = BasicInferrerClosure::new().infer(&v);
    dbg!(&schema);
    HeuristicInferrer {
        merging_similar_datatypes: true,
        merging_similar_unions: true,
    }
    .optimize(&mut schema);

    dbg!(schema);
}
