#![allow(dead_code)]

use strum::Display;

use crate::{
    error::LanguageError,
    expression::{Expression, VariableToken},
    token::Token,
};

// NOTE: I diverged a bit from the grammar notation
// everything else remains same
// I merged declaration into statement
// Later, I found it is wrong because of the case:
// if (<condition>) var a = 2;
// here we can not know where to define a: is it if scope or parent
// I am dumb sometimes
// I will keep the Statement enum as is
//
// grammar
// program   -> statement* EOF
// declaration -> variableDeclaration | statement
// statement -> exprStmt | printStmt | blockStmt | ifStmt | whileStmt | forStmt
// exprStmt  -> expression ";"
// printStmt -> "print" expression ";"
// variableDeclaration -> "var" IDENTIFIER ( "=" expression )? ";"
// blockStmt -> "{" declaration* "}"
// ifStmt -> "if" "(" expression ")" statement ( "else" statement )?
// whileStmt -> "while" "(" expression ")" statement
// forStmt -> "for" "(" varDecl | exprStmt | ";" )
//              expression? ";" expression? ")" statement
//
// NOTE: forStmt is syntactic sugar for whileStmt
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
    If(IfStatement),
    While(WhileStatement),
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
    fn visit_if(&mut self, statement: &IfStatement) -> R;
    fn visit_while(&mut self, statement: &WhileStatement) -> R;
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
            Statement::If(s) => visitor.visit_if(s),
            Statement::While(s) => visitor.visit_while(s),
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

// blockStmt -> "{" statement* "}"
#[derive(Debug)]
pub struct BlockStatement {
    pub statements: Vec<Statement>,
}

impl BlockStatement {
    pub fn new(statements: Vec<Statement>) -> Statement {
        Statement::Block(Self { statements })
    }
}

// ifStmt -> "if" "(" expression ")" statement ( "else" statement )?
#[derive(Debug)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_branch: Box<Statement>,
    pub else_branch: Option<Box<Statement>>,
}

impl IfStatement {
    pub fn new(
        condition: Expression,
        then_branch: Statement,
        else_branch: Option<Statement>,
    ) -> Statement {
        Statement::If(Self {
            condition,
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(|b| Box::new(b)),
        })
    }
}

// whileStmt -> "while" "(" expression ")" statement
#[derive(Debug)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Box<Statement>,
}

impl WhileStatement {
    pub fn new(condition: Expression, body: Statement) -> Statement {
        Statement::While(Self {
            condition,
            body: Box::new(body),
        })
    }
}
