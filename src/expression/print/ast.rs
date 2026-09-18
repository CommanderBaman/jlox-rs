#![allow(dead_code)]

use crate::expression::{
    BinaryExpression, Expression, GroupingExpression, LiteralExpression, UnaryExpression,
};

pub trait ExpressionAstString: Expression {
    fn to_ast_string(&self) -> String;
}

impl<T: ExpressionAstString> ExpressionAstString for GroupingExpression<T> {
    fn to_ast_string(&self) -> String {
        format!("(group {})", self.expression.to_ast_string())
    }
}

impl<T: ExpressionAstString> ExpressionAstString for UnaryExpression<T> {
    fn to_ast_string(&self) -> String {
        format!("({} {})", self.operator, self.expression.to_ast_string())
    }
}

impl<L: ExpressionAstString, R: ExpressionAstString> ExpressionAstString
    for BinaryExpression<L, R>
{
    fn to_ast_string(&self) -> String {
        format!(
            "({} {} {})",
            self.operator,
            self.left_expression.to_ast_string(),
            self.right_expression.to_ast_string()
        )
    }
}

impl ExpressionAstString for LiteralExpression {
    fn to_ast_string(&self) -> String {
        self.literal.to_string()
    }
}

pub struct AstPrinter;

impl AstPrinter {
    pub fn new() -> Self {
        Self {}
    }
    pub fn make_string<T: ExpressionAstString>(&self, expression: &T) -> String {
        expression.to_ast_string()
    }
}

#[cfg(test)]
mod test {
    use crate::{expression, token::Token};

    use super::*;

    #[test]
    fn ast_printer_literal() {
        let expressions = vec![
            (
                LiteralExpression::new(Token::True).expect("true literal correct conversion"),
                "True",
            ),
            (
                LiteralExpression::new(Token::False).expect("false literal correct conversion"),
                "False",
            ),
        ];

        let printer = AstPrinter::new();
        for (expression, result) in expressions {
            assert_eq!(result, printer.make_string(&expression))
        }
    }

    #[test]
    fn ast_printer_group() {
        let expressions = vec![
            (
                GroupingExpression::new(
                    LiteralExpression::new(Token::True).expect("true literal correct conversion"),
                ),
                "(group True)",
            ),
            (
                GroupingExpression::new(
                    LiteralExpression::new(Token::False).expect("false literal correct conversion"),
                ),
                "(group False)",
            ),
        ];

        let printer = AstPrinter::new();
        for (expression, result) in expressions {
            assert_eq!(result, printer.make_string(&expression))
        }
    }

    #[test]
    fn ast_printer_provided_example() {
        let expression = BinaryExpression::new(
            UnaryExpression::new(
                Token::Minus,
                LiteralExpression::new(Token::Number(123.0)).expect("123 is literal"),
            )
            .expect("-123 is unary expression"),
            Token::Star,
            GroupingExpression::new(
                LiteralExpression::new(Token::Number(45.67)).expect("45.67 is literal"),
            ),
        )
        .expect("-123 * (45.67) is a correct expression");
        let printer = AstPrinter::new();
        assert_eq!(
            printer.make_string(&expression),
            "(Star (Minus Number(123)) (group Number(45.67)))"
        );
    }
}
