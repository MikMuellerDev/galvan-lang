use crate::context::Context;
use crate::macros::{impl_transpile, impl_transpile_match, transpile};
use crate::Transpile;
use galvan_ast::{
    ConstructorCall, ConstructorCallArg, DeclModifier, Expression, FunctionCall, FunctionCallArg,
    MemberFieldAccess, MemberFunctionCall,
};

impl Transpile for FunctionCall {
    fn transpile(&self, ctx: &Context) -> String {
        let arguments = self.arguments.transpile(ctx);

        // TODO: Resolve function and check argument types + check if they should be submitted as &, &mut or Arc<Mutex>
        if self.identifier.as_str() == "println" {
            format!("println!(\"{{}}\", {})", arguments)
        } else if self.identifier.as_str() == "print" {
            format!("print!(\"{{}}\", {})", arguments)
        } else if self.identifier.as_str() == "debug" {
            format!("println!(\"{{:?}}\", {})", arguments)
        } else {
            let ident = self.identifier.transpile(ctx);
            format!("{}({})", ident, arguments)
        }
    }
}

impl Transpile for FunctionCallArg {
    fn transpile(&self, ctx: &Context) -> String {
        use DeclModifier as Mod;
        use Expression as Exp;
        let Self {
            modifier,
            expression,
        } = self;
        match (modifier, expression) {
            (Some(Mod::Let(_)), _) => {
                todo!("TRANSPILER ERROR: Let modifier is not allowed for function call arguments")
            }
            (None, expr @ Exp::Ident(_)) => {
                transpile!(ctx, "&(&{}).__borrow()", expr)
            }
            (None, expression) => {
                transpile!(ctx, "&({})", expression)
            }
            (Some(Mod::Mut(_)), expr @ Exp::MemberFieldAccess(_) | expr @ Exp::Ident(_)) => {
                transpile!(ctx, "&mut {}", expr)
            }
            (Some(Mod::Ref(_)), expr @ Exp::MemberFieldAccess(_) | expr @ Exp::Ident(_)) => {
                transpile!(ctx, "::std::sync::Arc::clone(&{})", expr)
            }
            _ => todo!("TRANSPILER ERROR: Modifier only allowed for fields or variables"),
        }
    }
}

impl_transpile!(
    MemberFunctionCall,
    "{}.{}({})",
    receiver,
    identifier,
    arguments
);
impl_transpile!(MemberFieldAccess, "{}.{}", receiver, identifier);

impl_transpile!(ConstructorCall, "{} {{ {} }}", identifier, arguments,);
impl_transpile!(ConstructorCallArg, "{}: {}", ident, expression);
