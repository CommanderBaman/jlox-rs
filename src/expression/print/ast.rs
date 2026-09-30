#![allow(dead_code)]

use crate::expression::{
    AssignmentExpression, Expression, ExpressionVisitor, LogicalExpression,
    VariableExpression,
};

pub struct SimpleAstPrinter {}

impl SimpleAstPrinter {
    pub fn new() -> Self {
        Self {}
    }
    pub fn to_string(&mut self, expression: &Expression) -> String {
        expression.accept(self)
    }
}

impl ExpressionVisitor<String> for SimpleAstPrinter {
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
        format!("(group {})", expression.expression.accept(self))
    }
    fn visit_unary(
        &mut self,
        expression: &crate::expression::UnaryExpression,
    ) -> String {
        format!(
            "({} {})",
            expression.operator,
            expression.expression.accept(self),
        )
    }
    fn visit_binary(
        &mut self,
        expression: &crate::expression::BinaryExpression,
    ) -> String {
        format!(
            "({} {} {})",
            expression.operator,
            expression.left_expression.accept(self),
            expression.right_expression.accept(self),
        )
    }
    fn visit_variable(&self, expression: &VariableExpression) -> String {
        format!("(variable {})", expression.variable)
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
}

#[cfg(test)]
mod test {
    use crate::{
        expression::{
            BinaryExpression, GroupingExpression, LiteralExpression,
            UnaryExpression,
        },
        token::Token,
    };

    use super::*;

    #[test]
    fn ast_printer_literal() {
        let expressions = vec![
            (
                LiteralExpression::new(Token::True)
                    .expect("true literal correct conversion"),
                "True",
            ),
            (
                LiteralExpression::new(Token::False)
                    .expect("false literal correct conversion"),
                "False",
            ),
        ];

        let printer = SimpleAstPrinter::new();
        for (expression, result) in expressions {
            assert_eq!(result, printer.to_string(&expression))
        }
    }

    #[test]
    fn ast_printer_group() {
        let expressions = vec![
            (
                GroupingExpression::new(
                    LiteralExpression::new(Token::True)
                        .expect("true literal correct conversion"),
                ),
                "(group True)",
            ),
            (
                GroupingExpression::new(
                    LiteralExpression::new(Token::False)
                        .expect("false literal correct conversion"),
                ),
                "(group False)",
            ),
        ];

        let printer = SimpleAstPrinter::new();
        for (expression, result) in expressions {
            assert_eq!(result, printer.to_string(&expression))
        }
    }

    #[test]
    fn ast_printer_provided_example() {
        let expression = BinaryExpression::new(
            UnaryExpression::new(
                Token::Minus,
                LiteralExpression::new(Token::Number(123.0))
                    .expect("123 is literal"),
            )
            .expect("-123 is unary expression"),
            Token::Star,
            GroupingExpression::new(
                LiteralExpression::new(Token::Number(45.67))
                    .expect("45.67 is literal"),
            ),
        )
        .expect("-123 * (45.67) is a correct expression");
        let printer = SimpleAstPrinter::new();
        assert_eq!(
            printer.to_string(&expression),
            "(Star (Minus Number(123)) (group Number(45.67)))"
        );
    }
}
