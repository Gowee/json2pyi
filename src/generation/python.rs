use indexmap::IndexMap;

use itertools::{multipeek, Itertools};
use serde::{Deserialize, Serialize};

use crate::schema::{ArenaIndex, ITypeArena, Map, Schema, Type, Union};
use std::{
    collections::HashSet,
    fmt::{self, Display, Write},
    unimplemented,
};

use super::{wrap, Indentation, TargetGenerator, Wrapped};

#[derive(Debug, Serialize, Deserialize)]
pub struct Python {
    pub kind: Kind,
    pub generate_type_alias_for_union: bool,
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
    /// Use `TypedDict` from the built-in `typing` module as the base class
    TypedDict,
    /// Use `TypedDict` from the built-in `typing` module as the base class with all sub classes
    /// nested into the root one
    NestedTypedDict,
}

#[typetag::serde]
impl TargetGenerator for Python {
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
    options: &Python,
    header: &mut dyn Write,
    body: &mut dyn Write,
    additional: &mut dyn Write,
) -> fmt::Result {
    let wrapper = wrap(&(), schema, options); // helper

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
    let mut import_base_class_or_class_decorators = false;
    let mut import_datetime = false;
    let mut import_uuid = false;

    for r#type in schema.iter_topdown() {
        match *r#type {
            Type::Map(Map {
                /* ref name_hints, */
                ref fields,
                ..
            }) => {
                import_base_class_or_class_decorators = true;
                fields
                    .iter()
                    .map(|(_, &r#type)| schema.arena.get(r#type).unwrap())
                    .for_each(|r#type| match *r#type {
                        Type::Any => {
                            imports_from_typing.insert("Any");
                        }
                        Type::Date => import_datetime = true,
                        Type::UUID => import_uuid = true,
                        _ => {}
                    });
                match options.kind {
                    Kind::TypedDict => {
                        write!(
                            body,
                            r#"{type_name} = TypedDict("{type_name}", {fields_and_totality})"#,
                            type_name = wrapper.wrap(r#type),
                            fields_and_totality = wrapper.wrap(fields),
                        )?;
                    }
                    Kind::NestedTypedDict => {
                        // The wholely nested root object is not generated during loop
                    }
                    _ => {
                        // class
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
                }
            }
            Type::Union(Union {
                /* ref name_hints, */
                ref types,
                ..
            }) => {
                let is_non_trivial = (types.len()
                    - types.contains(&schema.arena.get_index_of_primitive(Type::Null)) as usize)
                    > 1;
                if options.generate_type_alias_for_union && is_non_trivial {
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

    if options.kind == Kind::NestedTypedDict {
        // write!(
        //     body,
        //     "{}",
        //     wrapper.wrap(schema.arena.get(schema.root).unwrap())
        // )?;
        todo!();
    }

    if import_base_class_or_class_decorators {
        let import = match options.kind {
            Kind::Dataclass => "from dataclasses import dataclass",
            Kind::DataclassWithJSON => {
                "from dataclasses import dataclass\nfrom dataclasses_json import dataclass_json"
            }
            Kind::PydanticBaseModel => "from pydantic import BaseModel",
            Kind::PydanticDataclass => "from pydantic import dataclass",
            Kind::TypedDict | Kind::NestedTypedDict => "from typing import TypedDict",
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
    if import_datetime {
        write!(header, "from datatime import datetime\n\n")?;
    }
    if import_uuid {
        write!(header, "from uuid import UUID\n\n")?;
    }
    Ok(())
}

impl<'i, 's, 'g> Display for Wrapped<'i, 's, 'g, (), Python> {
    /// Generate [`Kind::NestedTypedDict`]
    ///
    /// Nested TypedDict requires special handling when recursion, resulting in its incompatibility
    /// with the [`Display`] trait. So just handle it seperately.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Wrapped {
            schema, options, ..
        } = self;
        assert!(options.kind == Kind::NestedTypedDict);

        let mut stack = vec![schema.root];
        let mut seen: HashSet<ArenaIndex> = stack.iter().cloned().collect();
        while let Some(curr) = stack.pop() {
            let r#type = schema.arena.get(curr).unwrap();
            match *r#type {
                Type::Map(Map{ref name_hints, ref fields}) => {
                    if seen.contains(&curr) {
                        self.wrap(r#type).fmt(f)?;
                    }
                    else {
                        seen.insert(curr);
                        write!(
                            f,
                            r#"TypedDict("{}", {})"#,
                            name_hints,
                            self.wrap(fields),
                        )?;
                    }
                    for (_, &r#type) in fields.iter().rev() {
                        stack.push(r#type);
                    }
                }
                _ => self.wrap(r#type).fmt(f)?
            }
        }
        Ok(())
    }
}

impl<'i, 's, 'g> Display for Wrapped<'i, 's, 'g, Type, Python> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner {
            Type::Map(Map {
                ref name_hints,
                ref fields,
            }) => {
                // TODO: eliminate unnecessary heap allocation
                let type_name = if name_hints.is_empty() {
                    format!("UnnammedType{:X}", self.inner as *const Type as usize)
                } else {
                    name_hints.to_string()
                };
                if self.options.kind == Kind::NestedTypedDict {
                    write!(
                        f,
                        r#"TypedDict("{type_name}", {fields_and_totality})"#,
                        type_name = type_name,
                        fields_and_totality = self.wrap(fields),
                    )
                } else {
                    write!(f, "{}", type_name)
                }
            }
            Type::Union(Union {
                ref name_hints,
                ref types,
            }) => {
                if self.options.generate_type_alias_for_union && {
                    let is_non_trivial = (types.len()
                        - types.contains(&self.schema.arena.get_index_of_primitive(Type::Null))
                            as usize)
                        > 1;
                    is_non_trivial
                } {
                    if name_hints.is_empty() {
                        write!(f, "UnnammedUnion{:X}", self.inner as *const Type as usize)
                    } else {
                        write!(f, "{}Union", name_hints)
                    }
                } else {
                    let optional =
                        types.contains(&self.schema.arena.get_index_of_primitive(Type::Null));
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
                    self.wrap(self.schema.arena.get(*r#type).unwrap())
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

impl<'i, 's, 'g> Display for Wrapped<'i, 's, 'g, HashSet<ArenaIndex>, Python> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // NOTE: return value is a union of variants instead of a concatenated string name hints;
        //       null is discarded here
        let mut iter = multipeek(
            self.inner
                .iter()
                .cloned()
                .map(|r#type| self.schema.arena.get(r#type).unwrap())
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

impl<'i, 's, 'g> Display for Wrapped<'i, 's, 'g, IndexMap<String, ArenaIndex>, Python> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.options.kind {
            Kind::TypedDict | Kind::NestedTypedDict => {
                // NOTE: return value are TS-interface-like dict with possible trailing totality
                let mut is_total = true;
                write!(f, "{{")?;
                let mut iter = self
                    .inner
                    .iter()
                    .map(|(key, &r#type)| (key, self.schema.arena.get(r#type).unwrap()))
                    .peekable();
                // manually intersperse
                while let Some((key, r#type)) = iter.next() {
                    if let Type::Union(Union { ref types, .. }) = *r#type {
                        if types.contains(&self.schema.arena.get_index_of_primitive(Type::Null)) {
                            is_total = false;
                        }
                    }
                    write!(f, r#""{}": {}"#, key, self.wrap(r#type))?;
                    if iter.peek().is_some() {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "}}")?;
                if !is_total {
                    write!(f, ", total=False")?;
                }
            }
            _ => {
                // NOTE: return value are lines of field_name: field_type instead of concatenated hints;
                let mut iter = self
                    .inner
                    .iter()
                    .map(|(key, &r#type)| (key, self.schema.arena.get(r#type).unwrap()));
                // .peekable();
                while let Some((key, r#type)) = iter.next() {
                    // // manually intersperse
                    write!(
                        f,
                        "{}{}: {}",
                        self.options.indentation,
                        key,
                        self.wrap(r#type)
                    )?;
                    // if iter.peek().is_none() {
                    write!(f, "\n")?;
                    // }
                }
            }
        }
        Ok(())
    }
}
