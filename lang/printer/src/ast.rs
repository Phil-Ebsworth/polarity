use std::rc::Rc;

use pretty::DocAllocator;

use syntax::ast::*;
use syntax::common::*;

use super::theme::ThemeExt;
use super::tokens::*;
use super::types::*;
use super::util::*;

impl<'a, P: Phase> Print<'a> for Prg<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Prg { decls, exp } = self;

        match exp {
            Some(exp) => {
                let top = if decls.is_empty() {
                    alloc.nil()
                } else {
                    let sep = if cfg.omit_decl_sep {
                        alloc.hardline()
                    } else {
                        alloc.hardline().append(alloc.hardline())
                    };
                    decls.print(cfg, alloc).append(sep)
                };
                top.append(exp.print(cfg, alloc))
            }
            None => decls.print(cfg, alloc),
        }
    }
}

impl<'a, P: Phase> Print<'a> for Decls<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let items = self.iter().map(|item| match item {
            Item::Data(data) => data.print_in_ctx(cfg, self, alloc),
            Item::Codata(codata) => codata.print_in_ctx(cfg, self, alloc),
            Item::Def(def) => def.print(cfg, alloc),
            Item::Codef(codef) => codef.print(cfg, alloc),
        });

        let sep = if cfg.omit_decl_sep { alloc.line() } else { alloc.line().append(alloc.line()) };
        alloc.intersperse(items, sep)
    }
}

impl<'a, P: Phase> PrintInCtx<'a> for Decl<P>
where
    P::InfTyp: ShiftInRange,
{
    type Ctx = Decls<P>;

    fn print_in_ctx(
        &'a self,
        cfg: &PrintCfg,
        ctx: &'a Self::Ctx,
        alloc: &'a Alloc<'a>,
    ) -> Builder<'a> {
        match self {
            Decl::Data(data) => data.print_in_ctx(cfg, ctx, alloc),
            Decl::Codata(codata) => codata.print_in_ctx(cfg, ctx, alloc),
            Decl::Def(def) => def.print(cfg, alloc),
            Decl::Codef(codef) => codef.print(cfg, alloc),
            Decl::Ctor(ctor) => ctor.print(cfg, alloc),
            Decl::Dtor(dtor) => dtor.print(cfg, alloc),
        }
    }
}

impl<'a, P: Phase> PrintInCtx<'a> for Item<'a, P>
where
    P::InfTyp: ShiftInRange,
{
    type Ctx = Decls<P>;

    fn print_in_ctx(
        &'a self,
        cfg: &PrintCfg,
        ctx: &'a Self::Ctx,
        alloc: &'a Alloc<'a>,
    ) -> Builder<'a> {
        match self {
            Item::Data(data) => data.print_in_ctx(cfg, ctx, alloc),
            Item::Codata(codata) => codata.print_in_ctx(cfg, ctx, alloc),
            Item::Def(def) => def.print(cfg, alloc),
            Item::Codef(codef) => codef.print(cfg, alloc),
        }
    }
}

impl<'a, P: Phase> PrintInCtx<'a> for Data<P>
where
    P::InfTyp: ShiftInRange,
{
    type Ctx = Decls<P>;

    fn print_in_ctx(
        &'a self,
        cfg: &PrintCfg,
        ctx: &'a Self::Ctx,
        alloc: &'a Alloc<'a>,
    ) -> Builder<'a> {
        let Data { info: _, name, typ, ctors } = self;

        let head = alloc
            .keyword(DATA)
            .append(alloc.space())
            .append(alloc.typ(name))
            .append(typ.params.print(cfg, alloc))
            .append(alloc.space());

        let sep = alloc.text(COMMA).append(alloc.hardline());

        let body =
            alloc
                .hardline()
                .append(alloc.intersperse(
                    ctors.iter().map(|x| ctx.map[x].print_in_ctx(cfg, ctx, alloc)),
                    sep,
                ))
                .nest(INDENT)
                .append(alloc.hardline())
                .braces_from(cfg);

        head.append(body)
    }
}

impl<'a, P: Phase> PrintInCtx<'a> for Codata<P>
where
    P::InfTyp: ShiftInRange,
{
    type Ctx = Decls<P>;

    fn print_in_ctx(
        &'a self,
        cfg: &PrintCfg,
        ctx: &'a Self::Ctx,
        alloc: &'a Alloc<'a>,
    ) -> Builder<'a> {
        let Codata { info: _, name, typ, dtors } = self;
        let head = alloc
            .keyword(CODATA)
            .append(alloc.space())
            .append(alloc.typ(name))
            .append(typ.params.print(cfg, alloc))
            .append(alloc.space());

        let sep = alloc.text(COMMA).append(alloc.hardline());

        let body =
            alloc
                .hardline()
                .append(alloc.intersperse(
                    dtors.iter().map(|x| ctx.map[x].print_in_ctx(cfg, ctx, alloc)),
                    sep,
                ))
                .nest(INDENT)
                .append(alloc.hardline())
                .braces_from(cfg);

        head.append(body)
    }
}

