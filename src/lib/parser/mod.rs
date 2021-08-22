pub(crate) mod expression;
use self::expression::Expr;
use super::token::{Keyword, Punctuator, Token, TokenKind};
use crate::error::{InnerError, LoxResult};
use Keyword::*;

/// Converts a list of tokens into an _AST_.
/// ```text
/// expression     → equality ;
/// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
/// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
/// term           → factor ( ( "-" | "+" ) factor )* ;
/// factor         → unary ( ( "/" | "*" ) unary )* ;
/// unary          → ( "!" | "-" ) unary | primary ;
/// primary        → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;
/// ```
/// 'a is the lifetime of the Vec<Token> generated by the lexer.
pub(crate) struct Parser<'a> {
    inner: InnerIter<'a, Token>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            inner: InnerIter::new(tokens),
        }
    }

    pub fn parse(self) -> LoxResult<Expr> {
        self.expression()
    }

    /// Helper function for recovering from errors.
    /// It walks the token buffer until it finds a statement boundary.
    #[allow(dead_code)]
    fn synchronize(&self) {
        self.inner.next();
        while let Some(e) = self.inner.peek() {
            if let Some(t) = self.inner.previous() {
                if t.kind() == &TokenKind::Punctuator(Punctuator::Semicolon) {
                    break;
                }
            }

            if let TokenKind::Keyword(Class | Fn | Let | For | If | While | Print | Return) =
                e.kind()
            {
                break;
            }

            self.inner.next();
        }
    }

    /// Compares a list of tokens to the next token in the buffer,
    /// if the comparison returns true, walks the list and return true.
    fn multi_check<T: Into<TokenKind> + Clone>(&self, tks: &[T]) -> bool {
        for t in tks {
            let t: TokenKind = t.to_owned().into();
            if self.inner.next_if(self.check(&t)) {
                return true;
            }
        }
        false
    }

    #[inline]
    fn check(&self, kind: &TokenKind) -> bool {
        matches!(self.inner.peek(), Some(e) if e.kind() == kind)
    }

    #[inline]
    fn expression(&self) -> LoxResult<Expr> {
        self.equality()
    }

    /// Parse (in)equality expressions
    #[inline]
    fn equality(&self) -> LoxResult<Expr> {
        self.parse_left(&[Punctuator::Eq, Punctuator::NotEq], Self::comparison)
    }

    /// Parse comparison expressions
    #[inline]
    fn comparison(&self) -> LoxResult<Expr> {
        self.parse_left(
            &[
                Punctuator::GreaterThan,
                Punctuator::GreaterThanOrEq,
                Punctuator::LessThan,
                Punctuator::LessThanOrEq,
            ],
            Self::term,
        )
    }

    /// Parse addition/subtraction expressions
    #[inline]
    fn term(&self) -> LoxResult<Expr> {
        self.parse_left(&[Punctuator::Add, Punctuator::Sub], Self::factor)
    }

    /// Parse division/multiplication expressions
    #[inline]
    fn factor(&self) -> LoxResult<Expr> {
        self.parse_left(&[Punctuator::Div, Punctuator::Mul], Self::unary)
    }

    /// Parse logic/arithmetic negation expressions
    fn unary(&self) -> LoxResult<Expr> {
        if self.multi_check(&[Punctuator::Not, Punctuator::Sub]) {
            let op = self.inner.previous().unwrap().to_owned();
            let rhs = self.unary()?;
            return Ok(Expr::Unary(op, rhs.into()));
        }
        self.primary()
    }

    /// Parse primary expressions (literals, groups)
    fn primary(&self) -> LoxResult<Expr> {
        if let Some(tk) = self.inner.peek() {
            let exp = match tk.kind() {
                TokenKind::BooleanLiteral(_) => Ok(Expr::Literal(tk.to_owned())),
                TokenKind::StringLiteral(_) => Ok(Expr::Literal(tk.to_owned())),
                TokenKind::NumericLiteral(_) => Ok(Expr::Literal(tk.to_owned())),
                TokenKind::Keyword(Keyword::Nil) => Ok(Expr::Literal(tk.to_owned())),
                TokenKind::Punctuator(Punctuator::OpenParen) => {
                    self.inner.next();
                    let expr = self.expression()?;
                    println!("{}", expr);
                    self.consume(Punctuator::CloseParen, "expected ')' after expression")?;
                    Ok(Expr::Grouping(expr.into()))
                }
                _ => Err(InnerError::new(*tk.to_owned().span(), "expected expression").into()),
            };

            if exp.is_ok() {
                self.inner.next();
            }
            return exp;
        }
        Err(InnerError::new(
            *self.inner.previous().unwrap().span(),
            "expected expression",
        )
        .into())
    }

    /// Consumes the next token if its kind is `T`. If not, return a `ParseError` with `msg`
    fn consume<T: Into<TokenKind>>(&self, kind: T, msg: &str) -> LoxResult<()> {
        let kind: TokenKind = kind.into();
        if self.check(&kind) {
            return Ok(());
        }
        Err(InnerError::new(*self.inner.previous().unwrap().span(), msg).into())
    }

    /// Parse left associative tokens
    fn parse_left<T, F>(&self, token_kinds: &[T], op_func: F) -> LoxResult<Expr>
    where
        T: Into<TokenKind> + Clone,
        F: std::ops::Fn(&Self) -> LoxResult<Expr>,
    {
        let mut expr = op_func(self)?;

        while self.multi_check(token_kinds) {
            let op = match self.inner.previous() {
                Some(op) => op.clone(),
                None => break,
            };
            let rhs = op_func(self)?;
            expr = Expr::Binary(expr.into(), op, rhs.into());
        }

        Ok(expr)
    }
}

use std::cell::RefCell;

/// Inner iterator for the parser.
struct InnerIter<'a, T> {
    collection: &'a [T],
    current: std::cell::RefCell<usize>,
}

impl<'a, T> InnerIter<'a, T> {
    #[inline]
    fn new(collection: &'a [T]) -> Self {
        Self {
            collection,
            current: RefCell::new(0),
        }
    }

    #[inline]
    fn next(&self) -> Option<&T> {
        if *self.current.borrow() < self.collection.len() {
            *self.current.borrow_mut() += 1;
        }
        self.previous()
    }

    #[inline]
    fn previous(&self) -> Option<&T> {
        self.collection
            .get((*self.current.borrow()).checked_sub(1)?)
    }

    #[inline]
    fn peek(&self) -> Option<&T> {
        self.collection.get(*self.current.borrow())
    }

    #[inline]
    fn next_if(&self, test: bool) -> bool {
        if test {
            self.next();
            return true;
        }
        false
    }
}
