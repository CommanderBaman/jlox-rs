use std::{fmt::Display, rc::Rc};

use crate::{error::ParseError, token::Token};
use strum::Display;

pub mod print;

// Grammar
// expression -> literal | unary | binary | grouping
//              | variable | assignment | logical | call | this
#[derive(Clone, Debug)]
pub enum Expression {
    Literal(LiteralExpression),
    Grouping(GroupingExpression),
    Unary(UnaryExpression),
    Binary(BinaryExpression),
    Variable(VariableExpression),
    Assignment(AssignmentExpression),
    Logical(LogicalExpression),
    Call(CallExpression),
    Get(GetExpression),
    Set(SetExpression),
    This(ThisExpression),
    Super(SuperExpression),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: please use a custom macro to generate this
        // Reference: https://lukaswirth.dev/tlborm/introduction.html
        // Little book of rust macros
        let string = match self {
            Self::Literal(e) => format!("{e}"),
            Self::Grouping(e) => format!("{e}"),
            Self::Unary(e) => format!("{e}"),
            Self::Binary(e) => format!("{e}"),
            Self::Variable(e) => format!("{e}"),
            Self::Assignment(e) => format!("{e}"),
            Self::Logical(e) => format!("{e}"),
            Self::Call(e) => format!("{e}"),
            Self::Get(e) => format!("{e}"),
            Self::Set(e) => format!("{e}"),
            Self::This(e) => format!("{e}"),
            Self::Super(e) => format!("{e}"),
        };

        write!(f, "{}", string)
    }
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
    fn visit_get(&mut self, expression: &GetExpression) -> R;
    fn visit_set(&mut self, expression: &SetExpression) -> R;
    fn visit_this(&mut self, expression: &ThisExpression) -> R;
    fn visit_super(&mut self, expression: &SuperExpression) -> R;
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
            Expression::Get(l) => visitor.visit_get(l),
            Expression::Set(l) => visitor.visit_set(l),
            Expression::This(l) => visitor.visit_this(l),
            Expression::Super(l) => visitor.visit_super(l),
        }
    }
}

// literal        → NUMBER | STRING | "true" | "false" | "nil" | this
#[derive(Clone, Debug, Display)]
pub enum LiteralToken {
    #[strum(to_string = "{0}")]
    Number(f64),
    #[strum(to_string = "'{0}'")]
    String(String),
    #[strum(to_string = "true")]
    True,
    #[strum(to_string = "false")]
    False,
    #[strum(to_string = "nil")]
    Nil,
}
impl TryFrom<Token> for LiteralToken {
    type Error = ParseError;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token {
            Token::Number(n) => Ok(LiteralToken::Number(n)),
            Token::String(s) => Ok(LiteralToken::String(s)),
            Token::True => Ok(LiteralToken::True),
            Token::False => Ok(LiteralToken::False),
            Token::Nil => Ok(LiteralToken::Nil),
            _ => Err(ParseError::IncorrectTokenConversion {
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
    pub fn build(token: Token) -> Result<Expression, ParseError> {
        let literal = token.try_into()?;
        Ok(Expression::Literal(Self { literal }))
    }
}

impl Display for LiteralExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.literal)
    }
}

// grouping       → "(" expression ")"
#[derive(Clone, Debug)]
pub struct GroupingExpression {
    pub expression: Rc<Expression>,
}
impl GroupingExpression {
    pub fn wrap(self) -> Expression {
        Expression::Grouping(self)
    }
    pub fn build(expression: Expression) -> Expression {
        Expression::Grouping(Self {
            expression: Rc::new(expression),
        })
    }
}

impl Display for GroupingExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.expression)
    }
}

// unary          → ( "-" | "!" ) expression
#[derive(Clone, Debug, Display, Copy)]
pub enum UnaryOperator {
    #[strum(to_string = "!")]
    Bang,
    #[strum(to_string = "-")]
    Minus,
}
impl TryFrom<Token> for UnaryOperator {
    type Error = ParseError;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token {
            Token::Minus => Ok(UnaryOperator::Minus),
            Token::Bang => Ok(UnaryOperator::Bang),
            _ => Err(ParseError::IncorrectTokenConversion {
                base_token: token,
                converted_to: "UnaryOperator",
            }),
        }
    }
}
#[derive(Clone, Debug)]
pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub expression: Rc<Expression>,
}
impl UnaryExpression {
    pub fn wrap(self) -> Expression {
        Expression::Unary(self)
    }
    pub fn build(
        token: Token,
        expression: Expression,
    ) -> Result<Expression, ParseError> {
        let operator = token.try_into()?;
        Ok(Expression::Unary(Self {
            operator,
            expression: Rc::new(expression),
        }))
    }
}
impl Display for UnaryExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.operator, self.expression)
    }
}

