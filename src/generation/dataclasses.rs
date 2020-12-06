use inflector::Inflector;
use itertools::Itertools;

use crate::mapset_impl::Map;
use crate::schema::Schema;

const ROOT_NAME: &'static str = "UnnamedObject";

impl Schema {
    pub fn to_dataclasses(&self, root_name: &str) -> String {
        // let named_schemas: Vec<(&str, Map<String, Schema>)>  = vec![];
        // let mut stack: Vec<&Schema> = vec![self];
        let mut output = vec![];

        fn traverse(
            schema: &Schema,
            outer_name: Option<String>,
            output: &mut Vec<String>,
        ) -> String {
            dbg!(schema, &outer_name);
            match &schema {
                Schema::Map(ref map) => {
                    let class_name = String::from(outer_name.unwrap_or(String::from(ROOT_NAME))); // TODO: convert case and suffix
                    dbg!(&class_name);
                    let fields: Vec<String> = map
                        .iter()
                        .map(|(key, schema)| {
                            let type_name = if schema.is_array() {
                                if &key.to_singular() == key && &key.to_plural() != key{
                                    format!("{}Item", key.to_pascal_case())
                                } else {
                                    key.to_pascal_case()
                                }
                            } else {
                                key.to_pascal_case()
                            };
                            format!("{} = {}", key, traverse(schema, Some(type_name), output))
                        })
                        .collect();
                    output.push(format!(
                        "\
@dataclass
class {}:
    {}",
                        class_name,
                        fields.join("\n    ")
                    ));
                    String::from(class_name)
                }
                Schema::Array(ref array) => {
                    format!("List[{}]", traverse(array, outer_name, output))
                }
                Schema::Union(ref union) => {
                    let mut optional = false;
                    let t = union
                        .iter()
                        .filter(|schema| {
                            if schema.is_null() {
                                optional = true;
                                false
                            } else {
                                true
                            }
                        })
                        .map(|schema| traverse(schema, outer_name.clone(), output))
                        .join(" | ");
                    if t.is_empty() {
                        if optional {
                            String::from("None")
                        } else {
                            panic!("Union should not be empty") // empty union
                        }
                    } else {
                        format!("Optional[{}]", t)
                    }
                }
                Schema::Int => String::from("int"),
                Schema::Float => String::from("float"),
                Schema::Bool => String::from("bool"),
                Schema::String => String::from("str"),
                // TODO: treat `* | null` as `Optional[*]`
                Schema::Null => String::from("None"), // unreachable!()
                Schema::Any => String::from("Any"),
            }
        }

        // while let Some(&schema) = stack.last() {
        //     match schema
        // }

        traverse(self, Some(String::from(root_name)), &mut output);

        output.join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use crate::inferer::infer;

    #[test]
    fn test_to_dataclasses() {
        let data = include_str!("../../tests/data/githubstatus.json");
        let s = infer(&serde_json::from_str(data).unwrap());
        println!("Redered: {}", s.to_dataclasses("RootObject"));
    }
}
