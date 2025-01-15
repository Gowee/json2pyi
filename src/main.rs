use serde_json::Value;

use json2pyi::inferrer::*;
use json2pyi::target::{
    Indentation, PythonClass, PythonKind, PythonTypedDict, Quote, TargetGenerator,
};

fn main() {
    let data = include_str!("../tests/data/issue8.json");
    let v: Value = serde_json::from_str(data).unwrap();

    let mut schema = infer_from_json(&v, None);
    dbg!("BUMP1");
    Optimizer {
        to_merge_similar_datatypes: true,
        to_merge_same_unions: true,
    }
    .optimize(&mut schema);
    dbg!("BUMP2");
    let _output = PythonClass {
        kind: PythonKind::Dataclass,
        to_generate_type_alias_for_union: false,
        indentation: Indentation::Space(4),
    }
    .generate(&schema);
}
