use serde::{Deserialize, Serialize};

use std::fmt::{self, Display, Write};

use crate::schema::Schema;

mod python_class;
pub use python_class::{Kind as PythonKind, PythonClass};
// mod rust;
mod python_inline;
pub use python_inline::PythonTypedDict;

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

// #[typetag::serde(tag = "target")]
pub trait TargetGenerator {
    fn generate(&self, schema: &Schema) -> GenOutput {
        let mut header = String::new();
        let mut body = String::new();
        let mut additional = String::new();
        self.write_output(schema, &mut header, &mut body, &mut additional)
            .unwrap();
        GenOutput {
            header,
            body,
            additional,
        }
    }

    fn write_output(
        &self,
        schema: &Schema,
        header: &mut dyn Write,
        body: &mut dyn Write,
        additional: &mut dyn Write,
    ) -> fmt::Result;
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

#[derive(Debug, Serialize, Deserialize)]
pub enum Quote {
    Single,
    Double,
}

impl Display for Quote {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Quote::Single => r#"'"#,
            Quote::Double => r#"""#,
        }
        .fmt(f)
    }
}

/// A helper type that facilitate taking advantage of [`Display`](std::fmt::Display)
struct Contexted<I, C: Copy> {
    inner: I,
    context: C,
}

impl<I, C: Copy> Contexted<I, C> {
    /// Wrap another type using the schema and the generator options of the current wrapper
    fn wrap<OtherI>(&self, another: OtherI) -> Contexted<OtherI, C> {
        with_context(another, self.context)
    }
}

/// Create and return a new [`WrappedType`]
fn with_context<I, C: Copy>(inner: I, context: C) -> Contexted<I, C> {
    Contexted { inner, context }
}

// trait IContext<C> {
//     fn with<I>(self, inner: I) -> Contexted<I, Self> {
//         with_context(inner, self)
//     }
// }
