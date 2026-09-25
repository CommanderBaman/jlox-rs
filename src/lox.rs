use crate::{
    error::LanguageError,
    expression::{
        BinaryExpression, Expression, GroupingExpression, LiteralExpression,
        UnaryExpression, print::rpn::RpnPrinter as ExpressionPrinter,
    },
    token::{Token, scan_tokens},
};

pub struct Lox {}

fn build_expression() -> Expression {
    BinaryExpression::new(
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
    .expect("-123 * (45.67) is a correct expression")
}

impl Lox {
    pub fn new() -> Self {
        Lox {}
    }
    pub fn run(
        &self,
        line: &str,
        _line_number: &u64,
    ) -> Result<(), LanguageError> {
        let tokens = scan_tokens(line)?;
        for token in tokens {
            println!("{token}");
        }
        let expression = build_expression();
        let printer = ExpressionPrinter::new();
        println!("rpn = {}", printer.to_string(&expression));
        Ok(())
    }
}
