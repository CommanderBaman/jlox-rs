use crate::{
    error::LanguageError,
    expression::print::rpn::RpnPrinter as ExpressionPrinter,
    interpreter::Interpreter, parser::parse_expression, token::scan_tokens,
};

pub struct Lox {}

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
        let expression = parse_expression(&tokens)?;
        let printer = ExpressionPrinter::new();
        println!("expression = {}", printer.to_string(&expression));
        let interpreter = Interpreter::new();
        let value = interpreter.evaluate(&expression)?;
        println!("value = {}", value);
        Ok(())
    }
}
