use indexmap::IndexMap;

use itertools::{multipeek, Itertools};
use serde::{Deserialize, Serialize};

use crate::schema::{ArenaIndex, ITypeArena, Map, Schema, Type, Union};
use std::{
    collections::HashSet,
    fmt::{self, Display, Write},
};

use super::{with_context, Contexted, Indentation, TargetGenerator};

#[derive(Clone, Copy, Debug)]
struct Context<'c>(&'c Schema, &'c PythonClass);

#[derive(Debug, Serialize, Deserialize)]
pub struct PythonClass {
    pub kind: Kind,
    pub to_generate_type_alias_for_union: bool,
    // pub use_pydantic_datamodel: bool,
    pub indentation: Indentation,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
/// Sub-target for Python type definitions generator
pub enum Kind {
    /// Use `dataclass` from built-in `dataclasses` module as the decorator
    Dataclass,
    /// Use `dataclass` from built-in `dataclasses` module as the decorator, additionally
    /// decorating with the external library `dataclass-json` for JSON (de)serilization support
    DataclassWithJSON,
    /// Use `BaseModel` from the external data validation framework [`pydantic`](https://pydantic-docs.helpmanual.io/)
    /// as the base class
    PydanticBaseModel,
    /// Use [`dataclass` from pydantic](https://pydantic-docs.helpmanual.io/usage/dataclasses/) as
    /// the decorator
    PydanticDataclass,
    // /// Use `TypedDict` from the built-in `typing` module as the base class
    // TypedDict,
    // /// Use `TypedDict` from the built-in `typing` module as the base class with all sub classes
    // /// nested into the root one
    // NestedTypedDict,
}

#[typetag::serde]
impl TargetGenerator for PythonClass {
    fn write_output(
        &self,
        schema: &Schema,
        header: &mut dyn Write,
        body: &mut dyn Write,
        additional: &mut dyn Write,
    ) -> fmt::Result {
        write_output(schema, self, header, body, additional)
    }
}

#[inline(always)]
fn write_output(
    schema: &Schema,
    options: &PythonClass,
    header: &mut dyn Write,
    body: &mut dyn Write,
    _additional: &mut dyn Write,
) -> fmt::Result {
    let wrapper = with_context((), Context(schema, options)); // helper

    let decorators = match options.kind {
        Kind::Dataclass | Kind::PydanticDataclass => "@dataclass\n",
        Kind::DataclassWithJSON => "@dataclass_json\n@dataclass\n",
        _ => "",
    };
    let base_class = match options.kind {
        Kind::PydanticBaseModel => "(BaseModel)",
        _ => "",
    };

    let mut imports_from_typing = HashSet::new();
    let mut importing_base_class_or_class_decorators = false;
    let mut importing_datetime = false;
    let mut importing_uuid = false;

    for r#type in schema
        .iter_topdown()
        .map(|arni| schema.arena.get(arni).unwrap())
    {
        match *r#type {
            Type::Map(Map {
                /* ref name_hints, */
                ref fields,
                ..
            }) => {
                importing_base_class_or_class_decorators = true;
                fields
                    .iter()
                    .map(|(_, &r#type)| schema.arena.get(r#type).unwrap())
                    .for_each(|r#type| match *r#type {
                        Type::Any => {
                            imports_from_typing.insert("Any");
                        }
                        Type::Date => importing_datetime = true,
                        Type::UUID => importing_uuid = true,
                        _ => {}
                    });
                write!(
                    body,
                    "{}class {}{}:\n{}",
                    decorators,
                    wrapper.wrap(r#type), // type name
                    base_class,           // to inherit
                    wrapper.wrap(fields)  // lines of fields and types
                )?;
                write!(body, "\n")?;
            }
            Type::Union(Union {
                /* ref name_hints, */
                ref types,
                ..
            }) => {
                let is_non_trivial = (types.len()
                    - types.contains(&schema.arena.get_index_of_primitive(Type::Null)) as usize)
                    > 1;
                if options.to_generate_type_alias_for_union && is_non_trivial {
                    imports_from_typing.insert("Union");
                    write!(body, "{} = {}", wrapper.wrap(r#type), wrapper.wrap(types))?;
                    write!(body, "\n")?;
                }
                imports_from_typing.insert(if is_non_trivial { "Union" } else { "Optional" });
            }
            Type::Array(_) => {
                imports_from_typing.insert("List");
            }
            _ => {}
        }
    }
    
    if importing_base_class_or_class_decorators {
        let import = match options.kind {
            Kind::Dataclass => "from dataclasses import dataclass",
            Kind::DataclassWithJSON => {
                "from dataclasses import dataclass\nfrom dataclasses_json import dataclass_json"
            }
            Kind::PydanticBaseModel => "from pydantic import BaseModel",
            Kind::PydanticDataclass => "from pydantic import dataclass",
        };
        write!(header, "from __future__ import annotations\n\n")?;

        write!(header, "{}\n\n", import)?;
    }
    if !imports_from_typing.is_empty() {
        write!(header, "from typing import ")?;
        imports_from_typing
            .into_iter()
            .intersperse(", ")
            .map(|e| write!(header, "{}", e))
            .collect::<fmt::Result>()?;
        write!(header, "\n\n")?;
    }
    if importing_datetime {
        write!(header, "from datatime import datetime\n\n")?;
    }
    if importing_uuid {
        write!(header, "from uuid import UUID\n\n")?;
    }
    Ok(())
}

impl<'i, 'c> Display for Contexted<&'c Type, Context<'c>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Contexted {
            inner: r#type,
            context: Context(schema, options),
        } = self;
        match r#type {
            Type::Map(Map {
                ref name_hints,
                ..
                // ref fields,
            }) => {
                // TODO: eliminate unnecessary heap allocation
                if name_hints.is_empty() {
                    write!(f, "UnnammedType{:X}", r#type as *const Type as usize)
                } else {
                    write!(f, "{}", name_hints)
                }
            }
            Type::Union(Union {
                ref name_hints,
                ref types,
            }) => {
                if options.to_generate_type_alias_for_union && {
                    let is_non_trivial = (types.len()
                        - types.contains(&schema.arena.get_index_of_primitive(Type::Null))
                            as usize)
                        > 1;
                    is_non_trivial
                } {
                    if name_hints.is_empty() {
                        write!(f, "UnnammedUnion{:X}", r#type as *const Type as usize)
                    } else {
                        write!(f, "{}Union", name_hints)
                    }
                } else {
                    let optional =
                        types.contains(&schema.arena.get_index_of_primitive(Type::Null));
                    let union = self.wrap(types);
                    if optional {
                        write!(f, "Optional[{}]", union)
                    } else {
                        union.fmt(f)
                    }
                }
            }
            Type::Array(r#type) => {
                // dbg!(r#type);
                write!(
                    f,
                    "List[{}]",
                    self.wrap(schema.arena.get(*r#type).unwrap())
                )
            }
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "str"),
            Type::Date => write!(f, "datetime"),
            Type::UUID => write!(f, "UUID"),
            Type::Null => write!(f, "None"),
            Type::Any => write!(f, "Any"),
        }
    }
}

impl<'i, 'c> Display for Contexted<&'c HashSet<ArenaIndex>, Context<'c>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Contexted {
            inner: arnis,
            context: Context(schema, _options),
        } = self;
        // NOTE: return value is a Union of variants instead of a concatenated string name hints;
        //       null is discarded here
        let mut iter = multipeek(
            arnis
                .iter()
                .cloned()
                .map(|r#type| schema.arena.get(r#type).unwrap())
                .filter(|&r#type| !r#type.is_null()),
        );
        let _ = iter.peek(); // Discard the first
        if iter.peek().is_some() {
            // Regardless of possibly discarded null, there are at least two other inner types.
            write!(f, "Union[")?;
            while let Some(r#type) = iter.next() {
                // manually intersperse
                self.wrap(r#type).fmt(f)?;
                if iter.peek().is_some() {
                    write!(f, ", ")?;
                }
            }
            write!(f, "]")
        } else {
            // Not a union anymore after dicarding Null
            self.wrap(
                iter.next()
                    .expect("The union should have at least one inner type other than Null"),
            )
            .fmt(f)
        }
    }
}

impl<'i, 'c> Display for Contexted<&'c IndexMap<String, ArenaIndex>, Context<'c>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Contexted {
            inner: fields,
            context: Context(schema, options),
        } = self;

        // NOTE: return value are lines of field_name: field_type instead of concatenated hints;
        let mut iter = fields
            .iter()
            .map(|(key, &r#type)| (key, schema.arena.get(r#type).unwrap()));
        // .peekable();
        while let Some((key, r#type)) = iter.next() {
            // // manually intersperse
            write!(f, "{}{}: {}", options.indentation, key, self.wrap(r#type))?;
            // if iter.peek().is_none() {
            write!(f, "\n")?;
            // }
        }
        Ok(())
    }
}
