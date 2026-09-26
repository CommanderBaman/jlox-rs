#![allow(dead_code)]

use strum::Display;

use crate::expression::Expression;

// grammar
// program   → statement* EOF
// statement → exprStmt | printStmt
// exprStmt  → expression ";"
// printStmt → "print" expression ";"
#[derive(Debug, Display)]
pub enum Statement {
    #[strum(to_string = "Expression({0:?})")]
    Expression(ExpressionStatement),
    #[strum(to_string = "Print({0:?})")]
    Print(PrintStatement),
}

pub trait StatementVisitor<R> {
    fn visit_expression(&self, statement: &ExpressionStatement) -> R;
    fn visit_print(&self, statement: &PrintStatement) -> R;
}

impl Statement {
    pub fn accept<R, T: StatementVisitor<R>>(&self, visitor: &T) -> R {
        match self {
            Statement::Print(s) => visitor.visit_print(s),
            Statement::Expression(s) => visitor.visit_expression(s),
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
