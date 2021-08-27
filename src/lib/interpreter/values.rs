use crate::error::{InnerError, LoxError, LoxResult};
use crate::lib::parser::Stmt;
use crate::lib::token::{Keyword, Numeric, TokenKind};
use std::cmp::PartialEq;
use std::convert::TryFrom;
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::rc::Rc;

use super::Environment;

macro_rules! check {
    ($test:expr, $($val:expr),+) => {
        {
            let mut check = true;
            $(
                if !$test($val) {
                    check = false;
                }
            )+
            check
        }
    };
}

/// Test each val, if the test returns false, returns an Error.
macro_rules! check_or {
    ($test:expr, $($val:expr),+;$msg:literal) => {
        {
            $(
                if !$test($val) {
                    return Err(LoxError::Generic($msg.to_string()));
                }
            )*
        }
    };
}

/// Performs a binary operation between two `[LoxValue]`s, using the given operator.
macro_rules! binop {
    ($lhs:expr, $rhs:expr, $op:tt) => {
        {
            // if at least one of them is decimal, then we return a decimal
            if $lhs.is_decimal() || $rhs.is_decimal() {
                return Ok(LoxValue::Decimal(($lhs.to_dec() $op $rhs.to_dec())));
            }

            // here we definitely have two integers
            Ok(LoxValue::Integer(($lhs.to_int() $op $rhs.to_int())))
        }
    }
}
macro_rules! cmpop {
    ($lhs:expr, $rhs:expr, $op:tt) => {
        {
            Ok(LoxValue::Boolean(($lhs.to_dec() $op $rhs.to_dec())))
        }
    }
}

/// Internal language types
#[derive(Clone)]
pub(crate) enum LoxValue {
    String(String),
    Nil,
    Decimal(f64),
    Integer(isize),
    Boolean(bool),
    Callable(Rc<dyn LoxCallable>),
}

pub(crate) trait LoxCallable {
    fn call(&self, interpreter: &Environment<'_>, args: &[LoxValue]) -> LoxResult<LoxValue>;
    fn arity(&self) -> usize;
    fn to_string(&self) -> String;
}

pub(crate) struct LoxFunction {
    declaration: Stmt,
    arity: usize,
}

impl LoxFunction {
    pub fn new(declaration: Stmt) -> LoxResult<Self> {
        if let Stmt::Function(_, ref params, _) = declaration {
            let arity = params.len();
            Ok(Self { declaration, arity })
        } else {
            Err(LoxError::Generic(format!(
                "cannot create lox function from {} statement",
                declaration
            )))
        }
    }
}

impl LoxCallable for LoxFunction {
    fn call(&self, env: &Environment<'_>, args: &[LoxValue]) -> LoxResult<LoxValue> {
        let env = Environment::from(env);
        if let Stmt::Function(_name, params, body) = &self.declaration {
            for (idx, identifier) in params.iter().enumerate() {
                env.define(
                    &identifier.to_string(),
                    args.get(idx)
                        .ok_or_else(|| {
                            InnerError::new(
                                *identifier.span(),
                                "missing arguments to function call",
                            )
                        })?
                        .to_owned(),
                )
            }

            if let Err(err) = body.execute(&env, &mut std::io::stdout()) {
                match err {
                    LoxError::Return(r) => {
                        return Ok(r.val);
                    }
                    _ => {
                        return Err(err);
                    }
                }
            }
        }

        Ok(LoxValue::Nil)
    }

    fn arity(&self) -> usize {
        self.arity
    }

    fn to_string(&self) -> String {
        if let Stmt::Function(name, _, _) = &self.declaration {
            return format!("<fn {}>", name.to_string());
        }
        unreachable!()
    }
}

impl LoxValue {
    /// Check if a value is `truthy` or not.
    /// `false` and `nil` are falsy, everything else is truthy.
    pub fn is_truthy(&self) -> bool {
        match &self {
            Self::Boolean(b) => *b,
            Self::Nil => false,
            _ => true,
        }
    }

    pub fn ge(&self, oth: &Self) -> LoxResult<LoxValue> {
        check_or!(LoxValue::is_num, self, oth; "operands must be numbers");
        cmpop!(self, oth, >=)
    }

    pub fn gt(&self, oth: &Self) -> LoxResult<LoxValue> {
        check_or!(LoxValue::is_num, self, oth; "operands must be numbers");
        cmpop!(self, oth, >)
    }
    pub fn le(&self, oth: &Self) -> LoxResult<LoxValue> {
        check_or!(LoxValue::is_num, self, oth; "operands must be numbers");
        cmpop!(self, oth, <=)
    }
    pub fn lt(&self, oth: &Self) -> LoxResult<LoxValue> {
        check_or!(LoxValue::is_num, self, oth; "operands must be numbers");
        cmpop!(self, oth, <)
    }

    fn to_int(&self) -> isize {
        match self {
            Self::Decimal(d) => *d as isize,
            Self::Integer(i) => *i,
            _ => unreachable!(),
        }
    }

