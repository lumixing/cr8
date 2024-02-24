use plex::lexer;

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
