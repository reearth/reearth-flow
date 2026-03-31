use crate::ast::{BinOp, Expr, UnaryOp};
use crate::error::{Error, Result};
use crate::lexer::{Lexer, Token};

pub struct Parser {
    lexer: Lexer,
    peeked: Option<Token>,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Self { lexer: Lexer::new(input), peeked: None }
    }

    fn peek(&mut self) -> Result<&Token> {
        if self.peeked.is_none() {
            self.peeked = Some(self.lexer.next_token()?);
        }
        Ok(self.peeked.as_ref().unwrap())
    }

    fn consume(&mut self) -> Result<Token> {
        match self.peeked.take() {
            Some(tok) => Ok(tok),
            None => self.lexer.next_token(),
        }
    }

    fn expect(&mut self, expected: &Token) -> Result<()> {
        let tok = self.consume()?;
        if &tok == expected {
            Ok(())
        } else {
            Err(Error::Parse {
                pos: self.lexer.pos(),
                msg: format!("expected {expected:?}, got {tok:?}"),
            })
        }
    }

    pub fn parse(&mut self) -> Result<Expr> {
        let expr = self.parse_or()?;
        let tok = self.consume()?;
        if tok != Token::Eof {
            return Err(Error::Parse {
                pos: self.lexer.pos(),
                msg: format!("unexpected token {tok:?} after expression"),
            });
        }
        Ok(expr)
    }

    // precedence levels (lowest → highest):
    // or → and → equality → comparison → add → mul → unary → postfix → primary

    fn parse_or(&mut self) -> Result<Expr> {
        let mut left = self.parse_and()?;
        while self.peek()? == &Token::Or {
            self.consume()?;
            let right = self.parse_and()?;
            left = Expr::Binary(Box::new(left), BinOp::Or, Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut left = self.parse_equality()?;
        while self.peek()? == &Token::And {
            self.consume()?;
            let right = self.parse_equality()?;
            left = Expr::Binary(Box::new(left), BinOp::And, Box::new(right));
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut left = self.parse_comparison()?;
        loop {
            let op = match self.peek()? {
                Token::Eq => BinOp::Eq,
                Token::Ne => BinOp::Ne,
                Token::Ident(s) if s == "in" => BinOp::In,
                _ => break,
            };
            self.consume()?;
            let right = self.parse_comparison()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut left = self.parse_add()?;
        loop {
            let op = match self.peek()? {
                Token::Lt => BinOp::Lt,
                Token::Le => BinOp::Le,
                Token::Gt => BinOp::Gt,
                Token::Ge => BinOp::Ge,
                _ => break,
            };
            self.consume()?;
            let right = self.parse_add()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_add(&mut self) -> Result<Expr> {
        let mut left = self.parse_mul()?;
        loop {
            let op = match self.peek()? {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.consume()?;
            let right = self.parse_mul()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_mul(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek()? {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                _ => break,
            };
            self.consume()?;
            let right = self.parse_unary()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        match self.peek()? {
            Token::Bang => {
                self.consume()?;
                let expr = self.parse_unary()?;
                Ok(Expr::Unary(UnaryOp::Not, Box::new(expr)))
            }
            Token::Minus => {
                self.consume()?;
                let expr = self.parse_unary()?;
                Ok(Expr::Unary(UnaryOp::Neg, Box::new(expr)))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;
        // chained index: expr[key][key2]...
        while self.peek()? == &Token::LBracket {
            self.consume()?;
            let key = self.parse_or()?;
            self.expect(&Token::RBracket)?;
            expr = Expr::Index(Box::new(expr), Box::new(key));
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        let tok = self.consume()?;
        match tok {
            Token::Int(n) => Ok(Expr::Int(n)),
            Token::Float(f) => Ok(Expr::Float(f)),
            Token::Str(s) => Ok(Expr::Str(s)),
            Token::Ident(s) => match s.as_str() {
                "true" => Ok(Expr::Bool(true)),
                "false" => Ok(Expr::Bool(false)),
                "null" => Ok(Expr::Null),
                _ => {
                    // check for ns::func(args)
                    if self.peek()? == &Token::ColonColon {
                        self.consume()?; // ::
                        let name = match self.consume()? {
                            Token::Ident(n) => n,
                            tok => return Err(Error::Parse {
                                pos: self.lexer.pos(),
                                msg: format!("expected function name after '::', got {tok:?}"),
                            }),
                        };
                        self.expect(&Token::LParen)?;
                        let args = self.parse_args()?;
                        self.expect(&Token::RParen)?;
                        Ok(Expr::FuncCall { ns: s, name, args })
                    } else {
                        Ok(Expr::Var(s))
                    }
                }
            },
            Token::LParen => {
                let expr = self.parse_or()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            Token::LBracket => {
                // array literal: [expr, expr, ...]
                let mut items = vec![];
                if self.peek()? != &Token::RBracket {
                    items.push(self.parse_or()?);
                    while self.peek()? == &Token::Comma {
                        self.consume()?;
                        if self.peek()? == &Token::RBracket { break; } // trailing comma
                        items.push(self.parse_or()?);
                    }
                }
                self.expect(&Token::RBracket)?;
                Ok(Expr::Array(items))
            }
            tok => Err(Error::Parse {
                pos: self.lexer.pos(),
                msg: format!("unexpected token {tok:?}"),
            }),
        }
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>> {
        let mut args = vec![];
        if self.peek()? == &Token::RParen {
            return Ok(args);
        }
        args.push(self.parse_or()?);
        while self.peek()? == &Token::Comma {
            self.consume()?;
            if self.peek()? == &Token::RParen { break; } // trailing comma
            args.push(self.parse_or()?);
        }
        Ok(args)
    }
}

pub fn parse(input: &str) -> Result<Expr> {
    Parser::new(input).parse()
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
        assert_eq!(
            parse("1 + 2").unwrap(),
            Expr::Binary(Box::new(Expr::Int(1)), BinOp::Add, Box::new(Expr::Int(2)))
        );
    }

    #[test]
    fn test_precedence() {
        // 1 + 2 * 3 should parse as 1 + (2 * 3)
        assert_eq!(
            parse("1 + 2 * 3").unwrap(),
            Expr::Binary(
                Box::new(Expr::Int(1)),
                BinOp::Add,
                Box::new(Expr::Binary(Box::new(Expr::Int(2)), BinOp::Mul, Box::new(Expr::Int(3))))
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
            parse(r#"file::join_path("a", "b")"#).unwrap(),
            Expr::FuncCall {
                ns: "file".into(),
                name: "join_path".into(),
                args: vec![Expr::Str("a".into()), Expr::Str("b".into())],
            }
        );
    }

    #[test]
    fn test_in_operator() {
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
        assert_eq!(parse("[1, 2, 3]").unwrap(), Expr::Array(vec![Expr::Int(1), Expr::Int(2), Expr::Int(3)]));
        assert_eq!(parse("[]").unwrap(), Expr::Array(vec![]));
    }

    #[test]
    fn test_unary() {
        assert_eq!(parse("!true").unwrap(), Expr::Unary(UnaryOp::Not, Box::new(Expr::Bool(true))));
        assert_eq!(parse("-1").unwrap(), Expr::Unary(UnaryOp::Neg, Box::new(Expr::Int(1))));
    }

    #[test]
    fn test_grouping() {
        assert_eq!(
            parse("(1 + 2) * 3").unwrap(),
            Expr::Binary(
                Box::new(Expr::Binary(Box::new(Expr::Int(1)), BinOp::Add, Box::new(Expr::Int(2)))),
                BinOp::Mul,
                Box::new(Expr::Int(3))
            )
        );
    }
}
