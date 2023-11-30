use std::path::PathBuf;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Token, Result, Expr};

struct MacroInput { prefix: Ident, tag: String, operation: Option<Ident>, processed: Expr }

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let prefix = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;
        let tag = input.parse::<Ident>()?.to_string();
        input.parse::<Token![,]>()?;
        let operation = input.parse::<Ident>().ok();
        if operation.is_some() { let _ = input.parse::<Token![,]>(); }
        let processed = input.parse::<Expr>()?;
        Ok(MacroInput { prefix, tag, operation, processed })
    }
}

#[proc_macro]
pub fn generate_code_tests(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let macro_input = syn::parse_macro_input!(input as MacroInput);

    // panic!("{:?}", std::env::current_dir());

    // Generate the test for each file in the example-code directory
    std::fs::read_dir("example-code")
        .unwrap()
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.file_type().is_ok_and(|ft| ft.is_file()))
        .map(|f| f.path())
        .map(|path| generate_test(path, &macro_input))
        .collect::<TokenStream>()
        .into()
}

fn generate_test(path: PathBuf, macro_input: &MacroInput) -> TokenStream {
    let MacroInput { prefix, tag, operation, processed } = macro_input;
    let name = path.file_stem().unwrap().to_str().unwrap();

    let Ok(test_file) = std::fs::read_to_string(&path) else { panic!("Test file not found!") };

    let expected_struct = expected_result(&test_file, tag).unwrap_or_else(|e| panic!("{}", e));
    let code = test_file;

    let test_name = format_ident!("{prefix}_{name}");
    if let Some(operation) = operation {
        quote! {
            #[test]
            fn #test_name() {
                let code = #code;
                let expected_struct = #expected_struct.#operation();
                let actual_struct = #processed.#operation();
                assert_eq!(expected_struct, actual_struct);
            }
        }
    } else {
        quote! {
            #[test]
            fn #test_name() {
                let code = #code;
                let expected_struct = #expected_struct;
                let actual_struct = #processed;
                assert_eq!(expected_struct, actual_struct);
            }
        }
    }
}

fn expected_result(test_file: &str, tag: &str) -> std::result::Result<TokenStream, &'static str> {
    let prefix = "/*#";
    let mut lines = vec![];
    let mut iter = test_file.lines();

    loop {
        match iter.next() {
            Some(line) if line.strip_prefix(prefix).is_some_and(|s| s.trim().starts_with(tag)) => break,
            Some(_) => continue,
            None => return Err("Tag not found in test file!"),
        }
    }

    loop {
        match iter.next() {
            Some(line) if line.starts_with("*/") => break,
            Some(line) => lines.push(line),
            None => return Err("Tag not closed in test file!"),
        }
    }

    let code = lines.join("\n");
    code.parse::<TokenStream>().map_err(|_| "Code in frontmatter is not valid Rust code!")
}