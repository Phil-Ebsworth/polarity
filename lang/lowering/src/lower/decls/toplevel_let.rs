use ast::IdBind;
use parser::cst::{self};

use super::super::*;
use super::lower_telescope;

impl Lower for cst::decls::Let {
    type Target = ast::Let;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        log::trace!("Lowering top-level let: {}", self.name.id);

        let cst::decls::Let { span, doc, name, attr, params, typ, body } = self;

        lower_telescope(params, ctx, |ctx, params| {
            Ok(ast::Let {
                span: Some(*span),
                doc: doc.lower(ctx)?,
                name: IdBind { span: Some(name.span), id: name.id.clone() },
                attr: attr.lower(ctx)?,
                params,
                typ: typ.lower(ctx)?,
                body: body.lower(ctx)?,
            })
        })
    }
}
