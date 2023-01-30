use std::rc::Rc;

use derivative::Derivative;

use crate::common::*;
use crate::env::*;
use crate::ust;

/// The result of evaluation
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum Val {
    TypCtor {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Info,
        name: Ident,
        args: Args,
    },
    Ctor {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Info,
        name: Ident,
        args: Args,
    },
    Type {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Info,
    },
    Comatch {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Info,
        name: Ident,
        // TODO: Ignore this field for PartialEq, Hash?
        body: Comatch,
    },
    Neu {
        exp: Neu,
    },
}

/// A term whose evaluation is blocked
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum Neu {
    Var {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Info,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        name: Ident,
        idx: Idx,
    },
    Dtor {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: ust::Info,
        exp: Rc<Neu>,
        name: Ident,
        args: Args,
    },
    Match {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Info,
        name: Ident,
        on_exp: Rc<Neu>,
        // TODO: Ignore this field for PartialEq, Hash?
        body: Match,
    },
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Match {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Info,
    pub cases: Vec<Case>,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Comatch {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Info,
    // TODO: Consider renaming this field to "cocases"
    pub cases: Vec<Cocase>,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Case {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Info,
    pub name: Ident,
    // TODO: Rename to params
    pub args: ust::TelescopeInst,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Closure>,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Cocase {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Info,
    pub name: Ident,
    // TODO: Rename to params
    pub args: ust::TelescopeInst,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Closure>,
}

#[derive(Debug, Clone)]
pub struct TypApp {
    pub info: Info,
    pub name: Ident,
    pub args: Args,
}

pub type Info = ust::Info;
pub type Args = Vec<Rc<Val>>;

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Closure {
    pub env: Env,
    pub body: Rc<ust::Exp>,
}
