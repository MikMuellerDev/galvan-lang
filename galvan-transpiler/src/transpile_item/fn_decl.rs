use crate::context::Context;
use crate::macros::{impl_transpile, transpile};
use crate::transpile_item::ident::Ownership;
use crate::{FnDecl, FnSignature, Param, ParamList, Transpile};
use galvan_ast::{DeclModifier, TypeElement};
use galvan_resolver::Scope;

impl_transpile!(FnDecl, "{} {}", signature, block);

impl Transpile for FnSignature {
    fn transpile(&self, ctx: &Context, scope: &mut Scope) -> String {
        let visibility = self.visibility.transpile(ctx, scope);
        let identifier = self.identifier.transpile(ctx, scope);
        let parameters = self.parameters.transpile(ctx, scope);
        format!(
            "{} fn {}{}{}",
            visibility,
            identifier,
            parameters,
            self.return_type
                .as_ref()
                .map_or("".into(), |return_type| transpile!(
                    ctx,
                    scope,
                    " -> {}",
                    return_type
                ))
        )
    }
}

impl_transpile!(ParamList, "({})", params);

macro_rules! transpile_type {
    ($self:ident, $ctx:ident, $scope:ident, $ownership:path, $prefix:expr) => {{
        use crate::transpile_item::ident::TranspileType;
        let ty = match &$self.param_type {
            TypeElement::Plain(plain) => plain.ident.transpile_type($ctx, $scope, $ownership),
            other => other.transpile($ctx, $scope),
        };

        transpile!($ctx, $scope, "{}: {} {}", &$self.identifier, $prefix, ty)
    }};

    ($self:ident, $ctx:ident, $scope:ident, $ownership:path, $prefix:expr, $prefix_copy:expr) => {{
        use crate::transpile_item::ident::TranspileType;
        let (prefix, ty) = match &$self.param_type {
            TypeElement::Plain(plain) => (
                if $ctx.mapping.is_copy(&plain.ident) {
                    $prefix_copy
                } else {
                    $prefix
                },
                plain.ident.transpile_type($ctx, $scope, $ownership),
            ),
            other => ($prefix, other.transpile($ctx, $scope)),
        };

        transpile!($ctx, $scope, "{}: {} {}", &$self.identifier, prefix, ty)
    }};
}

impl Transpile for Param {
    fn transpile(&self, ctx: &Context, scope: &mut Scope) -> String {
        let is_self = self.identifier.as_str() == "self";

        match self.decl_modifier {
            Some(DeclModifier::Let(_)) | None => {
                if is_self {
                    "&self".into()
                } else {
                    transpile_type!(self, ctx, scope, Ownership::Borrowed, "&", "")
                }
            }
            Some(DeclModifier::Mut(_)) => {
                if is_self {
                    "&mut self".into()
                } else {
                    transpile_type!(self, ctx, scope, Ownership::MutBorrowed, "&mut")
                }
            }
            Some(DeclModifier::Ref(_)) => {
                if is_self {
                    panic!("Functions with ref-receivers should be handled elsewhere!")
                }

                transpile!(
                    ctx,
                    scope,
                    "{}: std::sync::Arc<std::sync::Mutex<{}>>",
                    self.identifier,
                    self.param_type
                )
            }
        }
    }
}
