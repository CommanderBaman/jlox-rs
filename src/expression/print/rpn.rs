#![allow(dead_code)]

use crate::expression::{
    BinaryExpression, Expression, GroupingExpression, LiteralExpression, UnaryExpression,
};

pub trait ExpressionRpn: Expression {
    fn to_rpn_string(&self) -> String;
}

impl<T: ExpressionRpn> ExpressionRpn for GroupingExpression<T> {
    fn to_rpn_string(&self) -> String {
        format!("{}", self.expression.to_rpn_string())
    }
}

impl<T: ExpressionRpn> ExpressionRpn for UnaryExpression<T> {
    fn to_rpn_string(&self) -> String {
        format!("{} {}", self.expression.to_rpn_string(), self.operator)
    }
}

impl<L: ExpressionRpn, R: ExpressionRpn> ExpressionRpn for BinaryExpression<L, R> {
    fn to_rpn_string(&self) -> String {
        format!(
            "{} {} {}",
            self.left_expression.to_rpn_string(),
            self.right_expression.to_rpn_string(),
            self.operator,
        )
    }
}

impl ExpressionRpn for LiteralExpression {
    fn to_rpn_string(&self) -> String {
        self.literal.to_string()
    }
}

pub struct AstPrinter;

impl AstPrinter {
    pub fn new() -> Self {
        Self {}
    }
    pub fn to_string<T: ExpressionRpn>(&self, expression: &T) -> String {
        expression.to_rpn_string()
    }
}

#[cfg(test)]
mod test {
    use crate::token::Token;

    use super::*;

    #[test]
    fn print_example() {
        let expression = BinaryExpression::new(
            GroupingExpression::new(
                BinaryExpression::new(
                    LiteralExpression::new(Token::Number(1.0)).expect("1 is literal"),
                    Token::Plus,
                    LiteralExpression::new(Token::Number(2.0)).expect("2 is literal"),
                )
                .expect("1 + 2 is binary expression"),
            ),
            Token::Star,
            GroupingExpression::new(
                BinaryExpression::new(
                    LiteralExpression::new(Token::Number(4.0)).expect("4 is literal"),
                    Token::Minus,
                    LiteralExpression::new(Token::Number(3.0)).expect("3 is literal"),
                )
                .expect("4 - 3 is binary expression"),
            ),
        )
        .expect("(1 + 2) * (4 - 3) is expression");
        let printer = AstPrinter::new();
        assert_eq!(
            printer.to_string(&expression),
            "Number(1) Number(2) Plus Number(4) Number(3) Minus Star"
        );
    }
}
