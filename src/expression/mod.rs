use crate::{error::LanguageError, token::Token};
use strum::Display;

pub mod print;

// Grammar
// expression     → literal
//                | unary
//                | binary
//                | grouping ;
#[derive(Clone, Debug, Display)]
pub enum Expression {
    Literal(LiteralExpression),
    Grouping(GroupingExpression),
    Unary(UnaryExpression),
    Binary(BinaryExpression),
}

pub trait ExpressionVisitor<R> {
    fn visit_literal(&self, expression: &LiteralExpression) -> R;
    fn visit_grouping(&self, expression: &GroupingExpression) -> R;
    fn visit_unary(&self, expression: &UnaryExpression) -> R;
    fn visit_binary(&self, expression: &BinaryExpression) -> R;
}

impl Expression {
    pub fn accept<R, T: ExpressionVisitor<R>>(&self, visitor: &T) -> R {
        match self {
            Expression::Literal(l) => visitor.visit_literal(l),
            Expression::Grouping(l) => visitor.visit_grouping(l),
            Expression::Unary(l) => visitor.visit_unary(l),
            Expression::Binary(l) => visitor.visit_binary(l),
        }
    }
}

// literal        → NUMBER | STRING | "true" | "false" | "nil"
#[derive(Clone, Debug, Display)]
pub enum LiteralToken {
    #[strum(to_string = "Number({0})")]
    Number(f64),
    #[strum(to_string = "String('{0}')")]
    String(String),
    True,
    False,
    Nil,
}
impl TryFrom<Token> for LiteralToken {
    type Error = LanguageError;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token {
            Token::Number(n) => Ok(LiteralToken::Number(n)),
            Token::String(s) => Ok(LiteralToken::String(s)),
            Token::True => Ok(LiteralToken::True),
            Token::False => Ok(LiteralToken::False),
            Token::Nil => Ok(LiteralToken::Nil),
            _ => Err(LanguageError::IncorrectTokenConversion {
                base_token: token,
                converted_to: "LiteralToken",
            }),
        }
    }
}
#[derive(Clone, Debug)]
pub struct LiteralExpression {
    pub literal: LiteralToken,
}
impl LiteralExpression {
    pub fn wrap(self) -> Expression {
        Expression::Literal(self)
    }
    pub fn new(token: Token) -> Result<Expression, LanguageError> {
        let literal = token.try_into()?;
        Ok(Expression::Literal(Self { literal }))
    }
}

// grouping       → "(" expression ")"
#[derive(Clone, Debug)]
pub struct GroupingExpression {
    pub expression: Box<Expression>,
}
impl GroupingExpression {
    pub fn wrap(self) -> Expression {
        Expression::Grouping(self)
    }
    pub fn new(expression: Expression) -> Expression {
        Expression::Grouping(Self {
            expression: Box::new(expression),
        })
    }
}

// unary          → ( "-" | "!" ) expression
#[derive(Clone, Debug, Display)]
pub enum UnaryOperator {
    Bang,
    Minus,
}
impl TryFrom<Token> for UnaryOperator {
    type Error = LanguageError;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token {
            Token::Minus => Ok(UnaryOperator::Minus),
            Token::Bang => Ok(UnaryOperator::Bang),
            _ => Err(LanguageError::IncorrectTokenConversion {
                base_token: token,
                converted_to: "UnaryOperator",
            }),
        }
    }
}
#[derive(Clone, Debug)]
pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub expression: Box<Expression>,
}
impl UnaryExpression {
    pub fn wrap(self) -> Expression {
        Expression::Unary(self)
    }
    pub fn new(
        token: Token,
        expression: Expression,
    ) -> Result<Expression, LanguageError> {
        let operator = token.try_into()?;
        Ok(Expression::Unary(Self {
            operator,
            expression: Box::new(expression),
        }))
    }
}

// binary         → expression operator expression
// operator       → "==" | "!=" | "<" | "<=" | ">" | ">=" | "+" | "-" | "*" | "/"
#[derive(Clone, Debug, Display)]
pub enum BinaryOperator {
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Plus,
    Minus,
    Star,
    Slash,
}
impl TryFrom<Token> for BinaryOperator {
    type Error = LanguageError;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token {
            Token::EqualEqual => Ok(BinaryOperator::EqualEqual),
            Token::BangEqual => Ok(BinaryOperator::BangEqual),
            Token::Less => Ok(BinaryOperator::Less),
            Token::LessEqual => Ok(BinaryOperator::LessEqual),
            Token::Greater => Ok(BinaryOperator::Greater),
            Token::GreaterEqual => Ok(BinaryOperator::GreaterEqual),
            Token::Plus => Ok(BinaryOperator::Plus),
            Token::Minus => Ok(BinaryOperator::Minus),
            Token::Star => Ok(BinaryOperator::Star),
            Token::Slash => Ok(BinaryOperator::Slash),
            _ => Err(LanguageError::IncorrectTokenConversion {
                base_token: token,
                converted_to: "BinaryOperator",
            }),
        }
    }
}
#[derive(Clone, Debug)]
pub struct BinaryExpression {
    pub left_expression: Box<Expression>,
    pub operator: BinaryOperator,
    pub right_expression: Box<Expression>,
}
impl BinaryExpression {
    pub fn wrap(self) -> Expression {
        Expression::Binary(self)
    }
    pub fn new(
        left_expression: Expression,
        token: Token,
        right_expression: Expression,
    ) -> Result<Expression, LanguageError> {
        let operator = token.try_into()?;
        Ok(Expression::Binary(Self {
            left_expression: Box::new(left_expression),
            operator,
            right_expression: Box::new(right_expression),
        }))
    }
}
