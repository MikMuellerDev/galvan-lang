use derive_more::From;
use galvan_lexer::LexerString;

use crate::*;

#[derive(Debug)]
pub enum TypeDecl {
    TupleType(TupleTypeDecl),
    StructType(StructTypeDecl),
    AliasType(AliasTypeDecl),
}

#[derive(Debug)]
pub struct TupleTypeDecl {
    pub members: Vec<TupleTypeMember>,
}

#[derive(Debug)]
pub struct TupleTypeMember {
    pub visibility: Visibility,
    pub r#type: TypeItem,
}

#[derive(Debug)]
pub struct StructTypeDecl {
    pub members: Vec<StructTypeMember>,
}
#[derive(Debug)]
pub struct StructTypeMember {
    pub visibility: Visibility,
    pub ident: Ident,
    pub r#type: TypeItem,
}

#[derive(Debug)]
pub struct AliasTypeDecl {
    pub r#type: TypeItem,
}

#[derive(Debug, From)]
pub enum TypeItem {
    // #[from(forward)]
    Array(Box<ArrayTypeItem>),
    // #[from(forward)]
    Dictionary(Box<DictionaryTypeItem>),
    // #[from(forward)]
    Set(Box<SetTypeItem>),
    // #[from(forward)]
    Tuple(Box<TupleTypeItem>),
    Plain(BasicTypeItem),
}

impl From<Ident> for TypeItem {
    fn from(value: Ident) -> Self {
        Self::Plain(BasicTypeItem { ident: value })
    }
}

impl TypeItem {
    pub fn plain(ident: LexerString) -> Self {
        Self::Plain(BasicTypeItem {
            ident: Ident::new(ident),
        })
    }

    pub fn array(elements: TypeItem) -> Self {
        Self::Array(Box::new(ArrayTypeItem { elements }))
    }

    pub fn dict(key: TypeItem, value: TypeItem) -> Self {
        Self::Dictionary(Box::new(DictionaryTypeItem { key, value }))
    }

    pub fn set(elements: TypeItem) -> Self {
        Self::Set(Box::new(SetTypeItem { elements }))
    }

    pub fn tuple(elements: Vec<TypeItem>) -> Self {
        Self::Tuple(Box::new(TupleTypeItem { elements }))
    }
}

// TODO: Add a marker trait to constrain this to only type decls
#[derive(Debug)]
pub struct ArrayTypeItem {
    pub elements: TypeItem,
}

#[derive(Debug)]
pub struct DictionaryTypeItem {
    pub key: TypeItem,
    pub value: TypeItem,
}

#[derive(Debug)]
pub struct SetTypeItem {
    pub elements: TypeItem,
}

#[derive(Debug)]
pub struct TupleTypeItem {
    pub elements: Vec<TypeItem>,
}

#[derive(Debug)]
pub struct BasicTypeItem {
    pub ident: Ident,
    // TODO: Handle generics
}

#[derive(Debug)]
pub struct ReceiverType {}
#[derive(Debug)]
pub struct ReturnType {}
#[derive(Debug)]
pub struct ParamType {}
