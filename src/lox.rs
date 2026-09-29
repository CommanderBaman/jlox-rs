use crate::{
    error::LanguageError,
    // expression::print::rpn::RpnPrinter as ExpressionPrinter,
    interpreter::Interpreter,
    parser::parse,
    token::scan_tokens,
};

pub struct Lox {
    interpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
        }
    }
    pub fn run(
        &mut self,
        line: &str,
        _line_number: &u64,
    ) -> Result<(), LanguageError> {
        let tokens = scan_tokens(line)?;

        // // when expression evaluation is all you need
        // let expression = parse_expression(&tokens)?;
        // let printer = ExpressionPrinter::new();
        // println!("expression = {}", printer.to_string(&expression));
        // let interpreter = Interpreter::new();
        // let value = interpreter.evaluate(&expression)?;
        // println!("value = {}", value);

        let statements = parse(&tokens)?;
        self.interpreter.interpret(&statements)?;
        Ok(())
    }
}
