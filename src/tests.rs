use serde_json::Value;

use crate::inferrer::*;
use crate::target::{
    Indentation, PythonClass, PythonKind, PythonTypedDict, Quote, TargetGenerator,
};

#[test]
fn test_quicktype() {
    let data = include_str!("../tests/data/quicktype.json");
    let now = std::time::Instant::now();
    let v: Value = serde_json::from_str(data).unwrap();

    println!("{}", now.elapsed().as_millis());
    let mut schema = infer_from_json(&v, None);
    println!("{}", now.elapsed().as_millis());
    dbg!(&schema);
    Optimizer {
        to_merge_similar_datatypes: true,
        to_merge_same_unions: true,
    }
    .optimize(&mut schema);
    println!("{}", now.elapsed().as_millis());
    dbg!(&schema);
    let output = PythonTypedDict {
        quote_type: Quote::Double,
        to_generate_type_alias_for_union: true,
        to_nest_when_possible: true,
    }
    .generate(&schema);
    println!("{}", output.header);
    println!("{}", output.body);
    println!("{}", now.elapsed().as_millis());
}

#[test]
fn test_githubstatus() {
    let data = include_str!("../tests/data/githubstatus.json");
    let v: Value = serde_json::from_str(data).unwrap();

    let mut schema = infer_from_json(&v, None);
    Optimizer {
        to_merge_similar_datatypes: true,
        to_merge_same_unions: true,
    }
    .optimize(&mut schema);
    let _output = PythonClass {
        kind: PythonKind::Dataclass,
        to_generate_type_alias_for_union: false,
        indentation: Indentation::Space(4),
    }
    .generate(&schema);
}

#[test]
fn test_tree_recursion() {
    let data = include_str!("../tests/data/tree-recursion.json");
    let v: Value = serde_json::from_str(data).unwrap();

    let mut schema = infer_from_json(&v, None);
    Optimizer {
        to_merge_similar_datatypes: true,
        to_merge_same_unions: true,
    }
    .optimize(&mut schema);
    let _output = PythonClass {
        kind: PythonKind::Dataclass,
        to_generate_type_alias_for_union: false,
        indentation: Indentation::Space(4),
    }
    .generate(&schema);
}

#[test]
fn test_issue8_1() {
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

#[test]
fn test_issue8_2() {
    let data = include_str!("../tests/data/issue8-2.json");
    let v: Value = serde_json::from_str(data).unwrap();

    let mut schema = infer_from_json(&v, None);
    Optimizer {
        to_merge_similar_datatypes: true,
        to_merge_same_unions: true,
    }
    .optimize(&mut schema);
    let _output = PythonClass {
        kind: PythonKind::Dataclass,
        to_generate_type_alias_for_union: false,
        indentation: Indentation::Space(4),
    }
    .generate(&schema);
}
