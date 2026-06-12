use crate::{
    ast,
    context::Context,
    error::{Report, error_span},
    module::{Namespace, Symbol, SymbolList},
    types::TypeId,
};

pub(crate) trait CheckerHelpers<'a> {
    fn ctx(&self) -> &Context;
    fn symbols(&self) -> &SymbolList;
    fn get_namespace(&self, name: &str) -> Option<&Namespace>;

    /// Get a declared symbol. Returns not declared error message if not found.
    fn get_symbol(&self, name: &str) -> Result<&Symbol, String> {
        self.symbols()
            .get(name)
            .map_or(Err("not declared".to_string()), |sym| {
                Ok(self.ctx().symbols.get(sym.id))
            })
    }

    /// Get the TypeId of a declared symbol.
    fn get_symbol_type_id(&self, name: &str) -> Option<TypeId> {
        self.get_symbol(name).ok().map(|sym| sym.ty)
    }

    /// Evaluate an AST type node to its semantic type id.
    fn eval_type(&self, node: &ast::TypeNode) -> Result<TypeId, Report> {
        match node {
            ast::TypeNode::Ident(token) => self
                .get_symbol_type_id(&token.to_string())
                .ok_or(error_span("not a type", token)),
            ast::TypeNode::Imported { namespace, ty } => {
                let ns = self
                    .get_namespace(&namespace.to_string())
                    .map_or(Err(error_span("not an imported namespace", namespace)), Ok)?;

                let sym_id = ns.get(&ty.to_string()).ok_or(error_span(
                    &format!("namespace '{namespace}' has no member '{ty}'"),
                    ty,
                ))?;

                Ok(self.ctx().symbols.get(sym_id).ty)
            }
        }
    }
}
