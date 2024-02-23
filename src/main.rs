use crate::Token::*;
use std::{collections::HashMap, fs};

use plex::{lexer, parser};

// lexer
#[derive(Debug)]
pub enum Token {
    Whitespace,
    NewLine,
    Clear,
    Move,
    Register(u8),
    Integer(u16),
    Hex(u16),
    Comma,
}

lexer! {
    fn next_token(tok: 'a) -> Token;

    r#"\r\n"# => Token::NewLine,
    r#"[ \t]+"# => Token::Whitespace,
    r#","# => Token::Comma,

    r#"cls"# => Token::Clear,
    r#"mov"# => Token::Move,

    r#"v[0-9a-f]"# => {
        let num = u8::from_str_radix(tok.trim_start_matches("v"), 16).unwrap();
        Token::Register(num)
    }

    r#"[0-9]+"# => {
        let num = tok.parse().unwrap();
        Token::Integer(num)
    }

    r#"0x[0-9a-f]+"# => {
        let num = u16::from_str_radix(tok.trim_start_matches("0x"), 16).unwrap();
        Token::Hex(num)
    }

    r#"."# => panic!("invalid token: {tok}")
}

pub struct Lexer<'a> {
    original: &'a str,
    remaining: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(s: &'a str) -> Lexer<'a> {
        Lexer {
            original: s,
            remaining: s,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub lo: usize,
    pub hi: usize,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = (Token, Span);
    fn next(&mut self) -> Option<(Token, Span)> {
        loop {
            let (tok, span) = if let Some((tok, new_remaining)) = next_token(self.remaining) {
                let lo = self.original.len() - self.remaining.len();
                let hi = self.original.len() - new_remaining.len();
                self.remaining = new_remaining;
                (tok, Span { lo, hi })
            } else {
                return None;
            };
            match tok {
                Token::Whitespace => {
                    continue;
                }
                tok => {
                    return Some((tok, span));
                }
            }
        }
    }
}

// ast
#[derive(Debug)]
pub enum Expr_ {
    Number(u16),
    MoveRegisterRegister(u8, u8),
    MoveRegisterInteger(u8, Box<Expr>),
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

// parser
#[allow(clippy::filter_map_next)]
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
        statements[mut st] mov[e] NewLine => {
            st.push(e);
            st
        },
        statements[st] NewLine => {
            st
        }
    }

    nop: () {
        NewLine => {}
    }

    mov: Expr {
        Move Register(r1) Comma Register(r2) => Expr {
            span: span!(),
            node: Expr_::MoveRegisterRegister(r1, r2)
        },
        Move Register(r) Comma num[int] => Expr {
            span: span!(),
            node: Expr_::MoveRegisterInteger(r, Box::new(int))
        }
    }

    num: Expr {
        Integer(int) => Expr {
            span: span!(),
            node: Expr_::Number(int)
        },
        Hex(int) => Expr {
            span: span!(),
            node: Expr_::Number(int)
        }
    }
}

pub fn parse<I: Iterator<Item = (Token, Span)>>(
    i: I,
) -> Result<Program, (Option<(Token, Span)>, &'static str)> {
    parse_(i)
}

// interp
pub fn interp<'a>(p: &'a Program) {
    let mut env = HashMap::new();
    for expr in &p.statements {
        interp_expr(&mut env, expr);
    }
}

fn interp_expr<'a>(env: &mut HashMap<&'a str, i64>, expr: &'a Expr) -> i64 {
    use crate::Expr_::*;

    match expr.node {
        Number(ref a) => *a as i64,
        MoveRegisterRegister(ref a, ref b) => {
            println!("mov v{} v{}", a, b);
            0
        }
        MoveRegisterInteger(ref a, ref b) => {
            println!("mov v{} {}", a, interp_expr(env, b));
            0
        }
    }
}

fn main() {
    let source = fs::read_to_string("test.asm").unwrap();
    // println!("{:?}", source);
    let lexer = Lexer::new(source.as_str());
    // for (token, _) in lexer {
    // println!("{:?}", token);
    // }
    let program = parse(lexer).unwrap();
    for st in &program.statements {
        println!("{:?}", st.node);
    }
    interp(&program);
}
