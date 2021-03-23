use serde_json::Value;

use super::*;
use crate::generation::{
    Indentation, PythonClass, PythonKind, PythonTypedDict, Quote, TargetGenerator,
};
#[test]
fn test_quicktype() {
    let data = include_str!("../../tests/data/quicktype.json");
    let now = std::time::Instant::now();
    let v: Value = serde_json::from_str(data).unwrap();

    println!("{}", now.elapsed().as_millis());
    let mut schema = infer(&v, None);
    println!("{}", now.elapsed().as_millis());
    dbg!(&schema);
    Optimizer {
        merging_similar_datatypes: true,
        merging_same_unions: true,
    }
    .optimize(&mut schema);
    println!("{}", now.elapsed().as_millis());
    dbg!(&schema);
    let output = PythonTypedDict {
        quote_type: Quote::Double,
        generate_type_alias_for_union: true,
        nesting_when_possible: true,
        mark_optional_as_not_total: false,
    }
    .generate(&mut schema);
    println!("{}", output.header);
    println!("{}", output.body);
    println!("{}", now.elapsed().as_millis());
}

#[test]
fn test_githubstatus() {
    let data = include_str!("../../tests/data/githubstatus.json");
    let v: Value = serde_json::from_str(data).unwrap();

    let mut schema = infer(&v, None);
    Optimizer {
        merging_similar_datatypes: true,
        merging_same_unions: true,
    }
    .optimize(&mut schema);
    let _output = PythonClass {
        kind: PythonKind::Dataclass,
        generate_type_alias_for_union: false,
        indentation: Indentation::Space(4),
    }
    .generate(&mut schema);
}

#[test]
fn test_tree_recursion() {
    let data = include_str!("../../tests/data/tree-recursion.json");
    let v: Value = serde_json::from_str(data).unwrap();

    let mut schema = infer(&v, None);
    Optimizer {
        merging_similar_datatypes: true,
        merging_same_unions: true,
    }
    .optimize(&mut schema);
    let _output = PythonClass {
        kind: PythonKind::Dataclass,
        generate_type_alias_for_union: false,
        indentation: Indentation::Space(4),
    }
    .generate(&mut schema);
}
