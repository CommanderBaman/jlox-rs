#![allow(dead_code)]

use crate::expression::{
    AssignmentExpression, CallExpression, Expression, ExpressionVisitor,
    GetExpression, LogicalExpression, SetExpression, SuperExpression,
    ThisExpression, VariableExpression,
};

pub struct SimpleAstPrinter {}

impl SimpleAstPrinter {
    pub fn new() -> Self {
        Self {}
    }
    pub fn form_string(&mut self, expression: &Expression) -> String {
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
    fn visit_variable(&mut self, expression: &VariableExpression) -> String {
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
    fn visit_call(&mut self, _expression: &CallExpression) -> String {
        unimplemented!()
    }
    fn visit_get(&mut self, _expression: &GetExpression) -> String {
        unimplemented!()
    }
    fn visit_set(&mut self, _expression: &SetExpression) -> String {
        unimplemented!()
    }
    fn visit_this(&mut self, _expression: &ThisExpression) -> String {
        unimplemented!()
    }
    fn visit_super(&mut self, _expression: &SuperExpression) -> String {
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
                LiteralExpression::build(Token::True)
                    .expect("true literal correct conversion"),
                "true",
            ),
            (
                LiteralExpression::build(Token::False)
                    .expect("false literal correct conversion"),
                "false",
            ),
        ];

        let mut printer = SimpleAstPrinter::new();
        for (expression, result) in expressions {
            assert_eq!(result, printer.form_string(&expression))
        }
    }

    #[test]
    fn ast_printer_group() {
        let expressions = vec![
            (
                GroupingExpression::build(
                    LiteralExpression::build(Token::True)
                        .expect("true literal correct conversion"),
                ),
                "(group true)",
            ),
            (
                GroupingExpression::build(
                    LiteralExpression::build(Token::False)
                        .expect("false literal correct conversion"),
                ),
                "(group false)",
            ),
        ];

        let mut printer = SimpleAstPrinter::new();
        for (expression, result) in expressions {
            assert_eq!(result, printer.form_string(&expression))
        }
    }

    #[test]
    fn ast_printer_provided_example() {
        let expression = BinaryExpression::build(
            UnaryExpression::build(
                Token::Minus,
                LiteralExpression::build(Token::Number(123.0))
                    .expect("123 is literal"),
            )
            .expect("-123 is unary expression"),
            Token::Star,
            GroupingExpression::build(
                LiteralExpression::build(Token::Number(45.67))
                    .expect("45.67 is literal"),
            ),
        )
        .expect("-123 * (45.67) is a correct expression");
        let mut printer = SimpleAstPrinter::new();
        assert_eq!(
            printer.form_string(&expression),
            "(* (- 123) (group 45.67))"
        );
    }
}
