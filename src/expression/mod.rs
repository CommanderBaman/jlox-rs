use crate::token::Token;

pub mod print;

// NOTE: this doesn't really follow the visitor pattern
// but takes help of rust trait system to implement things
//
// I couldn't apply the visitor system because
// because my expression is not a single type but a trait
// but it still follows the extension method because

// Grammar
// expression     → literal
//                | unary
//                | binary
//                | grouping ;
pub trait Expression {}

// literal        → NUMBER | STRING | "true" | "false" | "nil"
pub struct LiteralExpression {
    literal: Token,
}
impl LiteralExpression {
    pub fn new(literal: Token) -> Option<Self> {
        match literal {
            Token::Number(_)
            | Token::String(_)
            | Token::True
            | Token::False
            | Token::Nil => Some(Self { literal }),
            _ => None,
        }
    }
}
impl Expression for LiteralExpression {}

// grouping       → "(" expression ")"
pub struct GroupingExpression<T: Expression> {
    expression: T,
}
impl<T: Expression> GroupingExpression<T> {
    pub fn new(expression: T) -> Self {
        Self { expression }
    }
}
impl<T: Expression> Expression for GroupingExpression<T> {}

// unary          → ( "-" | "!" ) expression
pub struct UnaryExpression<T: Expression> {
    operator: Token,
    expression: T,
}
impl<T: Expression> UnaryExpression<T> {
    pub fn new(operator: Token, expression: T) -> Option<Self> {
        match operator {
            Token::Bang | Token::Minus => Some(Self {
                operator,
                expression,
            }),
            _ => None,
        }
    }
}
impl<T: Expression> Expression for UnaryExpression<T> {}

// binary         → expression operator expression
// operator       → "==" | "!=" | "<" | "<=" | ">" | ">=" | "+" | "-" | "*" | "/"
pub struct BinaryExpression<L: Expression, R: Expression> {
    left_expression: L,
    operator: Token,
    right_expression: R,
}
impl<L: Expression, R: Expression> BinaryExpression<L, R> {
    pub fn new(
        left_expression: L,
        operator: Token,
        right_expression: R,
    ) -> Option<Self> {
        match operator {
            Token::EqualEqual
            | Token::BangEqual
            | Token::Less
            | Token::LessEqual
            | Token::Greater
            | Token::GreaterEqual
            | Token::Plus
            | Token::Minus
            | Token::Star
            | Token::Slash => Some(Self {
                left_expression,
                operator,
                right_expression,
            }),
            _ => None,
        }
    }
}
impl<L: Expression, R: Expression> Expression for BinaryExpression<L, R> {}
