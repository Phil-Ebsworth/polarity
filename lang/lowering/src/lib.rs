mod ctx;
mod downsweep;
mod lower;
mod result;

use parser::cst;
use syntax::ast;

use crate::downsweep::build_lookup_table;
use crate::lower::Lower;
pub use ctx::*;
pub use result::*;

pub fn lower_module(prg: &cst::decls::Module) -> Result<ast::Module, LoweringError> {
    let cst::decls::Module { items } = prg;

    let (top_level_map, lookup_table) = build_lookup_table(items)?;

    let mut ctx = Ctx::empty(top_level_map);
    // Lower definitions
    for item in items {
        item.lower(&mut ctx)?;
    }

    Ok(ast::Module { map: ctx.decls_map, lookup_table })
}
