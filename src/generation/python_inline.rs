use indexmap::IndexSet;
use itertools::{multipeek, Itertools};
use serde::{Deserialize, Serialize};

use std::{
    collections::HashSet,
    fmt::{self, Display, Write},
    unimplemented,
};

use crate::schema::{ArenaIndex, ITypeArena, Map, Schema, Type, Union};

use super::{withContext, Contexted, Indentation, Quote, TargetGenerator};

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
    // pub generate_type_alias_for_union: bool,
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
    additional: &mut dyn Write,
) -> fmt::Result {
    let mut imports_from_typing = HashSet::new();
    let mut import_base_class_or_class_decorators = false;
    let mut import_datetime = false;
    let mut import_uuid = false;

    let dominant = schema.get_dominant();

    // let wrapper = withContext((), (schema, options, &dominant)); // helper

    let mut referenceable = HashSet::<ArenaIndex>::new();
    // panic!("{:?}", dominant.iter().map(|&arni| schema.arena.get(arni).unwrap()).collect::<Vec<_>>());

    for arni in dominant.iter().cloned().rev() {
        let r#type = schema.arena.get(arni).unwrap();

        if let Some(map) = r#type.as_map() {
            dbg!(r#type);
            write!(
                body,
                "{} = {}\n\n",
                map,
                withContext(map, Context(schema, options, &dominant, &referenceable))
            )?;
            referenceable.insert(arni);
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

                // match options.kind {
                //     Kind::TypedDict => {
                //         write!(
                //             body,
                //             r#"{type_name} = TypedDict("{type_name}", {fields_and_totality})"#,
                //             type_name = wrapper.wrap(r#type),
                //             fields_and_totality = wrapper.wrap(fields),
                //         )?;
                //     }
                //     Kind::NestedTypedDict => {
                //         // The wholely nested root object is not generated during loop
                //     }
                //     _ => {
                //         // class
                //         write!(
                //             body,
                //             "{}class {}{}:\n{}",
                //             decorators,
                //             wrapper.wrap(r#type), // type name
                //             base_class,           // to inherit
                //             wrapper.wrap(fields)  // lines of fields and types
                //         )?;
                //         write!(body, "\n")?;
                //     }
                // }
            }
            Type::Union(Union {
                /* ref name_hints, */
                ref types,
                ..
            }) => {
                let is_non_trivial = (types.len()
                    - types.contains(&schema.arena.get_index_of_primitive(Type::Null)) as usize)
                    > 1;
                // if options.generate_type_alias_for_union && is_non_trivial {
                //     imports_from_typing.insert("Union");
                //     write!(body, "{} = {}", wrapper.wrap(r#type), wrapper.wrap(types))?;
                //     write!(body, "\n")?;
                // }
                imports_from_typing.insert(if is_non_trivial { "Union" } else { "Optional" });
            }
            Type::Array(_) => {
                imports_from_typing.insert("List");
            }
            _ => {}
        }
    }

    // if options.kind == Kind::NestedTypedDict {
    //     write!(body, "{}", wrapper.wrap(()))?;
    // }

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
                self.wrap(union).fmt(f)
                // let optional =
                //     types.contains(&self.schema.arena.get_index_of_primitive(Type::Null));
                // let union = self.wrap(types);

                // if optional {
                //     write!(f, "Optional[{}]", union)
                // } else {
                //     union.fmt(f)
                // }
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
            context: Context(schema, options, dominant, referenceable),
        } = self;
        let Union {
            ref name_hints,
            ref types,
        } = *union;
        let the_null = schema.arena.get_index_of_primitive(Type::Null);
        let optional = types.contains(&the_null);
        // let mut iter = multipeek(types.iter().cloned().filter(|&arni| arni != the_null));
        let mut iter = types
            .iter()
            .cloned()
            .filter(|&arni| arni != the_null)
            .peekable();

        if optional {
            write!(f, "Optional[")?;
        }
        // let _ = iter.peek(); // Discard the first
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
            context: Context(schema, options, dominant, referenceable),
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
            // NOTE: optional is for a field, but totality is only for its parental type
            write!(f, ", total=False")?;
        }
        write!(f, ")")?;
        Ok(())
    }
}
