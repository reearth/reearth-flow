use logos::Logos;

use super::error::Error;

fn lex_raw_string<'src>(lex: &mut logos::Lexer<'src, Token>) -> Option<String> {
    let raw = lex.slice();
    // strip leading r" and trailing "
    Some(raw[2..raw.len() - 1].to_string())
}

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
#[logos(skip r"#[^\n]*")]
pub enum Token {
    // literals
    #[regex(r"[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?|[0-9]+[eE][+-]?[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    #[regex(r"0[bB][01]+", |lex| i64::from_str_radix(&lex.slice()[2..], 2).ok())]
    #[regex(r"0[oO][0-7]+", |lex| i64::from_str_radix(&lex.slice()[2..], 8).ok())]
    #[regex(r"0[xX][0-9a-fA-F]+", |lex| i64::from_str_radix(&lex.slice()[2..], 16).ok())]
    Int(i64),

    #[regex(r#""([^"\\]|\\.)*""#, lex_string)]
    #[regex(r#"r"[^"]*""#, lex_raw_string)]
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
    #[token("not")]
    Not,
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("return")]
    Return,

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
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(";")]
    Semi,

    // arithmetic
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("**")]
    DoubleStar,
    #[token("/")]
    Slash,
    #[token("//")]
    DoubleSlash,
    #[token("%")]
    Percent,

    // compound assignment (must come before plain `=`, `+`, `-`, `*`, `/` — longer wins)
    #[token("+=")]
    PlusAssign,
    #[token("-=")]
    MinusAssign,
    #[token("*=")]
    StarAssign,
    #[token("**=")]
    DoubleStarAssign,
    #[token("/=")]
    SlashAssign,
    #[token("//=")]
    DoubleSlashAssign,
    #[token("%=")]
    PercentAssign,

    // assignment (single `=`; must come after `==` in logos priority — longer wins)
    #[token("=")]
    Assign,

    // bitwise
    #[token("&")]
    Amp,
    #[token("|")]
    Pipe,
    #[token("^")]
    Caret,
    #[token("<<=")]
    LShiftAssign,
    #[token(">>=")]
    RShiftAssign,
    #[token("&=")]
    AmpAssign,
    #[token("|=")]
    PipeAssign,
    #[token("^=")]
    CaretAssign,
    #[token("<<")]
    LShift,
    #[token(">>")]
    RShift,

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
        Token::lexer(input).collect::<Result<Vec<_>, _>>().unwrap()
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(
            tokenize("1 + 2.5"),
            vec![Token::Int(1), Token::Plus, Token::Float(2.5)]
        );
    }

    #[test]
    fn test_float_scientific_notation() {
        assert_eq!(tokenize("1e-10"), vec![Token::Float(1e-10)]);
        assert_eq!(tokenize("1e10"), vec![Token::Float(1e10)]);
        assert_eq!(tokenize("1.5e3"), vec![Token::Float(1.5e3)]);
        assert_eq!(tokenize("2.0E+4"), vec![Token::Float(2.0E+4)]);
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
            tokenize("a and b or not c"),
            vec![
                Token::Ident("a".into()),
                Token::And,
                Token::Ident("b".into()),
                Token::Or,
                Token::Not,
                Token::Ident("c".into()),
            ]
        );
    }

    #[test]
    fn test_in_keyword() {
        assert_eq!(
            tokenize("x in arr"),
            vec![
                Token::Ident("x".into()),
                Token::In,
                Token::Ident("arr".into())
            ]
        );
        assert_eq!(
            tokenize("x not in arr"),
            vec![
                Token::Ident("x".into()),
                Token::Not,
                Token::In,
                Token::Ident("arr".into()),
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
        assert_eq!(tokenize("inside"), vec![Token::Ident("inside".into())]);
        assert_eq!(tokenize("notify"), vec![Token::Ident("notify".into())]);
        assert_eq!(tokenize("android"), vec![Token::Ident("android".into())]);
        assert_eq!(tokenize("order"), vec![Token::Ident("order".into())]);
    }

    #[test]
    fn test_if_else_keywords() {
        assert_eq!(
            tokenize("if x { 1 } else { 2 }"),
            vec![
                Token::If,
                Token::Ident("x".into()),
                Token::LBrace,
                Token::Int(1),
                Token::RBrace,
                Token::Else,
                Token::LBrace,
                Token::Int(2),
                Token::RBrace,
            ]
        );
        // `iffy` and `elsewhere` must remain identifiers
        assert_eq!(tokenize("iffy"), vec![Token::Ident("iffy".into())]);
        assert_eq!(
            tokenize("elsewhere"),
            vec![Token::Ident("elsewhere".into())]
        );
    }

    #[test]
    fn test_integer_literals() {
        assert_eq!(tokenize("0b1010"), vec![Token::Int(0b1010)]);
        assert_eq!(tokenize("0B1010"), vec![Token::Int(0b1010)]);
        assert_eq!(tokenize("0o777"), vec![Token::Int(0o777)]);
        assert_eq!(tokenize("0xff"), vec![Token::Int(0xff)]);
        assert_eq!(tokenize("0XdeadBEEF"), vec![Token::Int(0xDEADBEEF)]);
    }

    #[test]
    fn test_line_comment() {
        assert_eq!(
            tokenize("1 # comment\n+ 2"),
            vec![Token::Int(1), Token::Plus, Token::Int(2)]
        );
        assert_eq!(tokenize("# full line comment\n42"), vec![Token::Int(42)]);
        assert_eq!(tokenize("1 # trailing"), vec![Token::Int(1)]);
        // # inside a string must not start a comment
        assert_eq!(
            tokenize(r#""hello#world""#),
            vec![Token::Str("hello#world".into())]
        );
        assert_eq!(
            tokenize(r#""foo#bar" # strip this"#),
            vec![Token::Str("foo#bar".into())]
        );
    }

    #[test]
    fn test_raw_string() {
        // backslashes are literal, not escape sequences
        assert_eq!(tokenize(r#"r"\n\t""#), vec![Token::Str(r"\n\t".into())]);
        // regular content works too
        assert_eq!(
            tokenize(r#"r"hello world""#),
            vec![Token::Str("hello world".into())]
        );
    }

    #[test]
    fn test_string_escapes() {
        assert_eq!(
            tokenize(r#""hello\nworld""#),
            vec![Token::Str("hello\nworld".into())]
        );
        assert_eq!(
            tokenize(r#""tab\there""#),
            vec![Token::Str("tab\there".into())]
        );
        assert_eq!(tokenize(r#""quo\"te""#), vec![Token::Str("quo\"te".into())]);
    }
}
