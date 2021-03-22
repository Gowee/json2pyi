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

#[typetag::serde(tag = "target")]
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
struct Wrapped<'s, 'g, I, G: TargetGenerator> {
    inner: I,
    schema: &'s Schema,
    options: &'g G,
}

impl<'s, 'g, I, G: TargetGenerator> Wrapped<'s, 'g, I, G> {
    /// Wrap another type using the schema and the generator options of the current wrapper
    fn wrap<OtherI>(&self, another: OtherI) -> Wrapped<'s, 'g, OtherI, G> {
        wrap(another, self.schema, self.options)
    }
}

/// Create and return a new [`WrappedType`]
fn wrap<'s, 'g, I, G: TargetGenerator>(
    inner: I,
    schema: &'s Schema,
    options: &'g G,
) -> Wrapped<'s, 'g, I, G> {
    Wrapped {
        inner,
        schema,
        options,
    }
}
