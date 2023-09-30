use std::vec::IntoIter;

use arc_lexer::Token;

use crate::*;

/// Parses a type definition. This method assumes that modifiers and the type keyword were already consumed
pub fn parse_type(lexer: &mut Tokenizer, mods: &Modifiers) -> Result<TypeDecl> {
    let (token, _span) = lexer
        .next()
        .ok_or(lexer.eof("Expected type name but found end of file."))?;
    let token = token.map_err(|_| lexer.invalid_idenfier("Invalid identifier for type name"))?;

    if let Token::Ident(name) = token {
        let (token, _span) = lexer
            .next()
            .ok_or(lexer.eof("Expected type name but found end of file."))?;
        let token =
            token.map_err(|_| lexer.invalid_idenfier("Invalid identifier for type name"))?;

        match token {
            Token::BraceOpen => {
                let mut tokens = lexer.parse_until_matching(MatchingToken::Brace)?;
                let (_, _) = tokens.pop().ensure_token(Token::BraceClose)?;
                tokens.trim_trailing(Token::Newline);

                let members = parse_struct_type_members(tokens)?;
                let t = StructTypeDecl { members };
                Ok(TypeDecl::StructType(t))
            }
            Token::ParenOpen => {
                let mut tokens = lexer.parse_until_matching(MatchingToken::Paren)?;
                let (_, _) = tokens.pop().ensure_token(Token::ParenClose)?;
                tokens.trim_trailing(Token::Newline);

                let members = parse_tuple_type_members(tokens)?;
                let t = TupleTypeDecl { members };
                Ok(TypeDecl::TupleType(t))
            }
            Token::Assign => {
                // TODO: Allow newlines after some symbols like +
                // TODO: Also allow an end of file here
                let mut tokens = lexer.parse_until_token(Token::Newline)?;
                let (_, _) = tokens.pop().ensure_token(Token::Newline)?;

                let aliased_type = parse_type_alias(tokens)?;
                let t = AliasTypeDecl {
                    r#type: aliased_type,
                };
                Ok(TypeDecl::AliasType(t))
            }
            _ => Err(lexer.unexpected(format!(
                "Expected one of the following:
                        - type alias:  'type {name} = TypeA'
                        - struct type: 'type {name} {{ attr: TypeA, ... }}'
                        - tuple type:  'type {name}(TypeA, TypeB, ...)
                                
                    ...but found unexpected token instead
                    "
            ))),
        }
    } else {
        Err(lexer.invalid_idenfier("Invalid identifier for type name at: "))
    }
}

fn parse_struct_type_members(tokens: Vec<SpannedToken>) -> Result<Vec<StructTypeMember>> {
    let mut token_iter = tokens.into_iter();
    let mut members = vec![];
    // TODO: Also allow comma here
    // TODO: Allow directly starting with members without newline
    while token_iter.next().ensure_token(Token::Newline).is_ok() {
        // TODO: parse visibility modifiers and keywords such as ref here, probably parse all until newline to do that
        let field_name = token_iter.next().ident()?;
        let field = Ident::new(field_name);
        let (_, _) = token_iter.next().ensure_token(Token::Colon)?;
        let type_name = token_iter.next().ident()?;

        let member_type = TypeItem::Plain(BasicTypeItem {
            ident: Ident::new(type_name),
        });

        let member = StructTypeMember {
            visibility: Visibility::Inherited,
            ident: field,
            r#type: member_type,
        };

        members.push(member);
    }

    Ok(members)
}

fn parse_tuple_type_members(tokens: Vec<SpannedToken>) -> Result<Vec<TupleTypeMember>> {
    fn push_member(
        token_iter: &mut IntoIter<SpannedToken>,
        members: &mut Vec<TupleTypeMember>,
    ) -> Result<()> {
        // TODO: parse visibility modifiers and keywords here
        let type_name = token_iter.next().ident()?;
        let member_type = TypeItem::plain(type_name);
        let member = TupleTypeMember {
            visibility: Visibility::Inherited,
            r#type: member_type,
        };
        members.push(member);
        Ok(())
    }

    let mut token_iter = tokens.into_iter();
    let mut members = vec![];
    push_member(&mut token_iter, &mut members)?;
    // TODO: Also allow newlines here instead
    // TODO: Allow trailing commas and trailing newlines
    while token_iter.next().ensure_token(Token::Comma).is_ok() {
        push_member(&mut token_iter, &mut members)?;
    }

    Ok(members)
}

fn parse_type_alias(tokens: Vec<SpannedToken>) -> Result<TypeItem<BasicTypeItem>> {
    // TODO: parse more complex types such as Copy + Clone or Array types or dicts
    let mut token_iter = tokens.into_iter();
    let type_name = token_iter.next().ident()?;
    let member_type = TypeItem::plain(type_name);

    Ok(member_type)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_struct_type() -> DisplayResult<'static, ()> {
        // Note that type keyword is expected to be consumed already
        let src = "TypeA {
    member_b: TypeB
    member_c: TypeC
}";
        let mut tokenizer = Tokenizer::from_str(src);
        let parsed = parse_type(&mut tokenizer, &Modifiers::default()).leak_with_source(&src)?;

        assert!(matches!(parsed, TypeDecl::StructType(_)));

        Ok(())
    }

    #[test]
    fn test_parse_struct_type_members() -> DisplayResult<'static, ()> {
        let src = "
a: TypeA
b: TypeB";
        let tokenizer = Tokenizer::from_str(src);
        let tokens = tokenizer
            .map(|spanned_token| spanned_token.unpack())
            .collect::<Result<Vec<SpannedToken>>>()
            .leak_with_source(&src)?;
        let parsed = parse_struct_type_members(tokens).leak_with_source(&src)?;

        assert!(parsed.len() == 2);
        let a = &parsed[0];
        let b = &parsed[1];

        assert!(matches!(a.visibility, Visibility::Inherited));
        assert!(matches!(b.visibility, Visibility::Inherited));

        assert_eq!(a.ident, Ident::new("a".to_owned()));
        assert_eq!(b.ident, Ident::new("b".to_owned()));

        assert!(matches!(a.r#type, TypeItem::Plain(_)));
        assert!(matches!(b.r#type, TypeItem::Plain(_)));

        match a.r#type {
            TypeItem::Plain(ref inner) => assert_eq!(inner.ident, Ident::new("TypeA".to_owned())),
            _ => unreachable!(),
        }

        match b.r#type {
            TypeItem::Plain(ref inner) => assert_eq!(inner.ident, Ident::new("TypeB".to_owned())),
            _ => unreachable!(),
        }

        Ok(())
    }
}
