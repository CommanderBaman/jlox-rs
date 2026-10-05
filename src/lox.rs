use crate::{
    error::LanguageError, function::add_native_functions,
    interpreter::Interpreter, parser::parse, resolver::Resolver,
    token::scan_tokens,
};

pub struct Lox {
    interpreter: Interpreter,
    // interactive: bool,
}

impl Lox {
    pub fn new(interactive: bool) -> Self {
        let mut interpreter = Interpreter::new(interactive);
        add_native_functions(&mut interpreter);
        Self {
            interpreter, // interactive,
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

        let mut resolver = Resolver::new(&mut self.interpreter);
        resolver.resolve(&statements)?;
        self.interpreter.interpret(&statements)?;
        Ok(())
    }
}
