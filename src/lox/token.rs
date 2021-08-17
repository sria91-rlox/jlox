#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    kind: TokenKind,
    lexeme: String,
    literal: String,
    span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, literal: String, span: Span) -> Self {
        Self {
            kind,
            lexeme,
            literal,
            span,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.kind, self.lexeme, self.literal)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    line: u32,
    col: u32,
}

impl Position {
    pub fn new(line: u32, col: u32) -> Self {
        Self { line, col }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    start: Position,
    end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Keyword(Keyword),
    Punctuator(Punctuator),
    Identifier(Box<str>),
    StringLiteral(Box<str>),
    NumericLiteral(Numeric),
    BooleanLiteral(bool),
    Comment,
    EOF,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            TokenKind::Keyword(ref k) => write!(f, "{}", k),
            TokenKind::Identifier(ref ident) => write!(f, "{}", ident),
            TokenKind::Punctuator(ref punc) => write!(f, "{}", punc),
            TokenKind::StringLiteral(ref s) => write!(f, "{}", s),
            TokenKind::NumericLiteral(Numeric::Integer(n)) => write!(f, "{}", n),
            TokenKind::NumericLiteral(Numeric::Decimal(n)) => write!(f, "{}", n),
            TokenKind::BooleanLiteral(ref b) => write!(f, "{}", b),
            TokenKind::Comment => write!(f, "comment"),
            TokenKind::EOF => write!(f, "end of file"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Punctuator {
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    Assign,
    AssignAdd,
    AssignSub,
    AssignMul,
    AssignDiv,
    Not,
    Eq,
    NotEq,
    Add,
    Sub,
    Mul,
    Div,
    Dot,
    Comma,
    Semicolon,
    GreaterThan,
    GreaterThanOrEq,
    LessThan,
    LessThanOrEq,
}

impl std::fmt::Display for Punctuator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Punctuator::OpenParen => "(",
                Punctuator::CloseParen => ")",
                Punctuator::OpenBracket => "[",
                Punctuator::CloseBracket => "]",
                Punctuator::Comma => ",",
                Punctuator::Dot => ".",
                Punctuator::Semicolon => ";",
                Punctuator::Assign => "=",
                Punctuator::AssignAdd => "+=",
                Punctuator::AssignSub => "-=",
                Punctuator::AssignDiv => "/=",
                Punctuator::AssignMul => "*=",
                Punctuator::Eq => "==",
                Punctuator::Sub => "-",
                Punctuator::Add => "+",
                Punctuator::Div => "/",
                Punctuator::Mul => "*",
                Punctuator::GreaterThan => ">",
                Punctuator::GreaterThanOrEq => ">=",
                Punctuator::LessThan => "<",
                Punctuator::LessThanOrEq => "<=",
                Punctuator::Not => "!",
                Punctuator::NotEq => "!=",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keyword {
    And,
    Class,
    Else,
    Let,
    While,
    Fn,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    Extends,
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Keyword::And => "and",
                Keyword::Class => "class",
                Keyword::Else => "else",
                Keyword::Let => "let",
                Keyword::While => "while",
                Keyword::Fn => "fn",
                Keyword::For => "for",
                Keyword::If => "if",
                Keyword::Nil => "nil",
                Keyword::Or => "or",
                Keyword::Print => "print",
                Keyword::Return => "return",
                Keyword::Super => "super",
                Keyword::This => "this",
                Keyword::Extends => "extends",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Numeric {
    Integer(usize),
    Decimal(f64),
}