impl<'a, P: Phase> Print<'a> for Def<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Def { info: _, name, params, self_param, ret_typ, body } = self;
        let head = alloc
            .keyword(DEF)
            .append(alloc.space())
            .append(self_param.print(cfg, alloc))
            .append(DOT)
            .append(alloc.dtor(name))
            .append(params.print(cfg, alloc))
            .append(alloc.space())
            .append(COLON)
            .append(alloc.space())
            .append(ret_typ.print(cfg, alloc));

        let body = body.print(cfg, alloc);

        head.append(alloc.space()).append(body)
    }
}

impl<'a, P: Phase> Print<'a> for Codef<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Codef { info: _, name, params, typ, body } = self;
        let head = alloc
            .keyword(CODEF)
            .append(alloc.space())
            .append(alloc.ctor(name))
            .append(params.print(cfg, alloc))
            .append(alloc.space())
            .append(COLON)
            .append(alloc.space())
            .append(typ.print(cfg, alloc));

        let body = body.print(cfg, alloc);

        head.append(alloc.space()).append(body)
    }
}

impl<'a, P: Phase> Print<'a> for Ctor<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Ctor { info: _, name, params, typ } = self;
        alloc
            .ctor(name)
            .append(params.print(cfg, alloc))
            .append(alloc.space())
            .append(COLON)
            .append(alloc.space())
            .append(typ.print(cfg, alloc))
    }
}

impl<'a, P: Phase> Print<'a> for Dtor<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Dtor { info: _, name, params, self_param, ret_typ } = self;
        self_param
            .print(cfg, alloc)
            .append(DOT)
            .append(alloc.dtor(name))
            .append(params.print(cfg, alloc))
            .append(alloc.space())
            .append(COLON)
            .append(alloc.space())
            .append(ret_typ.print(cfg, alloc))
    }
}

impl<'a, P: Phase> Print<'a> for Comatch<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Comatch { info: _, cases } = self;
        let sep = alloc.text(COMMA).append(alloc.hardline());

        alloc
            .hardline()
            .append(alloc.intersperse(cases.iter().map(|x| x.print(cfg, alloc)), sep))
            .nest(INDENT)
            .append(alloc.hardline())
            .braces_from(cfg)
    }
}

impl<'a, P: Phase> Print<'a> for Match<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Match { info: _, cases } = self;
        let sep = alloc.text(COMMA).append(alloc.hardline());
        alloc
            .hardline()
            .append(alloc.intersperse(cases.iter().map(|x| x.print(cfg, alloc)), sep))
            .nest(INDENT)
            .append(alloc.hardline())
            .braces_from(cfg)
    }
}

impl<'a, P: Phase> Print<'a> for Case<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Case { info: _, name, args, body } = self;

        let body = match body {
            None => alloc.keyword(ABSURD),
            Some(body) => alloc
                .text(FAT_ARROW)
                .append(alloc.line())
                .append(body.print(cfg, alloc))
                .nest(INDENT),
        };

        alloc.ctor(name).append(args.print(cfg, alloc)).append(alloc.space()).append(body).group()
    }
}

impl<'a, P: Phase> Print<'a> for Cocase<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Cocase { info: _, name, params: args, body } = self;

        let body = match body {
            None => alloc.keyword(ABSURD),
            Some(body) => alloc
                .text(FAT_ARROW)
                .append(alloc.line())
                .append(body.print(cfg, alloc))
                .nest(INDENT),
        };

        alloc.ctor(name).append(args.print(cfg, alloc)).append(alloc.space()).append(body).group()
    }
}