    fn to_dec(&self) -> f64 {
        match self {
            Self::Decimal(d) => *d,
            Self::Integer(i) => *i as f64,
            _ => unreachable!(),
        }
    }

    fn is_num(&self) -> bool {
        matches!(self, Self::Integer(_) | Self::Decimal(_))
    }

    fn is_decimal(&self) -> bool {
        matches!(self, Self::Decimal(_))
    }

    fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }
}

impl TryFrom<&TokenKind> for LoxValue {
    type Error = &'static str;
    fn try_from(t: &TokenKind) -> Result<Self, Self::Error> {
        match t {
            TokenKind::StringLiteral(s) => Ok(LoxValue::String(s.to_string())),
            TokenKind::BooleanLiteral(b) => Ok(LoxValue::Boolean(*b)),
            TokenKind::NumericLiteral(Numeric::Integer(i)) => Ok(LoxValue::Integer(*i)),
            TokenKind::NumericLiteral(Numeric::Decimal(d)) => Ok(LoxValue::Decimal(*d)),
            TokenKind::Keyword(Keyword::Nil) => Ok(LoxValue::Nil),
            _ => Err("could not convert token to LoxValue"),
        }
    }
}

impl std::fmt::Display for LoxValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &*self {
            LoxValue::Decimal(d) => write!(f, "{}", d),
            LoxValue::Integer(i) => write!(f, "{}", i),
            LoxValue::Boolean(b) => write!(f, "{}", b),
            LoxValue::String(s) => write!(f, "{}", s),
            LoxValue::Callable(callable) => write!(f, "{}", callable.to_string()),
            LoxValue::Nil => write!(f, "nil"),
        }
    }
}
impl std::fmt::Debug for LoxValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &*self {
            LoxValue::Decimal(d) => write!(f, "{}", d),
            LoxValue::Integer(i) => write!(f, "{}", i),
            LoxValue::Boolean(b) => write!(f, "{}", b),
            LoxValue::String(s) => write!(f, "{}", s),
            LoxValue::Callable(callable) => write!(f, "{}", callable.to_string()),
            LoxValue::Nil => write!(f, "nil"),
        }
    }
}

impl Sub for LoxValue {
    type Output = LoxResult<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        check_or!(LoxValue::is_num, &self, &rhs; "operands must be numbers");

        binop!(self, rhs, -)
    }
}

impl Div for LoxValue {
    type Output = LoxResult<Self>;

    fn div(self, rhs: Self) -> Self::Output {
        check_or!(LoxValue::is_num, &self, &rhs; "operands must be numbers");
        let zero = LoxValue::Decimal(0.0);
        if self.eq(&zero) || rhs.eq(&zero) {
            return Err(LoxError::Generic("attempt to divide by zero".to_string()));
        }
        binop!(self, rhs, /)
    }
}

impl Mul for LoxValue {
    type Output = LoxResult<Self>;

    fn mul(self, rhs: Self) -> Self::Output {
        check_or!(LoxValue::is_num, &self, &rhs; "operands must be numbers");

        binop!(self, rhs, *)
    }
}

impl Add for LoxValue {
    type Output = LoxResult<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        if check!(Self::is_num, &self, &rhs) {
            return binop!(self, rhs, +);
        }

        if self.is_string() || rhs.is_string() {
            return Ok(LoxValue::String(format!("{}{}", self, rhs)));
        }

        Err(LoxError::Generic(
            "operands must be number or string".to_string(),
        ))
    }
}

impl Neg for LoxValue {
    type Output = LoxResult<Self>;

    fn neg(self) -> Self::Output {
        match self {
            Self::Decimal(d) => Ok(LoxValue::Decimal(-d)),
            Self::Integer(i) => Ok(LoxValue::Integer(-i)),
            _ => Err(LoxError::Generic(
                "unary operand must be a number".to_string(),
            )),
        }
    }
}

// Returns true if all the values match the pattern
macro_rules! multi_matches {
    ($expression:pat, $($val:expr),+) => {
        {
            let mut ret = true;
            $(ret &= matches!($val, $expression);)+
            ret
        }
    }
}

// Nil, Nil => true
// String, String => cmp.string,
// Int, int = cmp.int,
// bool, bool, => cmp.bool,
// _ => false
impl PartialEq for LoxValue {
    fn eq(&self, oth: &Self) -> bool {
        if multi_matches!(&LoxValue::Nil, &self, &oth) {
            return true;
        }

        match &self {
            LoxValue::Nil => false,
            LoxValue::String(s) => {
                if let LoxValue::String(oth) = oth {
                    return s.eq(oth);
                }
                false
            }
            LoxValue::Boolean(b) => {
                if let LoxValue::Boolean(oth) = oth {
                    return b.eq(oth);
                }
                false
            }
            _ => {
                if !oth.is_num() {
                    return false;
                }

                let lhs = self.to_dec();
                let rhs = oth.to_dec();

                lhs.eq(&rhs)
            }
        }
    }
}
