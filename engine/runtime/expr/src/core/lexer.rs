use logos::Logos;

use super::error::Error;

fn lex_string<'src>(lex: &mut logos::Lexer<'src, Token>) -> Option<String> {
    let raw = lex.slice();
    let quote = raw.chars().next()?;
    let inner = &raw[1..raw.len() - 1];
    let mut result = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next()? {
                'n' => result.push('\n'),
                't' => result.push('\t'),
                '\\' => result.push('\\'),
                c if c == quote => result.push(c),
                _ => return None,
            }
        } else {
            result.push(c);
        }
    }
    Some(result)
}

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\r]+")]
pub enum Token {
    // literals
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Int(i64),

    #[regex(r#""([^"\\]|\\.)*""#, lex_string)]
    Str(String),

    // keywords — must be defined before Ident so logos gives them priority
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("null")]
    Null,
    #[token("in")]
    In,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    // punctuation
    #[token(".")]
    Dot,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,

    // arithmetic
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,

    // comparison
    #[token("==")]
    Eq,
    #[token("!=")]
    Ne,
    #[token("<=")]
    Le,
    #[token("<")]
    Lt,
    #[token(">=")]
    Ge,
    #[token(">")]
    Gt,

    // logical
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
    Bang,
}

pub struct Tokens<'src> {
    inner: logos::SpannedIter<'src, Token>,
}

impl<'src> Tokens<'src> {
    pub fn new(input: &'src str) -> Self {
        Self {
            inner: Token::lexer(input).spanned(),
        }
    }
}

impl Iterator for Tokens<'_> {
    type Item = Result<(usize, Token, usize), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(result, span)| {
            result
                .map(|tok| (span.start, tok, span.end))
                .map_err(|()| Error::Lex {
                    pos: span.start,
                    msg: "unexpected character".into(),
                })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(input: &str) -> Vec<Token> {
        Token::lexer(input)
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
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
        assert_eq!(
            tokenize("a == b"),
            vec![
                Token::Ident("a".into()),
                Token::Eq,
                Token::Ident("b".into())
            ]
        );
        assert_eq!(
            tokenize("a != b"),
            vec![
                Token::Ident("a".into()),
                Token::Ne,
                Token::Ident("b".into())
            ]
        );
        assert_eq!(
            tokenize("a <= b"),
            vec![
                Token::Ident("a".into()),
                Token::Le,
                Token::Ident("b".into())
            ]
        );
    }

    #[test]
    fn test_index_access() {
        assert_eq!(
            tokenize(r#"feature["key"]"#),
            vec![
                Token::Ident("feature".into()),
                Token::LBracket,
                Token::Str("key".into()),
                Token::RBracket
            ]
        );
    }

    #[test]
    fn test_func_call() {
        assert_eq!(
            tokenize("value(a, b)"),
            vec![
                Token::Ident("value".into()),
                Token::LParen,
                Token::Ident("a".into()),
                Token::Comma,
                Token::Ident("b".into()),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn test_logical() {
        assert_eq!(
            tokenize("a && b || !c"),
            vec![
                Token::Ident("a".into()),
                Token::And,
                Token::Ident("b".into()),
                Token::Or,
                Token::Bang,
                Token::Ident("c".into()),
            ]
        );
    }

    #[test]
    fn test_keywords() {
        assert_eq!(
            tokenize("true false null"),
            vec![Token::True, Token::False, Token::Null]
        );
        // keywords embedded in identifiers remain identifiers
        assert_eq!(tokenize("inline"), vec![Token::Ident("inline".into())]);
        assert_eq!(tokenize("trueish"), vec![Token::Ident("trueish".into())]);
    }

    #[test]
    fn test_in_keyword() {
        assert_eq!(
            tokenize("x in arr"),
            vec![Token::Ident("x".into()), Token::In, Token::Ident("arr".into())]
        );
    }

    #[test]
    fn test_string_escapes() {
        assert_eq!(tokenize(r#""hello\nworld""#), vec![Token::Str("hello\nworld".into())]);
        assert_eq!(tokenize(r#""tab\there""#), vec![Token::Str("tab\there".into())]);
        assert_eq!(tokenize(r#""quo\"te""#), vec![Token::Str("quo\"te".into())]);
    }
}
