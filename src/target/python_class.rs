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
    /// Use `TypedDict` from the built-in `typing` module as the base class, as explained in [PEP-589](https://www.python.org/dev/peps/pep-0589/#class-based-syntax)
    TypedDict, // TODO: totality?
}

// #[typetag::serde]
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
        Kind::TypedDict => "(TypedDict)",
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
                    "{}class {}{}:\n{}", // fields has a trailing LF
                    decorators,
                    wrapper.wrap(r#type), // type name
                    base_class,           // to inherit
                    wrapper.wrap(fields)  // lines of fields and types
                )?;
                writeln!(body)?;
            }
            Type::Union(Union {
                /* ref name_hints, */
                ref types,
                ..
            }) => {
                let is_non_trivial = (types.len()
                    - types.contains(&schema.arena.get_index_of_primitive(Type::Missing)) as usize
                    - types.contains(&schema.arena.get_index_of_primitive(Type::Null)) as usize)
                    > 1;
                if options.to_generate_type_alias_for_union && is_non_trivial {
                    writeln!(body, "{} = {}", wrapper.wrap(r#type), wrapper.wrap(types))?;
                    writeln!(body)?;
                }
                if is_non_trivial {
                    imports_from_typing.insert("Union");
                }
                if types.contains(&schema.arena.get_index_of_primitive(Type::Missing)) {
                    // per PEP 655:
                    // > It is an error to use Required[] or NotRequired[] in any location that is
                    // not an item of a TypedDict.
                    // > Such a Missing constant could also be used for other scenarios such as the
                    // type of a variable which is only conditionally defined.
                    //
                    // So we use NotRequired for TypedDict and Missing otherwise.
                    //
                    // `NotRequired[]` is invalid. So a single `Missing` is used instead.
                    imports_from_typing.insert(
                        if options.kind == Kind::TypedDict || types.len() > 1 {
                            "NotRequired"
                        } else {
                            "Missing"
                        },
                    );
                }
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
            Kind::PydanticDataclass => "from pydantic.dataclasses import dataclass",
            Kind::TypedDict => {
                imports_from_typing.insert("TypedDict");
                ""
            }
        };
        writeln!(header, "from __future__ import annotations")?;

        writeln!(header, "{}", import)?;
    }
    if !imports_from_typing.is_empty() {
        let typing_mod = if ["NotRequired", "Missing"]
            .iter()
            .any(|&t| imports_from_typing.contains(t))
        {
            "typing_extensions"
        } else {
            "typing"
        };
        write!(header, "from {} import ", typing_mod)?;
        Itertools::intersperse(imports_from_typing.into_iter(), ", ")
            .try_for_each(|e| write!(header, "{}", e))?;
        if typing_mod == "typing_extensions" {
            write!(header, " # For Python < 3.11, pip install typing_extensions; For Python >= 3.11, just change it to `typing`")?;
        }
        writeln!(header)?;
    }
    if importing_datetime {
        writeln!(header, "from datetime import datetime")?;
    }
    if importing_uuid {
        writeln!(header, "from uuid import UUID")?;
    }
    // write!(header, "\n")?;
    Ok(())
}

