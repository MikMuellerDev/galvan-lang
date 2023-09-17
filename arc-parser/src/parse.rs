use arc_lexer::Token;

use crate::*;

mod parse_type;
pub use parse_type::*;

// TODO: Introduce type for parsed source
type ParsedSource = ();

pub fn parse_source(lexer: &mut Tokenizer<'_>) -> Result<ParsedSource> {
    let mut m = Modifiers::new();

    // TODO: Drain the spanned lexer completely into a peekable iterator and store occuring errors
    while let Some(ref token) = lexer.next() {
        match token {
            Ok(Token::FnKeyword) => {
                parse_fn(lexer, &m)?;
                m.reset();
            }
            Ok(Token::TypeKeyword) => {
                parse_type(lexer, &m)?;
                m.reset();
            }
            Ok(Token::MainKeyword) if !m.has_vis_modifier() && !m.has_const_modifier() => {
                parse_main(lexer, m.asyncness)?;
                m.reset();
            }
            Ok(Token::TestKeyword) if !m.has_vis_modifier() && !m.has_const_modifier() => {
                parse_test(lexer, m.asyncness)?;
                m.reset();
            }
            Ok(Token::BuildKeyword) => {
                return lexer.err(
                    "The build keyword is reserved but currently not implemented yet. Found at: ",
                );
            }

            Ok(Token::PublicKeyword) if !m.has_vis_modifier() => m.visibility = Visibility::Public,
            Ok(Token::ConstKeyword) if !m.has_const_modifier() => m.constness = Const::Const,
            Ok(Token::AsyncKeyword) if !m.has_async_modifier() => m.asyncness = Async::Async,

            // TODO: Add stringified token
            _ => return Err(lexer.unexpected_token()),
        }
    }

    Ok(())
}

pub fn parse_fn(lexer: &mut Tokenizer, mods: &Modifiers) -> Result<FnDecl> {
    let token = lexer
        .next()
        .ok_or(lexer.msg("Expected function name but found end of file."))?
        .map_err(|_| lexer.msg("Invalid identifier for type name at: "))?;

    todo!("Parse function declaration")
}

pub fn parse_main(lexer: &mut Tokenizer, asyncness: Async) -> Result<()> {
    let token = lexer
        .next()
        .ok_or(lexer.msg("Expected main body but found end of file."))?
        .map_err(|_| lexer.msg("Invalid identifier, expected '{' at: "))?;

    todo!("Parse main function")
}

pub fn parse_test(lexer: &mut Tokenizer, asyncness: Async) -> Result<()> {
    let token = lexer
        .next()
        .ok_or(lexer.msg("Expected test body or test description but found end of file."))?
        .map_err(|_| lexer.msg("Invalid identifier, expected '{' or test description at: "))?;

    todo!("Parse main function")
}
