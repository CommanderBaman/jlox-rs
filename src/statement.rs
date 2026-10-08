#![allow(dead_code)]

use std::rc::Rc;

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
// declaration -> functionDeclaration | variableDeclaration | statement
// statement -> exprStmt | printStmt | blockStmt | ifStmt
//              | whileStmt | forStmt | returnStmt
// exprStmt  -> expression ";"
// printStmt -> "print" expression ";"
// variableDeclaration -> "var" IDENTIFIER ( "=" expression )? ";"
// blockStmt -> "{" declaration* "}"
// ifStmt -> "if" "(" expression ")" statement ( "else" statement )?
// whileStmt -> "while" "(" expression ")" statement
// forStmt -> "for" "(" varDecl | exprStmt | ";" )
//              expression? ";" expression? ")" statement
// functionDeclaration -> "fun" function
// function -> IDENTIFIER "(" parameters? ")" blockStmt
// parameters -> IDENTIFIER ( "," IDENTIFIER )*
// returnStmt -> "return" expression? ";"
//
// NOTE: forStmt is syntactic sugar for whileStmt
#[derive(Debug, Display, Clone)]
pub enum Statement {
    #[strum(to_string = "Expression({0:?})")]
    Expression(Rc<ExpressionStatement>),
    #[strum(to_string = "Print({0:?})")]
    Print(Rc<PrintStatement>),
    #[strum(to_string = "VariableDeclaration({0:?})")]
    VariableDeclaration(Rc<VariableDeclarationStatement>),
    #[strum(to_string = "Block({0:?})")]
    Block(Rc<BlockStatement>),
    If(Rc<IfStatement>),
    While(Rc<WhileStatement>),
    Function(Rc<FunctionDeclarationStatement>),
    Return(Rc<ReturnStatement>),
}

pub trait StatementVisitor<R> {
    // NOTE: expression and print require mut because of expressions like this
    // (a = 2) OR  print a = 2;
    // both of them have assignment expression and can change state
    fn visit_expression(&mut self, statement: Rc<ExpressionStatement>) -> R;
    fn visit_print(&mut self, statement: Rc<PrintStatement>) -> R;
    fn visit_variable_declaration(
        &mut self,
        statement: Rc<VariableDeclarationStatement>,
    ) -> R;
    fn visit_block(&mut self, statement: Rc<BlockStatement>) -> R;
    fn visit_if(&mut self, statement: Rc<IfStatement>) -> R;
    fn visit_while(&mut self, statement: Rc<WhileStatement>) -> R;
    fn visit_function(
        &mut self,
        statement: Rc<FunctionDeclarationStatement>,
    ) -> R;
    fn visit_return(&mut self, statement: Rc<ReturnStatement>) -> R;
}

impl Statement {
    pub fn accept<R, T: StatementVisitor<R>>(
        statement: Rc<Statement>,
        visitor: &mut T,
    ) -> R {
        match &*statement {
            Statement::Print(s) => visitor.visit_print(s.clone()),
            Statement::Expression(s) => visitor.visit_expression(s.clone()),
            Statement::VariableDeclaration(s) => {
                visitor.visit_variable_declaration(s.clone())
            }
            Statement::Block(s) => visitor.visit_block(s.clone()),
            Statement::If(s) => visitor.visit_if(s.clone()),
            Statement::While(s) => visitor.visit_while(s.clone()),
            Statement::Function(s) => visitor.visit_function(s.clone()),
            Statement::Return(s) => visitor.visit_return(s.clone()),
        }
    }
}

// exprStmt  → expression ";"
#[derive(Debug, Clone)]
pub struct ExpressionStatement {
    pub expression: Expression,
}

impl ExpressionStatement {
    pub fn new(expression: Expression) -> Statement {
        Statement::Expression(Rc::new(Self { expression }))
    }
}

// printStmt → "print" expression ";"
#[derive(Debug, Clone)]
pub struct PrintStatement {
    pub expression: Expression,
}

impl PrintStatement {
    pub fn new(expression: Expression) -> Statement {
        Statement::Print(Rc::new(Self { expression }))
    }
}

// variableDeclaration -> "var" IDENTIFIER ( "=" expression )? ";"
#[derive(Debug, Clone)]
pub struct VariableDeclarationStatement {
    pub name: VariableToken,
    pub expression: Option<Expression>,
}

impl VariableDeclarationStatement {
    pub fn new(
        name: Token,
        expression: Option<Expression>,
    ) -> Result<Statement, LanguageError> {
        Ok(Statement::VariableDeclaration(Rc::new(Self {
            name: name.try_into()?,
            expression,
        })))
    }
}

// blockStmt -> "{" statement* "}"
#[derive(Debug, Clone)]
pub struct BlockStatement {
    pub statements: Vec<Rc<Statement>>,
}

impl BlockStatement {
    pub fn new(statements: Vec<Statement>) -> Statement {
        Statement::Block(Rc::new(Self {
            statements: statements.into_iter().map(Rc::new).collect(),
        }))
    }
}

// ifStmt -> "if" "(" expression ")" statement ( "else" statement )?
#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_branch: Rc<Statement>,
    pub else_branch: Option<Rc<Statement>>,
}

impl IfStatement {
    pub fn new(
        condition: Expression,
        then_branch: Statement,
        else_branch: Option<Statement>,
    ) -> Statement {
        Statement::If(Rc::new(Self {
            condition,
            then_branch: Rc::new(then_branch),
            else_branch: else_branch.map(|b| Rc::new(b)),
        }))
    }
}

// whileStmt -> "while" "(" expression ")" statement
#[derive(Debug, Clone)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Rc<Statement>,
}

impl WhileStatement {
    pub fn new(condition: Expression, body: Statement) -> Statement {
        Statement::While(Rc::new(Self {
            condition,
            body: Rc::new(body),
        }))
    }
}

// functionDeclaration -> "fun" function
// function -> IDENTIFIER "(" parameters? ")" blockStmt
// parameters -> IDENTIFIER ( "," IDENTIFIER )*
#[derive(Debug, Clone)]
pub struct FunctionDeclarationStatement {
    pub name: VariableToken,
    pub parameters: Vec<VariableToken>,
    pub body: Rc<BlockStatement>,
}

impl FunctionDeclarationStatement {
    pub fn new(
        name: Token,
        parameters: Vec<Token>,
        body: Statement,
    ) -> Result<Statement, LanguageError> {
        let name = name.try_into()?;
        let parameters: Vec<VariableToken> = parameters
            .into_iter()
            .map(|p| <Token as TryInto<VariableToken>>::try_into(p))
            .collect::<Result<Vec<VariableToken>, LanguageError>>()?;
        let Statement::Block(body) = body else {
            return Err(LanguageError::IncorrectStatementConversion {
                base_statement: body,
                converted_to: "BlockStatement",
            });
        };
        Ok(Statement::Function(Rc::new(Self {
            name,
            parameters,
            body,
        })))
    }
}

// returnStmt -> "return" expression? ";"
#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub value: Option<Expression>,
}

impl ReturnStatement {
    pub fn new(value: Option<Expression>) -> Statement {
        Statement::Return(Rc::new(Self { value }))
    }
}
