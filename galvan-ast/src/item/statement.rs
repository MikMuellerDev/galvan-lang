use super::*;
use galvan_pest::Rule;
use typeunion::type_union;

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::body))]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[type_union]
#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::statement))]
pub type Statement = Assignment + Expression + Declaration;

#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::declaration))]
pub struct Declaration {
    pub decl_modifier: DeclModifier,
    pub identifier: Ident,
    pub type_annotation: Option<TypeElement>,
    pub expression: Option<Expression>,
}

#[type_union]
#[derive(Debug, PartialEq, Eq, FromPest)]
#[pest_ast(rule(Rule::expression))]
pub type Expression = FunctionCall
    + ConstructorCall
    + MemberFunctionCall
    + MemberFieldAccess
    + StringLiteral
    + NumberLiteral
    + Ident;
