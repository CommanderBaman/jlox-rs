#![allow(dead_code)]

use crate::expression::{
    AssignmentExpression, CallExpression, Expression, ExpressionVisitor,
    LogicalExpression, VariableExpression,
};

pub struct RpnPrinter {}

impl RpnPrinter {
    pub fn new() -> Self {
        Self {}
    }
    pub fn to_string(&mut self, expression: &Expression) -> String {
        expression.accept(self)
    }
}

impl ExpressionVisitor<String> for RpnPrinter {
    fn visit_literal(
        &self,
        expression: &crate::expression::LiteralExpression,
    ) -> String {
        expression.literal.to_string()
    }
    fn visit_grouping(
        &mut self,
        expression: &crate::expression::GroupingExpression,
    ) -> String {
        expression.expression.accept(self)
    }
    fn visit_unary(
        &mut self,
        expression: &crate::expression::UnaryExpression,
    ) -> String {
        format!(
            "{} {}",
            expression.expression.accept(self),
            expression.operator
        )
    }
    fn visit_binary(
        &mut self,
        expression: &crate::expression::BinaryExpression,
    ) -> String {
        format!(
            "{} {} {}",
            expression.left_expression.accept(self),
            expression.right_expression.accept(self),
            expression.operator
        )
    }
    fn visit_variable(&mut self, _expression: &VariableExpression) -> String {
        unimplemented!()
    }
    fn visit_assignment(
        &mut self,
        _expression: &AssignmentExpression,
    ) -> String {
        unimplemented!()
    }
    fn visit_logical(&mut self, _expression: &LogicalExpression) -> String {
        unimplemented!()
    }
    fn visit_call(&mut self, _expression: &CallExpression) -> String {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use crate::{
        expression::{BinaryExpression, GroupingExpression, LiteralExpression},
        token::Token,
    };

    use super::*;

    #[test]
    fn print_example() {
        let expression = BinaryExpression::new(
            GroupingExpression::new(
                BinaryExpression::new(
                    LiteralExpression::new(Token::Number(1.0))
                        .expect("1 is literal"),
                    Token::Plus,
                    LiteralExpression::new(Token::Number(2.0))
                        .expect("2 is literal"),
                )
                .expect("1 + 2 is binary expression"),
            ),
            Token::Star,
            GroupingExpression::new(
                BinaryExpression::new(
                    LiteralExpression::new(Token::Number(4.0))
                        .expect("4 is literal"),
                    Token::Minus,
                    LiteralExpression::new(Token::Number(3.0))
                        .expect("3 is literal"),
                )
                .expect("4 - 3 is binary expression"),
            ),
        )
        .expect("(1 + 2) * (4 - 3) is expression");
        let mut printer = RpnPrinter::new();
        assert_eq!(
            printer.to_string(&expression),
            "Number(1) Number(2) Plus Number(4) Number(3) Minus Star"
        );
    }
}
