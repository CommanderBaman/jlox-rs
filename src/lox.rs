use crate::{
    function::add_native_functions,
    interpreter::{Interpreter, RuntimeError},
    parser::{ParseError, parse},
    resolver::{Resolver, ResolverError},
    token::{TokenError, scan_tokens},
};
use std::{process::ExitCode, rc::Rc};
use thiserror::Error;

pub struct Lox {
    interpreter: Interpreter,
    // interactive: bool,
}

#[derive(Error, Debug)]
pub enum LanguageError {
    #[error("parse errors:\n{}", .0.iter().fold(String::new(), |acc, e| format!("{acc}\n* {e}")).split_off(1))]
    Parse(Vec<ParseError>),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error(transparent)]
    Resolver(#[from] ResolverError),
    #[error(transparent)]
    Token(#[from] TokenError),
}

impl From<Vec<ParseError>> for LanguageError {
    fn from(value: Vec<ParseError>) -> Self {
        Self::Parse(value)
    }
}

impl From<LanguageError> for ExitCode {
    fn from(value: LanguageError) -> Self {
        let error_code = match value {
            LanguageError::Token(_)
            | LanguageError::Parse(_)
            | LanguageError::Resolver(_) => 65,
            LanguageError::Runtime(_) => 70,
        };
        ExitCode::from(error_code)
    }
}

impl Lox {
    pub fn new(interactive: bool) -> Result<Self, LanguageError> {
        let mut interpreter = Interpreter::new(interactive);
        add_native_functions(&mut interpreter)?;
        Ok(Self {
            interpreter, // interactive,
        })
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

        let statements = parse(&tokens)?.into_iter().map(Rc::new).collect();

        let mut resolver = Resolver::new(&mut self.interpreter);
        resolver.resolve(&statements)?;
        self.interpreter.interpret(&statements)?;
        Ok(())
    }
}
