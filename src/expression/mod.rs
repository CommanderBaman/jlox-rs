use crate::{error::LanguageError, token::Token};
use strum::Display;

pub mod print;

// Grammar
// expression -> literal | unary | binary | grouping
//              | variable | assignment | logical | call
#[derive(Clone, Debug, Display)]
pub enum Expression {
    #[strum(to_string = "LiteralExpr({0})")]
    Literal(LiteralExpression),
    Grouping(GroupingExpression),
    Unary(UnaryExpression),
    #[strum(to_string = "BinaryExpr({0})")]
    Binary(BinaryExpression),
    Variable(VariableExpression),
    Assignment(AssignmentExpression),
    Logical(LogicalExpression),
    Call(CallExpression),
}

// NOTE: we require mutable expression where ever we might need to evaluate
// because assignment expression mutates the visitor
pub trait ExpressionVisitor<R> {
    fn visit_literal(&self, expression: &LiteralExpression) -> R;
    fn visit_grouping(&mut self, expression: &GroupingExpression) -> R;
    fn visit_unary(&mut self, expression: &UnaryExpression) -> R;
    fn visit_binary(&mut self, expression: &BinaryExpression) -> R;
    fn visit_variable(&mut self, expression: &VariableExpression) -> R;
    fn visit_assignment(&mut self, expression: &AssignmentExpression) -> R;
    fn visit_logical(&mut self, expression: &LogicalExpression) -> R;
    fn visit_call(&mut self, expression: &CallExpression) -> R;
}

impl Expression {
    pub fn accept<R, T: ExpressionVisitor<R>>(&self, visitor: &mut T) -> R {
        match self {
            Expression::Literal(l) => visitor.visit_literal(l),
            Expression::Grouping(l) => visitor.visit_grouping(l),
            Expression::Unary(l) => visitor.visit_unary(l),
            Expression::Binary(l) => visitor.visit_binary(l),
            Expression::Variable(l) => visitor.visit_variable(l),
            Expression::Assignment(l) => visitor.visit_assignment(l),
            Expression::Logical(l) => visitor.visit_logical(l),
            Expression::Call(l) => visitor.visit_call(l),
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

impl std::fmt::Display for LiteralExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.literal.to_string())
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

impl std::fmt::Display for BinaryExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.left_expression.to_string(),
            self.operator,
            self.right_expression.to_string()
        )
    }
}
// logical -> expression operator expression
// operator -> "and" | "or"
#[derive(Clone, Debug, Display)]
pub enum LogicalOperator {
    And,
    Or,
}
impl TryFrom<Token> for LogicalOperator {
    type Error = LanguageError;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token {
            Token::And => Ok(LogicalOperator::And),
            Token::Or => Ok(LogicalOperator::Or),
            _ => Err(LanguageError::IncorrectTokenConversion {
                base_token: token,
                converted_to: "LogicalOperator",
            }),
        }
    }
}
#[derive(Clone, Debug)]
pub struct LogicalExpression {
    pub left_expression: Box<Expression>,
    pub operator: LogicalOperator,
    pub right_expression: Box<Expression>,
}
impl LogicalExpression {
    pub fn wrap(self) -> Expression {
        Expression::Logical(self)
    }
    pub fn new(
        left_expression: Expression,
        token: Token,
        right_expression: Expression,
    ) -> Result<Expression, LanguageError> {
        let operator = token.try_into()?;
        Ok(Expression::Logical(Self {
            left_expression: Box::new(left_expression),
            operator,
            right_expression: Box::new(right_expression),
        }))
    }
}

impl std::fmt::Display for LogicalExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.left_expression.to_string(),
            self.operator,
            self.right_expression.to_string()
        )
    }
}

// variable -> IDENTIFIER
#[derive(Clone, Debug, Display, PartialEq, Eq, Hash, PartialOrd)]
pub enum VariableToken {
    #[strum(to_string = "Identifier({0})")]
    Identifer(String),
}
impl TryFrom<Token> for VariableToken {
    type Error = LanguageError;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token {
            Token::Identifier(i) => Ok(VariableToken::Identifer(i)),
            _ => Err(LanguageError::IncorrectTokenConversion {
                base_token: token,
                converted_to: "VariableToken",
            }),
        }
    }
}
#[derive(Clone, Debug)]
pub struct VariableExpression {
    pub variable: VariableToken,
}
impl VariableExpression {
    pub fn wrap(self) -> Expression {
        Expression::Variable(self)
    }
    pub fn new(token: Token) -> Result<Expression, LanguageError> {
        let variable = token.try_into()?;
        Ok(Expression::Variable(Self { variable }))
    }
}

impl std::fmt::Display for VariableExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.variable.to_string())
    }
}

// assignment -> IDENTIFIER = expression
#[derive(Clone, Debug)]
pub struct AssignmentExpression {
    pub variable: VariableToken,
    pub expression: Box<Expression>,
}
impl AssignmentExpression {
    pub fn wrap(self) -> Expression {
        Expression::Assignment(self)
    }
    pub fn from(variable: VariableToken, expression: Expression) -> Expression {
        Expression::Assignment(Self {
            variable,
            expression: Box::new(expression),
        })
    }
}

impl std::fmt::Display for AssignmentExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} = {}",
            self.variable.to_string(),
            self.expression.to_string()
        )
    }
}

// call -> primary ( "(" arguments? ")" )*
#[derive(Clone, Debug)]
pub struct CallExpression {
    pub callee: Box<Expression>,
    pub arguments: Vec<Expression>,
}
impl CallExpression {
    pub fn wrap(self) -> Expression {
        Expression::Call(self)
    }
    pub fn new(callee: Expression, arguments: Vec<Expression>) -> Expression {
        Expression::Call(Self {
            callee: Box::from(callee),
            arguments,
        })
    }
}
