use crate::Token::*;
use plex::{lexer, parser};
use std::{collections::HashMap, env, fs};

// lexer
#[derive(Debug)]
pub enum Token {
    Whitespace,
    NewLine,
    Clear,
    Draw,
    Goto,
    Assign,
    Increment,
    Star,
    Ident(String),
    Colon,
    Register(u8),
    Int16(u16),
    Int8(u8),
    IRegister,
}

lexer! {
    fn next_token(tok: 'a) -> Token;

    r#"[ \t]+"# => Token::Whitespace,
    r#"\r\n"# => Token::NewLine,
    r#"clear"# => Token::Clear,
    r#"draw"# => Token::Draw,
    r#"goto"# => Token::Goto,
    r#"\+="# => Token::Increment,
    r#"="# => Token::Assign,
    r#"\*"# => Token::Star,
    r#":"# => Token::Colon,
    r#"i"# => Token::IRegister,

    r#"v[0-9a-f]"# => {
        let num = u8::from_str_radix(tok.trim_start_matches("v"), 16).unwrap();
        Token::Register(num)
    }

    r#"[0-9]+"# => {
        let num: u16 = tok.parse().unwrap();

        match num {
            0..=255 => Token::Int8(num as u8),
            0..=u16::MAX => Token::Int16(num as u16),
        }
    }

    r#"0x[0-9a-f]+"# => {
        let num = u16::from_str_radix(tok.trim_start_matches("0x"), 16).unwrap();
        match num {
            0..=255 => Token::Int8(num as u8),
            0..=u16::MAX => Token::Int16(num as u16),
        }
    }

    r#"[a-z]+"# => Token::Ident(tok.to_string()),

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

struct Props<'a> {
    pub pc: u16,
    pub ins: Vec<u8>,
    pub labels: HashMap<&'a str, u16>,
    pub line: usize,
}

// interp
pub fn interp<'a>(p: &'a Program) -> Vec<u8> {
    let mut props = Props {
        pc: 0,
        ins: vec![],
        labels: HashMap::new(),
        line: 0,
    };

    for expr in &p.statements {
        props.pc += 2;
        props.line += 1;
        interp_expr(&mut props, expr);
    }

    props.ins
}

fn interp_expr<'a>(props: &mut Props<'a>, expr: &'a Expr) {
    use crate::Expr_::*;

    match expr.node {
        Clear => {
            // 00E0
            props.ins.extend(vec![0x00, 0xE0]);
        }
        AssignRegisterRegister(ref r1, ref r2) => {
            // 8xy0
            props.ins.extend(vec![0x80 + r1, r2 << 4]);
        }
        AssignRegisterInteger(ref r, ref int) => {
            // 6xnn
            props.ins.extend(vec![0x60 + r, *int]);
        }
        AssignIRegisterInteger(ref int) => {
            // Annn
            let high_byte = 0xD0 + ((int & 0xF00) >> 8);
            let low_byte = int & 0x0FF;
            props.ins.extend(vec![high_byte as u8, low_byte as u8]);
        }
        AssignIRegisterRegisterSprite(ref r) => {
            // Fx1E
            props.ins.extend(vec![0xF0 + r, 0x29]);
        }
        DeclareLabel(ref id) => {
            props.labels.insert(id, props.pc);
            props.pc -= 2;
        }
        DrawIRegister(ref r1, ref r2, ref int) => {
            // Dxyn
            props.ins.extend(vec![0xD0 + r1, (r2 << 4) + int]);
        }
        IncrementRegisterInteger(ref r, ref int) => {
            // 7xnn
            props.ins.extend(vec![0x70 + r, *int]);
        }
        GotoLabel(ref id) => {
            // 1nnn
            let Some(pc) = props.labels.get(id.as_str()) else {
                panic!("line {}: could not find label {:?}", props.line, id);
            };
            let high_byte = 0x10 + ((pc & 0xF00) >> 8);
            let low_byte = (pc + 510) & 0x0FF;
            props.ins.extend(vec![high_byte as u8, low_byte as u8]);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_path = &args[1];
    let output_path = &args[2];

    let source = fs::read_to_string(input_path).unwrap();
    // println!("{:?}", source);

    let lexer = Lexer::new(source.as_str());
    // for (token, _) in lexer {
    // println!("{:?}", token);
    // }

    let program = parse(lexer).unwrap();
    // for st in &program.statements {
    // println!("{:?}", st.node);
    // }

    let ins = interp(&program);
    // println!("{:#04X?}", ins);

    fs::write(output_path, ins.clone()).unwrap();
    println!(
        "successfully compiled {} instructions to {}",
        ins.len(),
        output_path
    );
}
