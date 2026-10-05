use std::{collections::HashMap, mem};

use crate::{
    error::ResolverError,
    expression::{self, Expression, ExpressionVisitor, VariableToken},
    interpreter::Interpreter,
    statement::{
        self, FunctionDeclarationStatement, Statement, StatementVisitor,
    },
};

// in chapter 11, under section 11.4.1, we utilize the property of
// java map mapping every expression to a unique place
// in order to replicate that, I decide that we need to get the address of the
// expression
// this makes it so that if I use clone in my workflow, it will not be correct
// SCARY
pub trait Address {
    fn addr(&self) -> usize {
        let x = &raw const *self;
        x.addr()
    }
}
impl Address for expression::VariableExpression {}
impl Address for expression::AssignmentExpression {}

#[derive(Clone)]
enum FunctionKind {
    None,
    Function,
}

pub struct Resolver<'a> {
    scopes: Vec<HashMap<VariableToken, bool>>,
    interpreter: &'a mut Interpreter,
    current_function: FunctionKind,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            scopes: Vec::new(),
            interpreter,
            current_function: FunctionKind::None,
        }
    }
    pub fn resolve(
        &mut self,
        statements: &Vec<Statement>,
    ) -> Result<(), ResolverError> {
        for statement in statements {
            self.resolve_statement(statement)?;
        }
        Ok(())
    }
    fn resolve_statement(
        &mut self,
        statement: &Statement,
    ) -> Result<(), ResolverError> {
        statement.accept(self)
    }
    fn resolve_expression(
        &mut self,
        expression: &Expression,
    ) -> Result<(), ResolverError> {
        expression.accept(self)
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: VariableToken) -> Result<(), ResolverError> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name) {
                return Err(ResolverError::AlreadyDefined(name.to_string()));
            }
            scope.insert(name, false);
        }
        Ok(())
    }
    fn define(&mut self, name: VariableToken) {
        if let Some(scope) = self.scopes.last_mut() {
            // WARN: maybe we should throw if not found?
            scope.insert(name, true);
        }
    }

    fn get_from_current_scope(&self, name: &VariableToken) -> Option<bool> {
        self.scopes
            .last()
            .and_then(|scope| scope.get(name))
            .copied()
    }

    fn resolve_local<T: Address>(
        &mut self,
        expression: &T,
        name: &VariableToken,
    ) {
        for (num_hops, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name) {
                self.interpreter.resolve(expression.addr(), num_hops);
                break;
            }
        }
    }

    fn resolve_function(
        &mut self,
        function: &FunctionDeclarationStatement,
        kind: FunctionKind,
    ) -> Result<(), ResolverError> {
        let enclosing_function = mem::replace(&mut self.current_function, kind);
        self.begin_scope();
        for param in &function.parameters {
            self.declare(param.clone())?;
            self.define(param.clone());
        }
        // actually it was resolve, but I don't like cloning
        self.visit_block(&function.body)?;
        self.end_scope();
        self.current_function = enclosing_function;
        Ok(())
    }
}

impl ExpressionVisitor<Result<(), ResolverError>> for Resolver<'_> {
    fn visit_variable(
        &mut self,
        expression: &expression::VariableExpression,
    ) -> Result<(), ResolverError> {
        if !self.scopes.is_empty()
            && self.get_from_current_scope(&expression.variable) == Some(false)
        {
            return Err(ResolverError::UsageBeforeDeclaration(
                expression.variable.to_string(),
            ));
        }
        self.resolve_local(expression, &expression.variable);
        Ok(())
    }
    fn visit_assignment(
        &mut self,
        expression: &expression::AssignmentExpression,
    ) -> Result<(), ResolverError> {
        self.resolve_expression(&expression.expression)?;
        self.resolve_local(expression, &expression.variable);
        Ok(())
    }
    fn visit_binary(
        &mut self,
        expression: &expression::BinaryExpression,
    ) -> Result<(), ResolverError> {
        self.resolve_expression(&expression.left_expression)?;
        self.resolve_expression(&expression.right_expression)?;
        Ok(())
    }
    fn visit_call(
        &mut self,
        expression: &expression::CallExpression,
    ) -> Result<(), ResolverError> {
        self.resolve_expression(&expression.callee)?;
        for argument in expression.arguments.iter() {
            self.resolve_expression(argument)?;
        }
        Ok(())
    }
    fn visit_grouping(
        &mut self,
        expression: &expression::GroupingExpression,
    ) -> Result<(), ResolverError> {
        self.resolve_expression(&expression.expression)
    }
    fn visit_literal(
        &self,
        _expression: &expression::LiteralExpression,
    ) -> Result<(), ResolverError> {
        Ok(())
    }
    fn visit_logical(
        &mut self,
        expression: &expression::LogicalExpression,
    ) -> Result<(), ResolverError> {
        self.resolve_expression(&expression.left_expression)?;
        self.resolve_expression(&expression.right_expression)?;
        Ok(())
    }
    fn visit_unary(
        &mut self,
        expression: &expression::UnaryExpression,
    ) -> Result<(), ResolverError> {
        self.resolve_expression(&expression.expression)
    }
}

impl StatementVisitor<Result<(), ResolverError>> for Resolver<'_> {
    fn visit_block(
        &mut self,
        statement: &statement::BlockStatement,
    ) -> Result<(), ResolverError> {
        self.begin_scope();
        for statement in &statement.statements {
            self.resolve_statement(&statement)?;
        }
        self.end_scope();
        Ok(())
    }
    fn visit_variable_declaration(
        &mut self,
        statement: &statement::VariableDeclarationStatement,
    ) -> Result<(), ResolverError> {
        self.declare(statement.name.clone())?;
        if let Some(initializer) = &statement.expression {
            self.resolve_expression(initializer)?;
        }
        self.define(statement.name.clone());
        Ok(())
    }
    fn visit_function(
        &mut self,
        statement: &statement::FunctionDeclarationStatement,
    ) -> Result<(), ResolverError> {
        self.declare(statement.name.clone())?;
        self.define(statement.name.clone());
        self.resolve_function(statement, FunctionKind::Function)?;
        Ok(())
    }
    fn visit_expression(
        &mut self,
        statement: &statement::ExpressionStatement,
    ) -> Result<(), ResolverError> {
        self.resolve_expression(&statement.expression)
    }
    fn visit_if(
        &mut self,
        statement: &statement::IfStatement,
    ) -> Result<(), ResolverError> {
        self.resolve_expression(&statement.condition)?;
        self.resolve_statement(&statement.then_branch)?;
        if let Some(else_branch) = &statement.else_branch {
            self.resolve_statement(else_branch)?;
        }
        Ok(())
    }
    fn visit_print(
        &mut self,
        statement: &statement::PrintStatement,
    ) -> Result<(), ResolverError> {
        self.resolve_expression(&statement.expression)
    }
    fn visit_return(
        &mut self,
        statement: &statement::ReturnStatement,
    ) -> Result<(), ResolverError> {
        if matches!(self.current_function, FunctionKind::None) {
            return Err(ResolverError::TopLevelReturn);
        }
        if let Some(expression) = &statement.value {
            self.resolve_expression(expression)?;
        }
        Ok(())
    }
    fn visit_while(
        &mut self,
        statement: &statement::WhileStatement,
    ) -> Result<(), ResolverError> {
        self.resolve_expression(&statement.condition)?;
        self.resolve_statement(&statement.body)
    }
}
