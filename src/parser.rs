use plex::parser;

use crate::lexer::{
    Span,
    Token::{self, *},
};

#[derive(Debug)]
pub enum Expr_ {
    Clear,
    AssignRegisterRegister(u8, u8),
    AssignRegisterInteger(u8, u8),
    AssignIRegisterInteger(u16),
    AssignIRegisterRegisterSprite(u8),
    DeclareLabel(String),
    DrawIRegister(u8, u8, u8),
    IncrementRegisterInteger(u8, u8),
    GotoLabel(String),
}

#[derive(Debug)]
pub struct Expr {
    pub span: Span,
    pub node: Expr_,
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Expr>,
}

parser! {
    fn parse_(Token, Span);

    (a, b) {
        Span {
            lo: a.lo,
            hi: b.hi
        }
    }

    program: Program {
        statements[s] => Program {
            statements: s
        }
    }

    statements: Vec<Expr> {
        => vec![],
        statements[mut st] statement[e] NewLine => {
            st.push(e);
            st
        },
        statements[st] NewLine => {
            st
        }
    }

    statement: Expr {
        Clear => Expr {
            span: span!(),
            node: Expr_::Clear
        },
        Register(r1) Assign Register(r2) => Expr {
            span: span!(),
            node: Expr_::AssignRegisterRegister(r1, r2)
        },
        Register(r) Assign Int8(i) => Expr {
            span: span!(),
            node: Expr_::AssignRegisterInteger(r, i)
        },
        IRegister Assign Int16(i) => Expr {
            span: span!(),
            node: Expr_::AssignIRegisterInteger(i)
        },
        IRegister Assign Star Register(r) => Expr {
            span: span!(),
            node: Expr_::AssignIRegisterRegisterSprite(r)
        },
        Ident(id) Colon => Expr {
            span: span!(),
            node: Expr_::DeclareLabel(id)
        },
        Draw Register(r1) Register(r2) Int8(i) => Expr {
            span: span!(),
            node: Expr_::DrawIRegister(r1, r2, i)
        },
        Register(r) Increment Int8(i) => Expr {
            span: span!(),
            node: Expr_::IncrementRegisterInteger(r, i)
        },
        Goto Ident(id) => Expr {
            span: span!(),
            node: Expr_::GotoLabel(id)
        }
    }

    nop: () {
        NewLine => {}
    }
}

pub fn parse<I: Iterator<Item = (Token, Span)>>(
    i: I,
) -> Result<Program, (Option<(Token, Span)>, &'static str)> {
    parse_(i)
}
