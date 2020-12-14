use crate::generation::TargetLang;
struct Options {
    target: TargetLang,
    merge_similar_maps: bool,
    generate_type_alias_for_union: bool,
    merge_same_unions: bool,
    root_type_name: String,
}

pub fn infer_and_generate() -> String {

}