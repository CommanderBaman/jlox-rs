#![allow(dead_code)]

use strum::Display;

use crate::{
    error::LanguageError,
    expression::{Expression, VariableToken},
    token::Token,
};

// NOTE: diverged a bit from the grammar notation
// everything else remains same
// I merged declaration into statement
//
//
// grammar
// program   -> statement* EOF
// statement -> variableDeclaration | exprStmt | printStmt | blockStmt
// exprStmt  -> expression ";"
// printStmt -> "print" expression ";"
// variableDeclaration -> "var" IDENTIFIER ( "=" expression )? ";"
// blockStmt -> "{" statement* "}"
#[derive(Debug, Display)]
pub enum Statement {
    #[strum(to_string = "Expression({0:?})")]
    Expression(ExpressionStatement),
    #[strum(to_string = "Print({0:?})")]
    Print(PrintStatement),
    #[strum(to_string = "VariableDeclaration({0:?})")]
    VariableDeclaration(VariableDeclarationStatement),
    #[strum(to_string = "Block({0:?})")]
    Block(BlockStatement),
}

pub trait StatementVisitor<R> {
    // NOTE: expression and print require mut because of expressions like this
    // (a = 2) OR  print a = 2;
    // both of them have assignment expression and can change state
    fn visit_expression(&mut self, statement: &ExpressionStatement) -> R;
    fn visit_print(&mut self, statement: &PrintStatement) -> R;
    fn visit_variable_declaration(
        &mut self,
        statement: &VariableDeclarationStatement,
    ) -> R;
    fn visit_block(&mut self, statement: &BlockStatement) -> R;
}

impl Statement {
    pub fn accept<R, T: StatementVisitor<R>>(&self, visitor: &mut T) -> R {
        match self {
            Statement::Print(s) => visitor.visit_print(s),
            Statement::Expression(s) => visitor.visit_expression(s),
            Statement::VariableDeclaration(s) => {
                visitor.visit_variable_declaration(s)
            }
            Statement::Block(s) => visitor.visit_block(s),
        }
    }
}

// exprStmt  → expression ";"
#[derive(Debug)]
pub struct ExpressionStatement {
    pub expression: Expression,
}

impl ExpressionStatement {
    pub fn new(expression: Expression) -> Statement {
        Statement::Expression(Self { expression })
    }
}

// printStmt → "print" expression ";"
#[derive(Debug)]
pub struct PrintStatement {
    pub expression: Expression,
}

impl PrintStatement {
    pub fn new(expression: Expression) -> Statement {
        Statement::Print(Self { expression })
    }
}

// variableDeclaration -> "var" IDENTIFIER ( "=" expression )? ";"
#[derive(Debug)]
pub struct VariableDeclarationStatement {
    pub name: VariableToken,
    pub expression: Option<Expression>,
}

impl VariableDeclarationStatement {
    pub fn new(
        name: Token,
        expression: Option<Expression>,
    ) -> Result<Statement, LanguageError> {
        Ok(Statement::VariableDeclaration(Self {
            name: name.try_into()?,
            expression,
        }))
    }
}

// block
#[derive(Debug)]
pub struct BlockStatement {
    pub statements: Vec<Statement>,
}

impl BlockStatement {
    pub fn new(statements: Vec<Statement>) -> Statement {
        Statement::Block(Self { statements })
    }
}
