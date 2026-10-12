use std::{collections::HashMap, mem, rc::Rc};

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

enum VariableState {
    Define,
    Declare,
    Used,
}

impl VariableState {
    fn declare(
        stored: Option<&Self>,
        name: VariableToken,
    ) -> Result<Self, ResolverError> {
        match stored {
            None => Ok(VariableState::Declare),
            _ => Err(ResolverError::AlreadyDeclared(name)),
        }
    }
    fn define(&self, name: VariableToken) -> Result<Self, ResolverError> {
        match self {
            VariableState::Declare => Ok(VariableState::Define),
            _ => Err(ResolverError::AlreadyDefined(name)),
        }
    }
    fn consume(&self, name: VariableToken) -> Result<Self, ResolverError> {
        match self {
            VariableState::Define => Ok(VariableState::Used),
            VariableState::Used => Ok(VariableState::Used),
            VariableState::Declare => {
                Err(ResolverError::UsageBeforeDefinition(name))
            }
        }
    }

    fn can_use(&self) -> bool {
        match self {
            VariableState::Define | VariableState::Used => true,
            VariableState::Declare => false,
        }
    }
}

pub struct Resolver<'a> {
    scopes: Vec<HashMap<VariableToken, VariableState>>,
    resolves: HashMap<usize, usize>,
    interpreter: &'a mut Interpreter,
    current_function: FunctionKind,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            scopes: Vec::new(),
            resolves: HashMap::new(),
            interpreter,
            current_function: FunctionKind::None,
        }
    }
    pub fn resolve(
        &mut self,
        statements: &Vec<Rc<Statement>>,
    ) -> Result<(), ResolverError> {
        for statement in statements {
            self.resolve_statement(statement.clone())?;
        }
        Ok(())
    }
    fn resolve_statement(
        &mut self,
        statement: Rc<Statement>,
    ) -> Result<(), ResolverError> {
        Statement::accept(statement, self)
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
    fn end_scope(&mut self) -> Result<(), ResolverError> {
        if let Some(scope) = self.scopes.pop() {
            let unused_elements: Vec<VariableToken> = scope
                .into_iter()
                // could use filter map but this is better
                .filter(|(_, state)| !matches!(state, VariableState::Used))
                .map(|(variable, _)| variable)
                .collect();
            if !unused_elements.is_empty() {
                return Err(ResolverError::UnusedVariables(unused_elements));
            }
        }
        Ok(())
    }

    fn declare(&mut self, name: VariableToken) -> Result<(), ResolverError> {
        if let Some(scope) = self.scopes.last_mut() {
            let state = VariableState::declare(scope.get(&name), name.clone())?;
            scope.insert(name, state);
        }
        Ok(())
    }
    fn define(&mut self, name: VariableToken) -> Result<(), ResolverError> {
        if let Some(scope) = self.scopes.last_mut() {
            let state = scope
                .get(&name)
                .ok_or(ResolverError::NotDeclared(name.clone()))
                .and_then(|var| var.define(name.clone()))?;
            scope.insert(name, state);
        }
        Ok(())
    }
    fn consume(
        &mut self,
        expression: &expression::VariableExpression,
    ) -> Result<(), ResolverError> {
        let Some(num_hops) = self.resolves.get(&expression.addr()) else {
            // global variable
            return Ok(());
        };
        // FIX: silent panic here because of negation of usize
        // should be safe though
        let scope_index = self.scopes.len() - num_hops - 1;
        if let Some(scope) = self.scopes.get_mut(scope_index) {
            let name = expression.variable.clone();
            let state = scope
                .get(&name)
                .ok_or(ResolverError::NotDeclared(name.clone()))
                .and_then(|var| var.consume(name.clone()))?;
            scope.insert(name, state);
        }
        Ok(())
    }

    fn get_from_current_scope(
        &self,
        name: &VariableToken,
    ) -> Option<&VariableState> {
        self.scopes.last().and_then(|scope| scope.get(name))
    }

    fn resolve_local<T: Address>(
        &mut self,
        expression: &T,
        name: &VariableToken,
    ) {
        for (num_hops, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name) {
                self.interpreter.resolve(expression.addr(), num_hops);
                self.resolves.insert(expression.addr(), num_hops);
                break;
            }
        }
    }

    fn resolve_function(
        &mut self,
        function: Rc<FunctionDeclarationStatement>,
        kind: FunctionKind,
    ) -> Result<(), ResolverError> {
        let enclosing_function = mem::replace(&mut self.current_function, kind);
        self.begin_scope();
        for param in &function.parameters {
            self.declare(param.clone())?;
            self.define(param.clone())?;
        }
        // actually it was resolve, but I don't like cloning
        // -> this aged like milk
        self.visit_block(function.body.clone())?;
        self.end_scope()?;
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
            && self
                .get_from_current_scope(&expression.variable)
                .is_some_and(|state| !state.can_use())
        {
            return Err(ResolverError::UsageBeforeDeclaration(
                expression.variable.clone(),
            ));
        }
        self.resolve_local(expression, &expression.variable);
        self.consume(expression)?;
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
        statement: Rc<statement::BlockStatement>,
    ) -> Result<(), ResolverError> {
        self.begin_scope();
        for statement in &statement.statements {
            self.resolve_statement(statement.clone())?;
        }
        self.end_scope()?;
        Ok(())
    }
    fn visit_variable_declaration(
        &mut self,
        statement: Rc<statement::VariableDeclarationStatement>,
    ) -> Result<(), ResolverError> {
        self.declare(statement.name.clone())?;
        if let Some(initializer) = &statement.expression {
            self.resolve_expression(initializer)?;
        }
        self.define(statement.name.clone())?;
        Ok(())
    }
    fn visit_function(
        &mut self,
        statement: Rc<statement::FunctionDeclarationStatement>,
    ) -> Result<(), ResolverError> {
        self.declare(statement.name.clone())?;
        self.define(statement.name.clone())?;
        self.resolve_function(statement, FunctionKind::Function)?;
        Ok(())
    }
    fn visit_expression(
        &mut self,
        statement: Rc<statement::ExpressionStatement>,
    ) -> Result<(), ResolverError> {
        self.resolve_expression(&statement.expression)
    }
    fn visit_if(
        &mut self,
        statement: Rc<statement::IfStatement>,
    ) -> Result<(), ResolverError> {
        self.resolve_expression(&statement.condition)?;
        self.resolve_statement(statement.then_branch.clone())?;
        if let Some(else_branch) = &statement.else_branch {
            self.resolve_statement(else_branch.clone())?;
        }
        Ok(())
    }
    fn visit_print(
        &mut self,
        statement: Rc<statement::PrintStatement>,
    ) -> Result<(), ResolverError> {
        self.resolve_expression(&statement.expression)
    }
    fn visit_return(
        &mut self,
        statement: Rc<statement::ReturnStatement>,
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
        statement: Rc<statement::WhileStatement>,
    ) -> Result<(), ResolverError> {
        self.resolve_expression(&statement.condition)?;
        self.resolve_statement(statement.body.clone())
    }
}
