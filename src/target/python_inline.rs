use indexmap::IndexSet;
use itertools::{multipeek, Itertools};
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
    pub to_generate_type_alias_for_union: bool,
    pub to_nest_when_possible: bool,
}

// #[typetag::serde]
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
    let mut importing_base_class_or_class_decorators = false;
    let mut importing_datetime = false;
    let mut importing_uuid = false;

    let dominant = if options.to_nest_when_possible {
        // TODO: root array type is ignored for now
        schema.get_dominant()
    } else {
        schema.iter_topdown().collect()
    };

    let mut referenceable = HashSet::<ArenaIndex>::new();

    for arni in dominant.iter().cloned().rev() {
        let r#type = schema.arena.get(arni).unwrap();

        match r#type {
            Type::Map(map) => {
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
                    - union
                        .types
                        .contains(&schema.arena.get_index_of_primitive(Type::Null))
                        as usize
                    - union
                        .types
                        .contains(&schema.arena.get_index_of_primitive(Type::Missing))
                        as usize)
                    > 1;
                if options.to_generate_type_alias_for_union && is_non_trivial {
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
                importing_base_class_or_class_decorators = true;
                fields
                    .iter()
                    .map(|(_, &arni)| schema.arena.get(arni).unwrap())
                    .for_each(|r#type| match *r#type {
                        Type::Any => {
                            imports_from_typing.insert("Any");
                        }
                        Type::Date => importing_datetime = true,
                        Type::UUID => importing_uuid = true,
                        _ => {}
                    });
            }
            Type::Union(Union {
                /* ref name_hints, */
                ref types,
                ..
            }) => {
                let is_non_trivial = (types.len()
                    - types.contains(&schema.arena.get_index_of_primitive(Type::Null)) as usize
                    - types.contains(&schema.arena.get_index_of_primitive(Type::Missing)) as usize)
                    > 1;
                if is_non_trivial {
                    imports_from_typing.insert("Union");
                }
                if types.contains(&schema.arena.get_index_of_primitive(Type::Missing)) {
                    imports_from_typing.insert(if types.len() == 1 {
                        "Missing"
                    } else {
                        "NotRequired"
                    });
                }
            }
            Type::Array(inner) => {
                imports_from_typing.insert("List");
                if schema.arena.get(inner).unwrap().is_any() {
                    imports_from_typing.insert("Any");
                }
            }
            _ => {}
        }
    }

    if importing_base_class_or_class_decorators || !imports_from_typing.is_empty() {
        if imports_from_typing.contains("Union") {
            writeln!(additional, "# 💡 Starting from Python 3.10 (PEP 604), `Union[A, B]` can be simplified as `A | B`\n")?;
        }
        let typing_mod = if ["NotRequired", "Missing"]
            .iter()
            .any(|&t| imports_from_typing.contains(t))
        {
            // PEP 655 for now
            writeln!(
                additional,
                r#"# 💡 `NotRequired` or `Missing` are introduced since Python 3.11 (PEP 655).
#   `typing_extensions` is imported above for backwards compatibility.
#   For Python < 3.11, pip install typing_extensions. O.W., just change it to `typing`\n"#
            )?;
            "typing_extensions"
        } else {
            "typing"
        };

        write!(header, "from {} import ", typing_mod)?;
        if importing_base_class_or_class_decorators {
            write!(header, "TypedDict")?;
            if !imports_from_typing.is_empty() {
                write!(header, ", ")?;
            }
        }
        Itertools::intersperse(imports_from_typing.into_iter(), ", ")
            .try_for_each(|e| write!(header, "{}", e))?;
        writeln!(header)?;
    }
    if importing_datetime {
        writeln!(header, "from datatime import datetime")?;
    }
    if importing_uuid {
        writeln!(header, "from uuid import UUID")?;
    }
    // write!(header, "\n")?;
    Ok(())
}

impl<'c> Display for Contexted<ArenaIndex, Context<'c>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Contexted {
            inner: arni,
            context: Context(schema, options, dominant, referenceable),
        } = self;

        let r#type = schema.arena.get(arni).unwrap();
        // println!("{:?}\n\n", r#type);
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
                let not_required = union
                    .types
                    .contains(&schema.arena.get_index_of_primitive(Type::Missing))
                    && union.types.len() > 1;
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
                if not_required {
                    write!(f, "NotRequired[")?;
                }
                if is_non_trivial
                    && options.to_generate_type_alias_for_union
                    && dominant.contains(&arni)
                {
                    let nullable = union
                        .types
                        .contains(&schema.arena.get_index_of_primitive(Type::Null));
                    if nullable {
                        write!(f, "Union[")?;
                    }
                    if referenceable.contains(&arni) {
                        union.fmt(f)?;
                    } else {
                        write!(
                            f,
                            r#"{}{}{}"#,
                            options.quote_type, union, options.quote_type
                        )?;
                    }
                    if nullable {
                        // lifet up the None to the outer Union
                        write!(f, ", None]")?;
                    }
                } else {
                    self.wrap(union).fmt(f)?;
                }
                if not_required {
                    write!(f, "]")?;
                }
                Ok(())
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
            Type::Missing => write!(f, "Missing"),
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
        let the_missing = schema.arena.get_index_of_primitive(Type::Missing);
        let is_non_trivial = (union.types.len()
            - union.types.contains(&the_null) as usize
            - union.types.contains(&the_missing) as usize)
            > 1;

        let mut iter = multipeek(
            types
                .iter()
                .cloned()
                .filter(|&arni| arni != the_missing || types.len() == 1)
                .filter(|&arni| !is_non_trivial || arni != the_null),
        );

        let _ = iter.peek();
        if iter.peek().is_some() {
            // Regardless of a possible Missing, there are at least two other inner types.
            write!(f, "Union[")?;
            while let Some(arni) = iter.next() {
                // manually intersperse
                self.wrap(arni).fmt(f)?;
                if iter.peek().is_some() {
                    write!(f, ", ")?;
                }
            }
            write!(f, "]")?;
        } else {
            // Not a union anymore after dicarding Missing
            self.wrap(
                iter.next()
                    .expect("The union should have at least one inner type other than Missing"),
            )
            .fmt(f)?;
        }
        Ok(())
    }
}

impl<'i, 'c> Display for Contexted<&'i Map, Context<'c>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Contexted {
            inner: map,
            context: Context(_schema, options, _dominant, _referenceable),
        } = self;
        write!(
            f,
            "TypedDict({}{}{}, {{",
            options.quote_type, map, options.quote_type
        )?;
        let mut iter = map.fields.iter().map(|(key, &arni)| (key, arni)).peekable();

        // manually intersperse
        while let Some((key, arni)) = iter.next() {
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
        write!(f, ")")?;
        Ok(())
    }
}