// binary         → expression operator expression
// operator       → "==" | "!=" | "<" | "<=" | ">" | ">=" | "+" | "-" | "*" | "/"
#[derive(Clone, Debug, Display, Copy)]
pub enum BinaryOperator {
    #[strum(to_string = "==")]
    EqualEqual,
    #[strum(to_string = "!=")]
    BangEqual,
    #[strum(to_string = "<")]
    Less,
    #[strum(to_string = "<=")]
    LessEqual,
    #[strum(to_string = ">")]
    Greater,
    #[strum(to_string = ">=")]
    GreaterEqual,
    #[strum(to_string = "+")]
    Plus,
    #[strum(to_string = "-")]
    Minus,
    #[strum(to_string = "*")]
    Star,
    #[strum(to_string = "/")]
    Slash,
}
impl TryFrom<Token> for BinaryOperator {
    type Error = ParseError;
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
            _ => Err(ParseError::IncorrectTokenConversion {
                base_token: token,
                converted_to: "BinaryOperator",
            }),
        }
    }
}
#[derive(Clone, Debug)]
pub struct BinaryExpression {
    pub left_expression: Rc<Expression>,
    pub operator: BinaryOperator,
    pub right_expression: Rc<Expression>,
}
impl BinaryExpression {
    pub fn wrap(self) -> Expression {
        Expression::Binary(self)
    }
    pub fn build(
        left_expression: Expression,
        token: Token,
        right_expression: Expression,
    ) -> Result<Expression, ParseError> {
        let operator = token.try_into()?;
        Ok(Expression::Binary(Self {
            left_expression: Rc::new(left_expression),
            operator,
            right_expression: Rc::new(right_expression),
        }))
    }
}

impl std::fmt::Display for BinaryExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.left_expression, self.operator, self.right_expression,
        )
    }
}
// logical -> expression operator expression
// operator -> "and" | "or"
#[derive(Clone, Debug, Display)]
pub enum LogicalOperator {
    #[strum(to_string = "and")]
    And,
    #[strum(to_string = "or")]
    Or,
}
impl TryFrom<Token> for LogicalOperator {
    type Error = ParseError;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token {
            Token::And => Ok(LogicalOperator::And),
            Token::Or => Ok(LogicalOperator::Or),
            _ => Err(ParseError::IncorrectTokenConversion {
                base_token: token,
                converted_to: "LogicalOperator",
            }),
        }
    }
}
#[derive(Clone, Debug)]
pub struct LogicalExpression {
    pub left_expression: Rc<Expression>,
    pub operator: LogicalOperator,
    pub right_expression: Rc<Expression>,
}
impl LogicalExpression {
    pub fn wrap(self) -> Expression {
        Expression::Logical(self)
    }
    pub fn build(
        left_expression: Expression,
        token: Token,
        right_expression: Expression,
    ) -> Result<Expression, ParseError> {
        let operator = token.try_into()?;
        Ok(Expression::Logical(Self {
            left_expression: Rc::new(left_expression),
            operator,
            right_expression: Rc::new(right_expression),
        }))
    }
}

impl std::fmt::Display for LogicalExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.left_expression, self.operator, self.right_expression,
        )
    }
}

// variable -> IDENTIFIER
#[derive(Clone, Debug, Display, PartialEq, Eq, Hash, PartialOrd)]
pub enum VariableToken {
    #[strum(to_string = "{0}")]
    Identifier(String),
    This,
    Super,
}
impl TryFrom<Token> for VariableToken {
    type Error = ParseError;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token {
            Token::Identifier(i) => Ok(VariableToken::Identifier(i)),
            Token::This => Ok(VariableToken::This),
            Token::Super => Ok(VariableToken::Super),
            _ => Err(ParseError::IncorrectTokenConversion {
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
    pub fn build(token: Token) -> Result<Expression, ParseError> {
        let variable = token.try_into()?;
        Ok(Expression::Variable(Self { variable }))
    }
    pub fn new(token: Token) -> Result<Self, ParseError> {
        let variable = token.try_into()?;
        Ok(Self { variable })
    }
}

impl std::fmt::Display for VariableExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.variable)
    }
}

