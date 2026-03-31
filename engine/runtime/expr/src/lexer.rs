use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // literals
    Int(i64),
    Float(f64),
    Str(String),
    Ident(String), // includes true, false, null, in
    // punctuation
    LBracket,
    RBracket,
    LParen,
    RParen,
    Comma,
    ColonColon,
    // arithmetic
    Plus,
    Minus,
    Star,
    Slash,
    // comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // logical
    And,
    Or,
    Bang,
    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self { input: input.chars().collect(), pos: 0 }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    fn current(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.current(), Some(c) if c.is_whitespace()) {
            self.advance();
        }
    }

    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();
        let pos = self.pos;

        match self.current() {
            None => Ok(Token::Eof),
            Some(ch) => match ch {
                '[' => { self.advance(); Ok(Token::LBracket) }
                ']' => { self.advance(); Ok(Token::RBracket) }
                '(' => { self.advance(); Ok(Token::LParen) }
                ')' => { self.advance(); Ok(Token::RParen) }
                ',' => { self.advance(); Ok(Token::Comma) }
                '+' => { self.advance(); Ok(Token::Plus) }
                '-' => { self.advance(); Ok(Token::Minus) }
                '*' => { self.advance(); Ok(Token::Star) }
                '/' => { self.advance(); Ok(Token::Slash) }
                '<' => {
                    self.advance();
                    if self.current() == Some('=') { self.advance(); Ok(Token::Le) }
                    else { Ok(Token::Lt) }
                }
                '>' => {
                    self.advance();
                    if self.current() == Some('=') { self.advance(); Ok(Token::Ge) }
                    else { Ok(Token::Gt) }
                }
                '!' => {
                    self.advance();
                    if self.current() == Some('=') { self.advance(); Ok(Token::Ne) }
                    else { Ok(Token::Bang) }
                }
                '=' => {
                    self.advance();
                    if self.current() == Some('=') { self.advance(); Ok(Token::Eq) }
                    else { Err(Error::Lex { pos, msg: "unexpected '=', did you mean '=='?".into() }) }
                }
                '&' => {
                    self.advance();
                    if self.current() == Some('&') { self.advance(); Ok(Token::And) }
                    else { Err(Error::Lex { pos, msg: "expected '&&'".into() }) }
                }
                '|' => {
                    self.advance();
                    if self.current() == Some('|') { self.advance(); Ok(Token::Or) }
                    else { Err(Error::Lex { pos, msg: "expected '||'".into() }) }
                }
                ':' => {
                    self.advance();
                    if self.current() == Some(':') { self.advance(); Ok(Token::ColonColon) }
                    else { Err(Error::Lex { pos, msg: "expected '::'".into() }) }
                }
                '"' | '\'' => self.lex_string(ch),
                c if c.is_ascii_digit() => self.lex_number(),
                c if c.is_alphabetic() || c == '_' => self.lex_ident(),
                c => Err(Error::Lex { pos, msg: format!("unexpected character '{c}'") }),
            }
        }
    }

    fn lex_string(&mut self, quote: char) -> Result<Token> {
        self.advance(); // opening quote
        let mut s = String::new();
        loop {
            match self.current() {
                None => return Err(Error::Lex { pos: self.pos, msg: "unterminated string".into() }),
                Some(c) if c == quote => { self.advance(); break; }
                Some('\\') => {
                    self.advance();
                    match self.current() {
                        Some('n') => { s.push('\n'); self.advance(); }
                        Some('t') => { s.push('\t'); self.advance(); }
                        Some('\\') => { s.push('\\'); self.advance(); }
                        Some(c) if c == quote => { s.push(c); self.advance(); }
                        _ => return Err(Error::Lex { pos: self.pos, msg: "invalid escape sequence".into() }),
                    }
                }
                Some(c) => { s.push(c); self.advance(); }
            }
        }
        Ok(Token::Str(s))
    }

    fn lex_number(&mut self) -> Result<Token> {
        let start = self.pos;
        while matches!(self.current(), Some(c) if c.is_ascii_digit()) {
            self.advance();
        }
        // float if followed by '.' and another digit
        if self.current() == Some('.')
            && matches!(self.input.get(self.pos + 1), Some(c) if c.is_ascii_digit())
        {
            self.advance();
            while matches!(self.current(), Some(c) if c.is_ascii_digit()) {
                self.advance();
            }
            let s: String = self.input[start..self.pos].iter().collect();
            s.parse::<f64>()
                .map(Token::Float)
                .map_err(|_| Error::Lex { pos: start, msg: format!("invalid float '{s}'") })
        } else {
            let s: String = self.input[start..self.pos].iter().collect();
            s.parse::<i64>()
                .map(Token::Int)
                .map_err(|_| Error::Lex { pos: start, msg: format!("invalid integer '{s}'") })
        }
    }

    fn lex_ident(&mut self) -> Result<Token> {
        let start = self.pos;
        while matches!(self.current(), Some(c) if c.is_alphanumeric() || c == '_') {
            self.advance();
        }
        let s: String = self.input[start..self.pos].iter().collect();
        Ok(Token::Ident(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(input: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(input);
        let mut tokens = vec![];
        loop {
            let tok = lexer.next_token().unwrap();
            if tok == Token::Eof { break; }
            tokens.push(tok);
        }
        tokens
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(
            tokenize("1 + 2.5"),
            vec![Token::Int(1), Token::Plus, Token::Float(2.5)]
        );
    }

    #[test]
    fn test_comparison_ops() {
        assert_eq!(tokenize("a == b"), vec![Token::Ident("a".into()), Token::Eq, Token::Ident("b".into())]);
        assert_eq!(tokenize("a != b"), vec![Token::Ident("a".into()), Token::Ne, Token::Ident("b".into())]);
        assert_eq!(tokenize("a <= b"), vec![Token::Ident("a".into()), Token::Le, Token::Ident("b".into())]);
    }

    #[test]
    fn test_index_access() {
        assert_eq!(
            tokenize(r#"feature["key"]"#),
            vec![Token::Ident("feature".into()), Token::LBracket, Token::Str("key".into()), Token::RBracket]
        );
    }

    #[test]
    fn test_func_call() {
        assert_eq!(
            tokenize("file::join_path(a, b)"),
            vec![
                Token::Ident("file".into()), Token::ColonColon, Token::Ident("join_path".into()),
                Token::LParen, Token::Ident("a".into()), Token::Comma, Token::Ident("b".into()), Token::RParen,
            ]
        );
    }

    #[test]
    fn test_logical() {
        assert_eq!(
            tokenize("a && b || !c"),
            vec![
                Token::Ident("a".into()), Token::And, Token::Ident("b".into()),
                Token::Or, Token::Bang, Token::Ident("c".into()),
            ]
        );
    }

    #[test]
    fn test_in_keyword() {
        assert_eq!(
            tokenize("x in arr"),
            vec![Token::Ident("x".into()), Token::Ident("in".into()), Token::Ident("arr".into())]
        );
    }
}
