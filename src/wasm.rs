use wasm_bindgen::prelude::*;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::inferrer::*;
use crate::target::{
    GenOutput, Indentation, PythonClass, PythonKind, PythonTypedDict, Quote, TargetGenerator,
};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// #[wasm_bindgen]
// extern {
//     fn alert(s: &str);
// }

#[wasm_bindgen]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Target {
    Dataclass,
    DataclassWithJSON,
    PydanticBaseModel,
    PydanticDataclass,
    TypedDict,
    NestedTypedDict,
}

#[wasm_bindgen]
pub fn json2type(json: &str, target: Target) -> Option<String> {
    let v: Value = serde_json::from_str(json).ok()?;
    let mut schema = infer_from_json(&v, None);
    Optimizer {
        to_merge_similar_datatypes: true,
        to_merge_same_unions: true,
    }
    .optimize(&mut schema);

    let target: &dyn TargetGenerator = match target {
        Target::Dataclass => &PythonClass {
            kind: PythonKind::Dataclass,
            to_generate_type_alias_for_union: true,
            indentation: Indentation::Space(4),
        },
        Target::DataclassWithJSON => &PythonClass {
            kind: PythonKind::DataclassWithJSON,
            to_generate_type_alias_for_union: true,
            indentation: Indentation::Space(4),
        },
        Target::PydanticBaseModel => &PythonClass {
            kind: PythonKind::PydanticBaseModel,
            to_generate_type_alias_for_union: true,
            indentation: Indentation::Space(4),
        },
        Target::PydanticDataclass => &PythonClass {
            kind: PythonKind::PydanticDataclass,
            to_generate_type_alias_for_union: true,
            indentation: Indentation::Space(4),
        },
        Target::TypedDict => &PythonTypedDict {
            quote_type: Quote::Double,
            to_generate_type_alias_for_union: true,
            to_nest_when_possible: false,
            to_mark_optional_as_not_total: false,
        },
        Target::NestedTypedDict => &PythonTypedDict {
            quote_type: Quote::Double,
            to_generate_type_alias_for_union: true,
            to_nest_when_possible: true,
            to_mark_optional_as_not_total: false,
        },
    };
    let GenOutput {
        header,
        body,
        additional,
    } = target.generate(&schema);
    Some([&header, &body, &additional].iter().cloned().join("\n"))
}

// use crate::generation::TargetLang;
// struct Options {
//     target: TargetLang,
//     merge_similar_maps: bool,
//     to_generate_type_alias_for_union: bool,
//     merge_same_unions: bool,
//     root_type_name: String,
// }

// pub fn infer_and_generate() -> String {

// }