impl<'i, 'c> Display for Contexted<&'i Type, Context<'c>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Contexted {
            inner: r#type,
            context: Context(schema, options),
        } = self;
        match r#type {
            Type::Map(ref map) => {
                // TODO: eliminate unnecessary heap allocation
                map.fmt(f)
            }
            Type::Union(ref union) => {
                let is_non_trivial = (union.types.len()
                    - union
                        .types
                        .contains(&schema.arena.get_index_of_primitive(Type::Null))
                        as usize
                    - union
                        .types
                        .contains(&schema.arena.get_index_of_primitive(Type::Missing))
                        as usize)
                    > 1;
                let not_required = union
                    .types
                    .contains(&schema.arena.get_index_of_primitive(Type::Missing))
                    && union.types.len() > 1
                    && options.kind == Kind::TypedDict;
                // again, per PEP 655, use NotRequired for TypedDict item, Missing otherwise
                // <del>we assume Missing/NotRequired must come with other type in a union,
                // so we can safely use NotRequired whenever possible</del>
                // ...ditto </del>
                if not_required {
                    write!(f, "NotRequired[")?;
                }
                if options.to_generate_type_alias_for_union && is_non_trivial {
                    if union
                        .types
                        .contains(&schema.arena.get_index_of_primitive(Type::Null))
                    {
                        // Say, if we have `this = int | Map | None` here
                        // we prefer
                        // `UnionedType = Union[int, Map]; this = Union[UnionedType, None]`
                        // instead of
                        // `this = UnionedType = Union[int, Map, None]`
                        //
                        // per PEP 655:
                        // Optional[] is too ubiquitous to deprecate, although use of it may fade
                        // over time in favor of the T|None notation specified by PEP 604.
                        write!(f, "Union[{}, None]", union)?;
                    } else {
                        union.fmt(f)?;
                    }
                } else {
                    self.wrap(&union.types).fmt(f)?;
                }
                if not_required {
                    write!(f, "]")?;
                }
                Ok(())
            }
            Type::Array(r#type) => {
                // dbg!(r#type);
                write!(f, "List[{}]", self.wrap(schema.arena.get(*r#type).unwrap()))
            }
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "str"),
            Type::Date => write!(f, "datetime"),
            Type::UUID => write!(f, "UUID"),
            Type::Null => write!(f, "None"),
            Type::Missing => write!(f, "Missing"),
            Type::Any => write!(f, "Any"),
        }
    }
}

// inner of Union
impl<'i, 'c> Display for Contexted<&'i HashSet<ArenaIndex>, Context<'c>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Contexted {
            inner: arnis,
            context: Context(schema, options),
        } = self;
        // NOTE: return value is a Union of variants instead of a concatenated string name hints;
        let is_non_trivial = (arnis.len()
            - arnis.contains(&schema.arena.get_index_of_primitive(Type::Null)) as usize
            - arnis.contains(&schema.arena.get_index_of_primitive(Type::Missing)) as usize)
            > 1;
        let mut iter = multipeek(
            arnis
                .iter()
                .cloned()
                .map(|r#type| schema.arena.get(r#type).unwrap())
                // again, per PEP655, use NotRequired for TypedDict item, Missing otherwise
                // and specially, a single Missing is used in place of `NotRequired[]`
                .filter(|&r#type| {
                    options.kind != Kind::TypedDict || !r#type.is_missing() || arnis.len() == 1
                })
                .filter(|&r#type| {
                    !(options.to_generate_type_alias_for_union
                        && is_non_trivial
                        && r#type.is_null())
                }),
        );
        let _ = iter.peek(); // Discard the first
        if iter.peek().is_some() {
            // Regardless of possibly discarded Missing, there are at least two other inner types.
            // TODO: switch to PEP 604 (X | Y), which is only supported by Python 3.10 for now
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
            // Not a union anymore after dicarding Missing
            self.wrap(
                iter.next()
                    .expect("The union should have at least one inner type other than Missing"),
            )
            .fmt(f)
        }
    }
}

impl<'i, 'c> Display for Contexted<&'i IndexMap<String, ArenaIndex>, Context<'c>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Contexted {
            inner: fields,
            context: Context(schema, options),
        } = self;

        // NOTE: return value are lines of field_name: field_type instead of concatenated hints;
        let iter = fields
            .iter()
            .map(|(key, &r#type)| (key, schema.arena.get(r#type).unwrap()));
        // .peekable();
        for (key, r#type) in iter {
            // // manually intersperse
            write!(f, "{}{}: {}", options.indentation, key, self.wrap(r#type))?;
            // if iter.peek().is_none() {
            writeln!(f)?;
            // }
        }
        Ok(())
    }
}
