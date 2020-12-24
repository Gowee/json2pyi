use indexmap::IndexMap;

use itertools::{multipeek, Itertools};
use serde::{Deserialize, Serialize};

use std::{
    collections::HashSet,
    fmt::{self, Display, Write},
    write,
};
// use crate::mapset_impl::Map;
use crate::schema::{ArenaIndex, ITypeArena, Map, Schema, Type, Union};

use super::{wrap, Indentation, TargetGenerator, Wrapped};

#[derive(Debug, Serialize, Deserialize)]
pub struct PythonDataclasses {
    pub generate_type_alias_for_union: bool,
    // pub use_pydantic_datamodel: bool,
    pub indentation: Indentation,
}

#[typetag::serde]
impl TargetGenerator for PythonDataclasses {
    fn write_output(
        &self,
        schema: &Schema,
        header: &mut dyn Write,
        body: &mut dyn Write,
        _additional: &mut dyn Write,
    ) -> fmt::Result {
        write_output(schema, self, header, body)
    }
}

#[inline(always)]
fn write_output(
    schema: &Schema,
    options: &PythonDataclasses,
    header: &mut dyn Write,
    body: &mut dyn Write,
) -> fmt::Result {
    let wrapper = wrap(&(), schema, options);

    let mut imports_from_typing = HashSet::new();
    let mut import_dataclasses = false;
    let mut import_datetime = false;
    let mut import_uuid = false;

    for r#type in schema.iter_topdown() {
        match *r#type {
            Type::Map(Map {
                /* ref name_hints, */
                ref fields,
                ..
            }) => {
                import_dataclasses = true;
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

                write!(
                    body,
                    "@dataclass\nclass {}:\n{}",
                    wrapper.wrap(r#type),
                    wrapper.wrap(fields)
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
    if import_dataclasses {
        write!(header, "from dataclasses import dataclass\n\n")?;
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

impl<'i, 's, 'g> Display for Wrapped<'i, 's, 'g, Type, PythonDataclasses> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner {
            Type::Map(Map {
                ref name_hints,
                fields: _,
            }) => {
                if name_hints.is_empty() {
                    write!(f, "UnnammedType{:X}", self.inner as *const Type as usize)
                } else {
                    name_hints.fmt(f)
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

impl<'i, 's, 'g> Display for Wrapped<'i, 's, 'g, HashSet<ArenaIndex>, PythonDataclasses> {
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

impl<'i, 's, 'g> Display for Wrapped<'i, 's, 'g, IndexMap<String, ArenaIndex>, PythonDataclasses> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // NOTE: return value are lines of field_name: field_type instead of concatenated hints;
        // write!(f, "Union[")?;
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
        Ok(())
    }
}
