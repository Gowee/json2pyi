use indexmap::IndexSet;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use std::{
    collections::HashSet,
    fmt::{self, Display, Write},
};

use crate::schema::{ArenaIndex, ITypeArena, Map, Schema, Type, Union};

use super::{with_context, Contexted, Quote, TargetGenerator};

#[derive(Clone, Copy, Debug)] // Or just use &Context as a context
struct Context<'c>(
    &'c Schema,
    &'c PythonTypedDict,
    &'c IndexSet<ArenaIndex>,
    &'c HashSet<ArenaIndex>,
);

#[derive(Debug, Serialize, Deserialize)]
pub struct PythonTypedDict {
    pub quote_type: Quote,
    pub generate_type_alias_for_union: bool,
    pub nesting_when_possible: bool,
    pub mark_optional_as_not_total: bool,
}

#[typetag::serde]
impl TargetGenerator for PythonTypedDict {
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
    options: &PythonTypedDict,
    header: &mut dyn Write,
    body: &mut dyn Write,
    _additional: &mut dyn Write,
) -> fmt::Result {
    let mut imports_from_typing = HashSet::new();
    let mut import_base_class_or_class_decorators = false;
    let mut import_datetime = false;
    let mut import_uuid = false;

    let dominant = if options.nesting_when_possible {
        schema.get_dominant()
    } else {
        schema.iter_topdown().collect()
    };

    let mut referenceable = HashSet::<ArenaIndex>::new();

    for arni in dominant.iter().cloned().rev() {
        let r#type = schema.arena.get(arni).unwrap();

        match r#type {
            Type::Map(map) => {
                dbg!(r#type);
                write!(
                    body,
                    "{} = {}\n\n",
                    map,
                    with_context(map, Context(schema, options, &dominant, &referenceable))
                )?;
                referenceable.insert(arni);
            }
            Type::Union(union) => {
                let is_non_trivial = (union.types.len()
                    - union.types.contains(&schema.arena.get_index_of_primitive(Type::Null)) as usize)
                    > 1;
                if options.generate_type_alias_for_union && is_non_trivial {
                    write!(
                        body,
                        "{} = {}\n\n",
                        union,
                        with_context(union, Context(schema, options, &dominant, &referenceable))
                    )?;
                    referenceable.insert(arni);
                }
            }
            _ => (),
        }
    }
    for arni in schema.iter_topdown() {
        let r#type = schema.arena.get(arni).unwrap();
        match *r#type {
            Type::Map(Map {
                /* ref name_hints, */
                ref fields,
                ..
            }) => {
                import_base_class_or_class_decorators = true;
                fields
                    .iter()
                    .map(|(_, &arni)| schema.arena.get(arni).unwrap())
                    .for_each(|r#type| match *r#type {
                        Type::Any => {
                            imports_from_typing.insert("Any");
                        }
                        Type::Date => import_datetime = true,
                        Type::UUID => import_uuid = true,
                        _ => {}
                    });
            }
            Type::Union(Union {
                /* ref name_hints, */
                ref types,
                ..
            }) => {
                let is_non_trivial = (types.len()
                    - types.contains(&schema.arena.get_index_of_primitive(Type::Null)) as usize)
                    > 1;
                imports_from_typing.insert(if is_non_trivial { "Union" } else { "Optional" });
            }
            Type::Array(_) => {
                imports_from_typing.insert("List");
            }
            _ => {}
        }
    }

    if !imports_from_typing.is_empty() {
        write!(header, "from typing import ")?;
        if import_base_class_or_class_decorators {
            write!(header, "TypedDict")?;
            if !imports_from_typing.is_empty() {
                write!(header, ", ")?;
            }
        }
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

impl<'i, 'c> Display for Contexted<ArenaIndex, Context<'c>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Contexted {
            inner: arni,
            context: Context(schema, options, dominant, referenceable),
        } = self;

        let r#type = schema.arena.get(arni).unwrap();
        println!("{:?}\n\n", r#type);
        match *r#type {
            Type::Map(ref map) => {
                if dominant.contains(&arni) {
                    if referenceable.contains(&arni) {
                        map.fmt(f)
                    } else {
                        write!(f, r#"{}{}{}"#, options.quote_type, map, options.quote_type)
                    }
                } else {
                    self.wrap(map).fmt(f)
                }
            }
            Type::Union(ref union) => {
                let is_non_trivial = (union.types.len()
                    - union.types.contains(&schema.arena.get_index_of_primitive(Type::Null)) as usize)
                    > 1;
                if is_non_trivial && options.generate_type_alias_for_union && dominant.contains(&arni) {
                    if referenceable.contains(&arni) {
                        union.fmt(f)
                    } else {
                        write!(
                            f,
                            r#"{}{}{}"#,
                            options.quote_type, union, options.quote_type
                        )
                    }
                } else {
                    self.wrap(union).fmt(f)
                }
            }
            Type::Array(inner) => {
                write!(f, "List[{}]", self.wrap(inner))
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

impl<'i, 'c> Display for Contexted<&'i Union, Context<'c>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Contexted {
            inner: union,
            context: Context(schema, _options, _dominant, _referenceable),
        } = self;
        let Union {
            name_hints: _,
            ref types,
        } = *union;
        let the_null = schema.arena.get_index_of_primitive(Type::Null);
        let optional = types.contains(&the_null);
        let mut iter = types
            .iter()
            .cloned()
            .filter(|&arni| arni != the_null)
            .peekable();

        if optional {
            write!(f, "Optional[")?;
        }
        if types.len() - optional as usize > 1 {
            // Regardless of a possible null, there are at least two other inner types.
            while let Some(arni) = iter.next() {
                // manually intersperse
                self.wrap(arni).fmt(f)?;
                if iter.peek().is_some() {
                    write!(f, ", ")?;
                }
            }
            write!(f, "]")?;
        } else {
            // Not a union anymore after dicarding Null
            self.wrap(
                iter.next()
                    .expect("The union should have at least one inner type other than Null"),
            )
            .fmt(f)?;
        }
        if optional {
            write!(f, "]")?;
        }
        Ok(())
    }
}

impl<'i, 'c> Display for Contexted<&'i Map, Context<'c>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Contexted {
            inner: map,
            context: Context(schema, options, _dominant, _referenceable),
        } = self;
        let mut is_total = true;
        write!(
            f,
            "TypedDict({}{}{}, {{",
            options.quote_type, map, options.quote_type
        )?;
        let mut iter = map.fields.iter().map(|(key, &arni)| (key, arni)).peekable();

        // manually intersperse
        while let Some((key, arni)) = iter.next() {
            let r#type = schema.arena.get(arni).unwrap();
            if let Type::Union(Union { ref types, .. }) = *r#type {
                if options.mark_optional_as_not_total
                    && types.contains(&schema.arena.get_index_of_primitive(Type::Null))
                {
                    is_total = false;
                }
            }
            write!(
                f,
                "{}{}{}: {}",
                options.quote_type,
                key,
                options.quote_type,
                self.wrap(arni)
            )?;
            if iter.peek().is_some() {
                write!(f, ", ")?;
            }
        }
        write!(f, "}}")?;
        if !is_total {
            // NOTE: optional is for a field, but totality is only for its parental type in whole
            write!(f, ", total=False")?;
        }
        write!(f, ")")?;
        Ok(())
    }
}