// assignment -> IDENTIFIER = expression
#[derive(Clone, Debug)]
pub struct AssignmentExpression {
    pub variable: VariableToken,
    pub expression: Rc<Expression>,
}
impl AssignmentExpression {
    pub fn wrap(self) -> Expression {
        Expression::Assignment(self)
    }
    pub fn from(variable: VariableToken, expression: Expression) -> Expression {
        Expression::Assignment(Self {
            variable,
            expression: Rc::new(expression),
        })
    }
}

impl std::fmt::Display for AssignmentExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.variable, self.expression)
    }
}

// call -> primary (callExpr | getExpr)*
// callExpr -> "(" arguments? ")"
#[derive(Clone, Debug)]
pub struct CallExpression {
    pub callee: Rc<Expression>,
    pub arguments: Vec<Expression>,
}
impl CallExpression {
    pub fn wrap(self) -> Expression {
        Expression::Call(self)
    }
    pub fn build(callee: Expression, arguments: Vec<Expression>) -> Expression {
        Expression::Call(Self {
            callee: Rc::new(callee),
            arguments,
        })
    }
}
impl std::fmt::Display for CallExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({:?})", self.callee, self.arguments)
    }
}

// getExpr -> ( "." IDENTIFIER )*
#[derive(Clone, Debug)]
pub struct GetExpression {
    pub callee: Rc<Expression>,
    pub property: VariableToken,
}
impl GetExpression {
    pub fn wrap(self) -> Expression {
        Expression::Get(self)
    }
    pub fn build(
        callee: Expression,
        property: Token,
    ) -> Result<Expression, ParseError> {
        let property = property.try_into()?;
        Ok(Expression::Get(Self {
            callee: Rc::new(callee),
            property,
        }))
    }
}

impl std::fmt::Display for GetExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.callee, self.property)
    }
}

#[derive(Clone, Debug)]
pub struct SetExpression {
    pub callee: Rc<Expression>,
    pub property: VariableToken,
    pub value: Rc<Expression>,
}
impl SetExpression {
    pub fn wrap(self) -> Expression {
        Expression::Set(self)
    }
    pub fn from(
        callee: Rc<Expression>,
        property: VariableToken,
        value: Expression,
    ) -> Expression {
        Expression::Set(Self {
            callee,
            property,
            value: Rc::new(value),
        })
    }
}

impl std::fmt::Display for SetExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{} = {}", self.callee, self.property, self.value)
    }
}

// this -> "this"
#[derive(Clone, Debug)]
pub struct ThisExpression {
    pub token: VariableToken,
}
impl ThisExpression {
    pub fn wrap(self) -> Expression {
        Expression::This(self)
    }
    pub fn build(token: Token) -> Result<Expression, ParseError> {
        let this_token = token.clone().try_into()?;
        if !matches!(this_token, VariableToken::This) {
            return Err(ParseError::IncorrectTokenConversion {
                base_token: token,
                converted_to: "ThisToken",
            });
        }
        Ok(Expression::This(Self { token: this_token }))
    }
}

impl Display for ThisExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "this")
    }
}

// super -> "super" "." method
#[derive(Clone, Debug)]
pub struct SuperExpression {
    pub keyword: VariableToken,
    pub method: VariableToken,
}
impl SuperExpression {
    pub fn wrap(self) -> Expression {
        Expression::Super(self)
    }
    pub fn build(
        token: Token,
        method: Token,
    ) -> Result<Expression, ParseError> {
        let this_token = token.clone().try_into()?;
        if !matches!(this_token, VariableToken::Super) {
            return Err(ParseError::IncorrectTokenConversion {
                base_token: token,
                converted_to: "SuperToken",
            });
        }
        let method_token = method.clone().try_into()?;
        if !matches!(method_token, VariableToken::Identifier(_)) {
            return Err(ParseError::IncorrectTokenConversion {
                base_token: method,
                converted_to: "MethodToken",
            });
        }
        Ok(Expression::Super(Self {
            keyword: this_token,
            method: method_token,
        }))
    }
}

impl Display for SuperExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "super.{}", self.method)
    }
}
