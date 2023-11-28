extern crate core;

use galvan_ast::*;
use galvan_files::Source;

/// Name of the generated rust module that exports all public items from all galvan files in this crate
#[macro_export]
macro_rules! galvan_module {
    () => {
        "galvan_module.rs"
    };
}

#[macro_export]
macro_rules! include {
    () => {
        include!(concat!(
            env!("OUT_DIR"),
            "/",
            galvan_transpiler::galvan_module!()
        ));
        // use galvan_module::*;
    };
}

#[cfg(feature = "exec")]
pub mod exec;

// TODO: This should get its own error type
pub type TranspileError = AstError;
fn transpile_source(source: Source) -> Result<String, TranspileError> {
    let ast = source.try_into_ast()?;
    Ok(ast.transpile())
}

#[derive(Debug)]
pub struct Transpilation {
    pub source: Source,
    pub transpiled: Result<String, TranspileError>,
}

pub struct SuccessfulTranspilation {
    pub source: Source,
    pub transpiled: String,
}

pub struct FailedTranspilation {
    pub source: Source,
    pub errors: TranspileError,
}

impl From<Transpilation> for Result<SuccessfulTranspilation, FailedTranspilation> {
    fn from(value: Transpilation) -> Self {
        match value.transpiled {
            Ok(transpiled) => Ok(SuccessfulTranspilation {
                source: value.source,
                transpiled,
            }),
            Err(errors) => Err(FailedTranspilation {
                source: value.source,
                errors,
            }),
        }
    }
}

pub struct TranspileErrors<'t> {
    pub source: Source,
    pub errors: &'t [TranspileError],
}

impl TranspileErrors<'_> {
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

pub fn transpile(source: Source) -> Transpilation {
    Transpilation {
        source: source.clone(),
        transpiled: transpile_source(source),
    }
}

mod transpile_item {
    mod body;
    mod ident;
    mod r#struct;
    mod task;
    mod toplevel;
    mod r#type;
    mod visibility;
}

trait Transpile {
    fn transpile(self) -> String;
}

trait Punctuated {
    fn punctuation() -> &'static str;
}

mod macros {
    macro_rules! transpile {
        ($string:expr, $($items:expr),*$(,)?) => {
            format!($string, $(($items).transpile()),*)
        };
    }

    macro_rules! impl_transpile {
        ($ty:ty, $string:expr, $($field:ident),*$(,)?) => {
            impl crate::Transpile for $ty {
                fn transpile(self) -> String {
                    crate::macros::transpile!($string, $(self.$field),*)
                }
            }
        };
    }

    macro_rules! impl_transpile_fn {
        ($ty:ty, $string:expr, $($fun:ident),*$(,)?) => {
            impl crate::Transpile for $ty {
                fn transpile(self) -> String {
                    crate::macros::transpile!($string, $(self.$fun()),*)
                }
            }
        };
    }

    macro_rules! impl_transpile_match {
        ($ty:ty, $($case:pat_param => ($($args:expr),+)),+$(,)?) => {
            impl crate::Transpile for $ty {
                #[deny(bindings_with_variant_name)]
                #[deny(unreachable_patterns)]
                #[deny(non_snake_case)]
                fn transpile(self) -> String {
                    use $ty::*;
                    match self {
                        $($case => crate::macros::transpile!($($args),+),)+
                    }
                }
            }
        };
    }

    macro_rules! impl_transpile_variants {
        ($ty:ty; $($case:ident$(,)?)+) => {
            impl crate::Transpile for $ty {
                #[deny(bindings_with_variant_name)]
                #[deny(unreachable_patterns)]
                #[deny(non_snake_case)]
                fn transpile(self) -> String {
                    use $ty::*;
                    match self {
                        $($case(inner) => inner.transpile(),)+
                    }
                }
            }
        };
    }

    macro_rules! punct {
        ($string:expr, $($ty:ty),+) => {
            $(impl Punctuated for $ty {
                fn punctuation() -> &'static str {
                    $string
                }
            })+
        };
    }

    pub(crate) use {
        impl_transpile, impl_transpile_fn, impl_transpile_match, impl_transpile_variants, punct,
        transpile,
    };
}
use macros::punct;

punct!(", ", TypeElement, TupleTypeMember);
punct!(",\n", StructTypeMember);
punct!("\n\n", RootItem);
// punct!(";\n", Statement);

impl<T> Transpile for Vec<T>
where
    T: Transpile + Punctuated,
{
    fn transpile(self) -> String {
        let punct = T::punctuation();
        self.into_iter()
            .map(|e| e.transpile())
            .reduce(|acc, e| format!("{acc}{punct}{e}"))
            .unwrap_or_else(String::new)
    }
}
