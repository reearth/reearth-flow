use lalrpop_util::{lalrpop_mod, ParseError};

use crate::ast::Expr;
use crate::error::{Error, Result};
use crate::lexer::Tokens;

lalrpop_mod!(pub(crate) grammar);

pub fn parse(input: &str) -> Result<Expr> {
    grammar::ExprParser::new()
        .parse(Tokens::new(input))
        .map_err(|e| match e {
            ParseError::InvalidToken { location } => Error::Parse {
                pos: location,
                msg: "invalid token".into(),
            },
            ParseError::UnrecognizedEof { location, .. } => Error::Parse {
                pos: location,
                msg: "unexpected end of input".into(),
            },
            ParseError::UnrecognizedToken {
                token: (pos, tok, _),
                ..
            } => Error::Parse {
                pos,
                msg: format!("unexpected token {tok:?}"),
            },
            ParseError::ExtraToken { token: (pos, tok, _) } => Error::Parse {
                pos,
                msg: format!("extra token {tok:?}"),
            },
            ParseError::User { error } => error,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal() {
        assert_eq!(parse("42").unwrap(), Expr::Int(42));
        assert_eq!(parse("3.14").unwrap(), Expr::Float(3.14));
        assert_eq!(parse("true").unwrap(), Expr::Bool(true));
        assert_eq!(parse("null").unwrap(), Expr::Null);
        assert_eq!(parse(r#""hello""#).unwrap(), Expr::Str("hello".into()));
    }

    #[test]
    fn test_binary_ops() {
        use crate::ast::BinOp;
        assert_eq!(
            parse("1 + 2").unwrap(),
            Expr::Binary(Box::new(Expr::Int(1)), BinOp::Add, Box::new(Expr::Int(2)))
        );
    }

    #[test]
    fn test_precedence() {
        use crate::ast::BinOp;
        // 1 + 2 * 3 should parse as 1 + (2 * 3)
        assert_eq!(
            parse("1 + 2 * 3").unwrap(),
            Expr::Binary(
                Box::new(Expr::Int(1)),
                BinOp::Add,
                Box::new(Expr::Binary(
                    Box::new(Expr::Int(2)),
                    BinOp::Mul,
                    Box::new(Expr::Int(3))
                ))
            )
        );
    }

    #[test]
    fn test_index_access() {
        assert_eq!(
            parse(r#"feature["package"]"#).unwrap(),
            Expr::Index(
                Box::new(Expr::Var("feature".into())),
                Box::new(Expr::Str("package".into()))
            )
        );
    }

    #[test]
    fn test_chained_index() {
        assert_eq!(
            parse(r#"a["b"]["c"]"#).unwrap(),
            Expr::Index(
                Box::new(Expr::Index(
                    Box::new(Expr::Var("a".into())),
                    Box::new(Expr::Str("b".into()))
                )),
                Box::new(Expr::Str("c".into()))
            )
        );
    }

    #[test]
    fn test_func_call() {
        assert_eq!(
            parse(r#"getattr("package")"#).unwrap(),
            Expr::FuncCall {
                name: "getattr".into(),
                args: vec![Expr::Str("package".into())],
            }
        );
    }

    #[test]
    fn test_in_operator() {
        use crate::ast::BinOp;
        assert_eq!(
            parse("x in arr").unwrap(),
            Expr::Binary(
                Box::new(Expr::Var("x".into())),
                BinOp::In,
                Box::new(Expr::Var("arr".into()))
            )
        );
    }

    #[test]
    fn test_array_literal() {
        assert_eq!(
            parse("[1, 2, 3]").unwrap(),
            Expr::Array(vec![Expr::Int(1), Expr::Int(2), Expr::Int(3)])
        );
        assert_eq!(parse("[]").unwrap(), Expr::Array(vec![]));
    }

    #[test]
    fn test_unary() {
        use crate::ast::UnaryOp;
        assert_eq!(
            parse("!true").unwrap(),
            Expr::Unary(UnaryOp::Not, Box::new(Expr::Bool(true)))
        );
        assert_eq!(
            parse("-1").unwrap(),
            Expr::Unary(UnaryOp::Neg, Box::new(Expr::Int(1)))
        );
    }

    #[test]
    fn test_grouping() {
        use crate::ast::BinOp;
        assert_eq!(
            parse("(1 + 2) * 3").unwrap(),
            Expr::Binary(
                Box::new(Expr::Binary(
                    Box::new(Expr::Int(1)),
                    BinOp::Add,
                    Box::new(Expr::Int(2))
                )),
                BinOp::Mul,
                Box::new(Expr::Int(3))
            )
        );
    }
}
