#![allow(dead_code)]

use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    expression::{Expression, VariableExpression, VariableToken},
    parser::ParseError,
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
// program   -> declaration* EOF
// declaration -> functionDeclaration | variableDeclaration
//                  | classDeclaration | statement
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
// classDeclaration ->  "class" IDENTIFIER ( "<" IDENTIFIER )? "{" function* "}"
//
// NOTE: forStmt is syntactic sugar for whileStmt
#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Rc<ExpressionStatement>),
    Print(Rc<PrintStatement>),
    VariableDeclaration(Rc<VariableDeclarationStatement>),
    Block(Rc<BlockStatement>),
    If(Rc<IfStatement>),
    While(Rc<WhileStatement>),
    Function(Rc<FunctionDeclarationStatement>),
    Return(Rc<ReturnStatement>),
    Class(Rc<ClassDeclarationStatement>),
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Self::Expression(e) => format!("{}", e),
            Self::Print(e) => format!("{}", e),
            Self::VariableDeclaration(e) => format!("{}", e),
            Self::Block(e) => format!("{}", e),
            Self::If(e) => format!("{}", e),
            Self::While(e) => format!("{}", e),
            Self::Function(e) => format!("{}", e),
            Self::Return(e) => format!("{}", e),
            Self::Class(e) => format!("{}", e),
        };
        write!(f, "{}", string)
    }
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
    fn visit_class(&mut self, statement: Rc<ClassDeclarationStatement>) -> R;
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
            Statement::Class(s) => visitor.visit_class(s.clone()),
        }
    }
}

// exprStmt  → expression ";"
#[derive(Debug, Clone)]
pub struct ExpressionStatement {
    pub expression: Expression,
}

impl ExpressionStatement {
    pub fn build(expression: Expression) -> Statement {
        Statement::Expression(Rc::new(Self { expression }))
    }
}

impl Display for ExpressionStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{};", self.expression)
    }
}

// printStmt → "print" expression ";"
#[derive(Debug, Clone)]
pub struct PrintStatement {
    pub expression: Expression,
}

impl PrintStatement {
    pub fn build(expression: Expression) -> Statement {
        Statement::Print(Rc::new(Self { expression }))
    }
}

impl Display for PrintStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "print {};", self.expression)
    }
}

// variableDeclaration -> "var" IDENTIFIER ( "=" expression )? ";"
#[derive(Debug, Clone)]
pub struct VariableDeclarationStatement {
    pub name: VariableToken,
    pub expression: Option<Expression>,
}

impl VariableDeclarationStatement {
    pub fn build(
        name: Token,
        expression: Option<Expression>,
    ) -> Result<Statement, ParseError> {
        Ok(Statement::VariableDeclaration(Rc::new(Self {
            name: name.try_into()?,
            expression,
        })))
    }
}

impl Display for VariableDeclarationStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let expression_string = self
            .expression
            .as_ref()
            .map_or_default(|e| format!(" = {}", e));
        write!(f, "var {}{};", self.name, expression_string)
    }
}

// blockStmt -> "{" statement* "}"
#[derive(Debug, Clone)]
pub struct BlockStatement {
    pub statements: Vec<Rc<Statement>>,
}

impl BlockStatement {
    pub fn build(statements: Vec<Statement>) -> Statement {
        Statement::Block(Rc::new(Self {
            statements: statements.into_iter().map(Rc::new).collect(),
        }))
    }
}

impl Display for BlockStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ {} }}",
            self.statements
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join("; ")
        )
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
    pub fn build(
        condition: Expression,
        then_branch: Statement,
        else_branch: Option<Statement>,
    ) -> Statement {
        Statement::If(Rc::new(Self {
            condition,
            then_branch: Rc::new(then_branch),
            else_branch: else_branch.map(Rc::new),
        }))
    }
}

impl Display for IfStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let else_string = self
            .else_branch
            .as_ref()
            .map_or_default(|b| format!(" else {}", b));
        write!(
            f,
            "if ({}) {}{}",
            self.condition, self.then_branch, else_string
        )
    }
}

// whileStmt -> "while" "(" expression ")" statement
#[derive(Debug, Clone)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Rc<Statement>,
}

impl WhileStatement {
    pub fn build(condition: Expression, body: Statement) -> Statement {
        Statement::While(Rc::new(Self {
            condition,
            body: Rc::new(body),
        }))
    }
}

impl Display for WhileStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "while ({}) {}", self.condition, self.body)
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
    pub fn build(
        name: Token,
        parameters: Vec<Token>,
        body: Statement,
    ) -> Result<Statement, ParseError> {
        let name = name.try_into()?;
        let parameters: Vec<VariableToken> = parameters
            .into_iter()
            .map(<Token as TryInto<VariableToken>>::try_into)
            .collect::<Result<Vec<VariableToken>, ParseError>>()?;
        let Statement::Block(body) = body else {
            return Err(ParseError::IncorrectStatementConversion {
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

impl Display for FunctionDeclarationStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "fun {}({}) {}",
            self.name,
            self.parameters
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            self.body
        )
    }
}

// returnStmt -> "return" expression? ";"
#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub value: Option<Expression>,
}

impl ReturnStatement {
    pub fn build(value: Option<Expression>) -> Statement {
        Statement::Return(Rc::new(Self { value }))
    }
}

impl Display for ReturnStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "return{};",
            self.value.as_ref().map_or_default(|e| format!(" {e}"))
        )
    }
}

// classDeclaration ->  "class" IDENTIFIER "{" function* "}"
#[derive(Debug, Clone)]
pub struct ClassDeclarationStatement {
    // NOTE: I could have used a vector for the functions, but it is better to
    // store them in a hashmap so that we can use this statement directly in
    // the lox class definition like the lox function
    pub name: VariableToken,
    pub functions: HashMap<VariableToken, Rc<FunctionDeclarationStatement>>,
    // I am confused as to keep variable expression or expression
    // keeping variable expression as it feels correct
    pub superclass: Option<VariableExpression>,
}

impl ClassDeclarationStatement {
    pub fn build(
        name: Token,
        functions: Vec<Statement>,
        superclass: Option<VariableExpression>,
    ) -> Result<Statement, ParseError> {
        let name = name.try_into()?;
        let functions = functions
            .into_iter()
            .map(|stmt| match stmt {
                Statement::Function(f) => Ok((f.name.clone(), f)),
                _ => Err(ParseError::IncorrectStatementConversion {
                    base_statement: stmt,
                    converted_to: "FunctionDeclarationStatement",
                }),
            })
            .collect::<Result<
                HashMap<VariableToken, Rc<FunctionDeclarationStatement>>,
                ParseError,
            >>()?;
        Ok(Statement::Class(Rc::new(Self {
            name,
            functions,
            superclass,
        })))
    }
}

impl Display for ClassDeclarationStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "class {} {{{}}}",
            self.name,
            self.functions
                .values()
                .map(|f| format!("{f}"))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
