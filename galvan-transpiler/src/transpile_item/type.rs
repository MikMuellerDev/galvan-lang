use crate::macros::{impl_transpile, impl_transpile_fn, impl_transpile_variants, transpile};
use crate::{Transpile, TypeElement};
use galvan_ast::*;

// TODO: Re-export used types from galvan library to avoid referencing the used crates directly

impl_transpile!(ArrayTypeItem, "std::collections::Vec<{}>", elements);
impl_transpile!(
    DictionaryTypeItem,
    "std::collections::HashMap<{}, {}>",
    key,
    value
);
impl_transpile!(OrderedDictionaryTypeItem, "TODO {} {}", key, value);
impl_transpile!(SetTypeItem, "std::collections::HashSet<{}>", elements);
impl_transpile!(TupleTypeItem, "({})", elements);
impl_transpile_fn!(OptionalTypeItem, "Option<{}>", element);
impl_transpile_fn!(RefTypeItem, "std::sync::Arc<std::sync::Mutex<{}>>", element);
impl_transpile!(BasicTypeItem, "{}", ident);

impl Transpile for ResultTypeItem {
    fn transpile(self) -> String {
        let DowncastResultTypeItem { success, error } = self.into();
        if let Some(error) = error {
            transpile!("Result<{}, {}>", success, error)
        } else {
            transpile!("anyhow::Result<{}>", success)
        }
    }
}

impl_transpile_variants! { TypeElement;
    Plain
    Ref
    Array
    Dictionary
    OrderedDictionary
    Set
    Tuple
    Optional
    Result
}
