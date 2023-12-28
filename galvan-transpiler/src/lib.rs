use galvan_ast::*;
use galvan_files::{FileError, Source};
use std::collections::HashMap;
use std::iter;
use thiserror::Error;

pub(crate) use galvan_resolver::LookupContext;
use galvan_resolver::LookupError;

// TODO: Maybe use something like https://crates.io/crates/ruast to generate the Rust code in a more reliable way

/// Name of the generated rust module that exports all public items from all galvan files in this crate
#[macro_export]
macro_rules! galvan_module {
    () => {
        "galvan_module.rs"
    };
}

#[cfg(feature = "exec")]
pub mod exec;

#[derive(Debug, Error)]
pub enum TranspileError {
    #[error(transparent)]
    Ast(#[from] AstError),
    #[error(transparent)]
    Lookup(#[from] LookupError),
    #[error(transparent)]
    File(#[from] FileError),
}

fn transpile_sources(sources: Vec<Source>) -> Result<Vec<TranspileOutput>, TranspileError> {
    let asts = sources
        .into_iter()
        .map(|s| s.try_into_ast())
        .collect::<Result<Vec<_>, _>>()?;

    transpile_asts(asts)
}

fn transpile_asts(asts: Vec<Ast>) -> Result<Vec<TranspileOutput>, TranspileError> {
    // TODO: Declare extern types in standard library instead of hardcoding them here
    let predefined = Source::from_string(
        "
        pub type Int
        pub type Float 
        pub type String
        ",
    )
    .try_into_ast()
    .expect("Failed to parse predefined types")
    .segmented()
    .expect("Predefined not to have conflicting declarations");

    let segmented = asts.segmented()?;
    let lookup = LookupContext::new().with(&predefined)?.with(&segmented)?;

    transpile_segmented(&segmented, &lookup)
}

struct TypeFileContent<'a> {
    pub ty: &'a TypeDecl,
    pub fns: Vec<&'a FnDecl>,
}

fn transpile_segmented(
    segmented: &SegmentedAsts,
    lookup: &LookupContext,
) -> Result<Vec<TranspileOutput>, TranspileError> {
    let mut type_files: HashMap<TypeIdent, TypeFileContent> = HashMap::new();
    let modules = type_files
        .keys()
        .map(|id| id.as_str())
        .map(|id| format!("mod {};\npub use {}::*\n", id, id))
        .collect::<Vec<_>>()
        .join("\n");

    for ty in &segmented.types {
        type_files
            .insert(
                ty.item.ident().clone(),
                TypeFileContent {
                    ty,
                    fns: Vec::new(),
                },
            )
            .ok_or_else(|| LookupError::DuplicateType(ty.item.ident().clone()))?;
    }

    let mut toplevel_functions = Vec::new();
    for func in &segmented.functions {
        if let Some(receiver) = func.signature.receiver() {
            let TypeElement::Plain(ty) = &receiver.param_type else {
                todo!("Allow extending complex types")
            };
            let content = type_files.get_mut(&ty.ident).expect(
                "TODO: Handle error for member functions that refer to types that are not declared",
            );
            content.fns.push(&func.item);
        } else {
            toplevel_functions.push(func);
        }
    }

    let toplevel_functions = toplevel_functions
        .iter()
        .map(|func| func.transpile(lookup))
        .collect::<Vec<_>>()
        .join("\n\n");
    let lib = TranspileOutput {
        file_name: galvan_module!().into(),
        content: [modules, toplevel_functions].join("\n\n").into(),
    };

    let type_files = type_files.iter().map(|(k, v)| TranspileOutput {
        file_name: format!("{}.rs", k.as_str()).into(),
        content: iter::once(v.ty.transpile(lookup))
            .chain(v.fns.iter().map(|item| item.transpile(lookup)))
            .collect::<Vec<_>>()
            .join("\n\n")
            .into(),
    });

    Ok(iter::once(lib).chain(type_files).collect())
}

pub struct TranspileOutput {
    pub file_name: Box<str>,
    pub content: Box<str>,
}

#[derive(Debug)]
pub struct Transpilation {
    pub sources: Vec<Source>,
    pub transpiled: Result<String, TranspileError>,
}

pub struct SuccessfulTranspilation {
    pub sources: Vec<Source>,
    pub transpiled: String,
}

pub struct FailedTranspilation {
    pub sources: Vec<Source>,
    pub errors: TranspileError,
}

impl From<Transpilation> for Result<SuccessfulTranspilation, FailedTranspilation> {
    fn from(value: Transpilation) -> Self {
        match value.transpiled {
            Ok(transpiled) => Ok(SuccessfulTranspilation {
                sources: value.sources,
                transpiled,
            }),
            Err(errors) => Err(FailedTranspilation {
                sources: value.sources,
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

pub fn transpile(sources: Vec<Source>) -> Result<Vec<TranspileOutput>, TranspileError> {
    transpile_sources(sources)
}

mod transpile_item {
    mod fn_decl;
    mod ident;
    mod statement;
    mod r#struct;
    mod task;
    mod toplevel;
    mod r#type;
    mod visibility;
}

trait Transpile {
    fn transpile(&self, lookup: &LookupContext) -> String;
}

trait Punctuated {
    fn punctuation() -> &'static str;
}

mod macros {
    macro_rules! transpile {
        ($lookup:ident, $string:expr, $($items:expr),*$(,)?) => {
            format!($string, $(($items).transpile($lookup)),*)
        };
    }

    macro_rules! impl_transpile {
        ($ty:ty, $string:expr, $($field:ident),*$(,)?) => {
            impl crate::Transpile for $ty {
                fn transpile(&self, lookup: &crate::LookupContext) -> String {
                    crate::macros::transpile!(lookup, $string, $(self.$field),*)
                }
            }
        };
    }

    macro_rules! impl_transpile_fn {
        ($ty:ty, $string:expr, $($fun:ident),*$(,)?) => {
            impl crate::Transpile for $ty {
                fn transpile(&self, lookup: &crate::LookupContext) -> String {
                    crate::macros::transpile!(lookup, $string, $(self.$fun()),*)
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
                fn transpile(&self, lookup: &crate::LookupContext) -> String {
                    use $ty::*;
                    match self {
                        $($case => crate::macros::transpile!(lookup, $($args),+),)+
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
                fn transpile(&self, lookup: &crate::LookupContext) -> String {
                    use $ty::*;
                    match self {
                        $($case(inner) => inner.transpile(lookup),)+
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

punct!(", ", TypeElement, TupleTypeMember, Param);
punct!(",\n", StructTypeMember);
punct!("\n\n", RootItem);
punct!(";\n", Statement);

impl<T> Transpile for Vec<T>
where
    T: Transpile + Punctuated,
{
    fn transpile(&self, lookup: &LookupContext) -> String {
        let punct = T::punctuation();
        self.into_iter()
            .map(|e| e.transpile(lookup))
            .reduce(|acc, e| format!("{acc}{punct}{e}"))
            .unwrap_or_else(String::new)
    }
}