impl<'a, P: Phase> Print<'a> for Telescope<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Telescope { params } = self;
        let mut output = alloc.nil();
        let mut running_type: Option<&Rc<Exp<P>>> = None;
        for Param { name, typ } in params {
            match running_type {
                // We need to shift before comparing to ensure we compare the correct De-Bruijn indices
                Some(rtype) if &rtype.shift((0, 1)) == typ => {
                    // We are adding another parameter of the same type.
                    output = output.append(alloc.space()).append(alloc.text(name));
                }
                Some(rtype) => {
                    // We are adding another parameter with a different type,
                    // and have to close the previous list first.
                    output = output
                        .append(COLON)
                        .append(alloc.space())
                        .append(rtype.print(cfg, alloc))
                        .append(COMMA)
                        .append(alloc.space());
                    output = output.append(alloc.text(name));
                }
                None => {
                    // We are adding the very first parameter.
                    output = output.append(alloc.text(name));
                }
            }
            running_type = Some(typ);
        }
        // Close the last parameter
        match running_type {
            None => {}
            Some(rtype) => {
                output = output.append(COLON).append(alloc.space()).append(rtype.print(cfg, alloc));
            }
        }
        output.opt_parens()
    }
}

impl<'a, P: Phase> Print<'a> for TelescopeInst<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        self.params.print(cfg, alloc).opt_parens()
    }
}

impl<'a, P: Phase> Print<'a> for Param<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Param { name, typ } = self;
        alloc.text(name).append(COLON).append(alloc.space()).append(typ.print(cfg, alloc))
    }
}

impl<'a, P: Phase> Print<'a> for ParamInst<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let ParamInst { info: _, name, typ: _ } = self;
        alloc.text(name)
    }
}

impl<'a, P: Phase> Print<'a> for SelfParam<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let SelfParam { info: _, name, typ } = self;

        match name {
            Some(name) => alloc
                .text(name)
                .append(COLON)
                .append(alloc.space())
                .append(typ.print(cfg, alloc))
                .parens(),
            None => typ.print(cfg, alloc),
        }
    }
}

impl<'a, P: Phase> Print<'a> for TypApp<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let TypApp { info: _, name, args: subst } = self;
        alloc.typ(name).append(subst.print(cfg, alloc).opt_parens())
    }
}

impl<'a, P: Phase> Print<'a> for Exp<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Exp::Var { info: _, name, idx } => {
                alloc.text(P::print_var(name, cfg.de_bruijn.then_some(*idx)))
            }
            Exp::TypCtor { info: _, name, args: subst } => {
                alloc.typ(name).append(subst.print(cfg, alloc).opt_parens())
            }
            Exp::Ctor { info: _, name, args: subst } => {
                alloc.ctor(name).append(subst.print(cfg, alloc).opt_parens())
            }
            Exp::Dtor { info: _, exp, name, args: subst } => exp
                .print(cfg, alloc)
                .append(DOT)
                .append(alloc.dtor(name))
                .append(subst.print(cfg, alloc).opt_parens()),
            Exp::Anno { info: _, exp, typ } => {
                exp.print(cfg, alloc).parens().append(COLON).append(typ.print(cfg, alloc))
            }
            Exp::Type { info: _ } => alloc.typ(TYPE),
            Exp::Match { info: _, name, on_exp, motive, ret_typ: _, body } => on_exp
                .print(cfg, alloc)
                .append(DOT)
                .append(alloc.keyword(MATCH))
                .append(alloc.space())
                .append(alloc.text(name))
                .append(motive.as_ref().map(|m| m.print(cfg, alloc)).unwrap_or(alloc.nil()))
                .append(alloc.space())
                .append(body.print(cfg, alloc)),
            Exp::Comatch { info: _, name, body } => alloc
                .keyword(COMATCH)
                .append(alloc.space())
                .append(alloc.text(name))
                .append(alloc.space())
                .append(body.print(cfg, alloc)),
            Exp::Hole { info: _ } => alloc.keyword(HOLE),
        }
    }
}

impl<'a, P: Phase> Print<'a> for Motive<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Motive { info: _, name, ret_typ } = self;

        alloc
            .space()
            .append(alloc.keyword(AS))
            .append(alloc.space())
            .append(alloc.text(name))
            .append(alloc.text(FAT_ARROW))
            .append(ret_typ.print(cfg, alloc))
    }
}

impl<'a, T: Print<'a>> Print<'a> for Rc<T> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        T::print(self, cfg, alloc)
    }
}

fn print_comma_separated<'a, T: Print<'a>>(
    vec: &'a Vec<T>,
    cfg: &PrintCfg,
    alloc: &'a Alloc<'a>,
) -> Builder<'a> {
    if vec.is_empty() {
        alloc.nil()
    } else {
        let sep = alloc.text(COMMA).append(alloc.space());
        alloc.intersperse(vec.iter().map(|x| x.print(cfg, alloc)), sep)
    }
}

impl<'a, T: Print<'a>> Print<'a> for Vec<T> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        print_comma_separated(self, cfg, alloc)
    }
}
