use serde::{Deserialize, Serialize};

use std::fmt::{self, Display, Write};

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

impl Display for Indentation {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Indentation::Space(len) => {
                for _ in 0..len {
                    write!(fmt, " ")?;
                }
            }
            Indentation::Tab => {
                write!(fmt, "\t")?;
            }
        }
        Ok(())
    }
}

/// A helper type that facilitate taking advantage of [`Display`](std::fmt::Display)
struct Wrapped<'i, 's, 'g, I, G: TargetGenerator> {
    inner: &'i I,
    schema: &'s Schema,
    options: &'g G,
}

impl<'i, 's, 'g, I, G: TargetGenerator> Wrapped<'i, 's, 'g, I, G> {
    /// Wrap another type using the schema and generator options of the current wrapper
    fn wrap(&self, another: &'i I) -> Self {
        wrap(another, self.schema, self.options)
    }
}

/// Create and return a new [`WrappedType`]
fn wrap<'i, 's, 'g, I, G: TargetGenerator>(
    inner: &'i I,
    schema: &'s Schema,
    options: &'g G,
) -> Wrapped<'i, 's, 'g, I, G> {
    Wrapped {
        inner,
        schema,
        options,
    }
}
