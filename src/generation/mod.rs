mod dataclasses;

pub use dataclasses::*;

pub enum TargetLang {
    PythonDataclasses,
    PythonTypedDict,
    RustSerde,
    TypeScriptInterface
}
