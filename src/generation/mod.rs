use serde::{Deserialize, Serialize};

use crate::schema::{ArenaIndex, Map, Schema, Type, Union};

mod python_dataclasses;
pub use python_dataclasses::PythonDataclasses;

// pub use dataclasses::*;

// pub enum TargetLang {
//     PythonDataclasses(dataclasses::Options),
//     PythonTypedDict,
//     PythonPydantic,
//     RustSerde,
//     TypeScriptInterface
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct GenOutput {
    pub header: String,
    pub body: String,
    pub additional: String,
}

// #[typetag::serde(tag = "target")]
// pub trait TargetGenerator {
//     fn generate();
// }

#[typetag::serde(tag = "type")]
pub trait TargetGenerator {
    fn generate(&self, schema: &Schema) -> GenOutput;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Indentation {
    Space(usize),
    Tab,
}
