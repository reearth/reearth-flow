use lalrpop_util::{lalrpop_mod, ParseError};

use super::ast::Expr;
use super::error::{Error, Result};
use super::lexer::Tokens;

lalrpop_mod!(pub(crate) grammar, "/core/grammar.rs");

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
            ParseError::ExtraToken {
                token: (pos, tok, _),
            } => Error::Parse {
                pos,
                msg: format!("extra token {tok:?}"),
            },
            ParseError::User { error } => error,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ast::test_util::e;
    use crate::core::ast::{BinOp, ExprKind, UnaryOp};

    fn assert_parse(input: &str, expected: crate::core::ast::Expr) {
        let got = parse(input).unwrap_or_else(|err| panic!("parse({input:?}) failed: {err}"));
        assert!(
            got == expected,
            "parse({input:?})\n  got:      {:?}\n  expected: {:?}",
            got.kind,
            expected.kind
        );
    }

    #[test]
    fn test_literal() {
        assert_parse("42", e(ExprKind::Int(42)));
        assert_parse("1.5", e(ExprKind::Float(1.5)));
        assert_parse("true", e(ExprKind::Bool(true)));
        assert_parse("null", e(ExprKind::Null));
        assert_parse(r#""hello""#, e(ExprKind::Str("hello".into())));
    }

    #[test]
    fn test_binary_ops() {
        assert_parse(
            "1 + 2",
            e(ExprKind::Binary(
                Box::new(e(ExprKind::Int(1))),
                BinOp::Add,
                Box::new(e(ExprKind::Int(2))),
            )),
        );
    }

    #[test]
    fn test_precedence() {
        // 1 + 2 * 3 should parse as 1 + (2 * 3)
        assert_parse(
            "1 + 2 * 3",
            e(ExprKind::Binary(
                Box::new(e(ExprKind::Int(1))),
                BinOp::Add,
                Box::new(e(ExprKind::Binary(
                    Box::new(e(ExprKind::Int(2))),
                    BinOp::Mul,
                    Box::new(e(ExprKind::Int(3))),
                ))),
            )),
        );
    }

    #[test]
    fn test_index_access() {
        assert_parse(
            r#"feature["package"]"#,
            e(ExprKind::Index(
                Box::new(e(ExprKind::Var("feature".into()))),
                Box::new(e(ExprKind::Str("package".into()))),
            )),
        );
    }

    #[test]
    fn test_chained_index() {
        assert_parse(
            r#"a["b"]["c"]"#,
            e(ExprKind::Index(
                Box::new(e(ExprKind::Index(
                    Box::new(e(ExprKind::Var("a".into()))),
                    Box::new(e(ExprKind::Str("b".into()))),
                ))),
                Box::new(e(ExprKind::Str("c".into()))),
            )),
        );
    }

    #[test]
    fn test_func_call() {
        assert_parse(
            r#"value("package")"#,
            e(ExprKind::FuncCall {
                name: "value".into(),
                args: vec![e(ExprKind::Str("package".into()))],
            }),
        );
    }

    #[test]
    fn test_slice() {
        assert_parse(
            r#""abc"[1:2]"#,
            e(ExprKind::Slice {
                target: Box::new(e(ExprKind::Str("abc".into()))),
                start: Some(Box::new(e(ExprKind::Int(1)))),
                stop: Some(Box::new(e(ExprKind::Int(2)))),
                step: None,
            }),
        );
        assert_parse(
            r#""abc"[::-1]"#,
            e(ExprKind::Slice {
                target: Box::new(e(ExprKind::Str("abc".into()))),
                start: None,
                stop: None,
                step: Some(Box::new(e(ExprKind::Unary(
                    UnaryOp::Neg,
                    Box::new(e(ExprKind::Int(1))),
                )))),
            }),
        );
        assert_parse(
            r#""abc"[:]"#,
            e(ExprKind::Slice {
                target: Box::new(e(ExprKind::Str("abc".into()))),
                start: None,
                stop: None,
                step: None,
            }),
        );
    }

    #[test]
    fn test_array_literal() {
        assert_parse(
            "[1, 2, 3]",
            e(ExprKind::Array(vec![
                e(ExprKind::Int(1)),
                e(ExprKind::Int(2)),
                e(ExprKind::Int(3)),
            ])),
        );
        assert_parse("[]", e(ExprKind::Array(vec![])));
    }

    #[test]
    fn test_unary() {
        assert_parse(
            "not true",
            e(ExprKind::Unary(
                UnaryOp::Not,
                Box::new(e(ExprKind::Bool(true))),
            )),
        );
        assert_parse(
            "-1",
            e(ExprKind::Unary(UnaryOp::Neg, Box::new(e(ExprKind::Int(1))))),
        );
    }

    #[test]
    fn test_grouping() {
        assert_parse(
            "(1 + 2) * 3",
            e(ExprKind::Binary(
                Box::new(e(ExprKind::Binary(
                    Box::new(e(ExprKind::Int(1))),
                    BinOp::Add,
                    Box::new(e(ExprKind::Int(2))),
                ))),
                BinOp::Mul,
                Box::new(e(ExprKind::Int(3))),
            )),
        );
    }
}

#[cfg(test)]
mod parse_smoke {
    use super::*;
    use crate::core::ast::ExprKind;

    #[test]
    fn smoke_assign_forms() {
        let cases = [
            ("x = 1 + 1; x", true),
            ("x = { 1 + 1 }; x", true),
            ("x = 1 + 1;", true),   // trailing semi returns Null
            ("x = 1", true),        // assign alone is a valid expr
            ("x = y = 2; x", true), // chained assign
        ];
        for (src, should_ok) in cases {
            let r = parse(src);
            assert_eq!(r.is_ok(), should_ok, "input={src:?}  result={r:?}");
        }
    }

    #[test]
    fn smoke_assign_ast() {
        let expr = parse("x = 42").unwrap();
        assert!(
            matches!(&expr.kind, ExprKind::Assign { lvalue, .. }
                if matches!(&lvalue.kind, ExprKind::Var(n) if n == "x")),
            "expected Assign with Var lvalue, got {:?}",
            expr.kind
        );
    }
}
