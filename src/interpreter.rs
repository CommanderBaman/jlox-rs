use std::{cell::RefCell, collections::HashMap, rc::Rc, time::SystemTimeError};
use thiserror::Error;

use crate::{
    class::LoxClass,
    environment::Environment,
    expression::{
        AssignmentExpression, BinaryExpression, CallExpression, Expression,
        ExpressionVisitor, GetExpression, GroupingExpression,
        LiteralExpression, LiteralToken, LogicalExpression, LogicalOperator,
        SetExpression, SuperExpression, ThisExpression, UnaryExpression,
        VariableExpression, VariableToken,
    },
    function::{LoxCallable, LoxFunction},
    resolver::Address,
    statement::{
        BlockStatement, ClassDeclarationStatement, ExpressionStatement,
        FunctionDeclarationStatement, IfStatement, PrintStatement,
        ReturnStatement, Statement, StatementVisitor,
        VariableDeclarationStatement, WhileStatement,
    },
    value::{Value, ValueError},
};

impl Value {
    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        mut arguments: Vec<Self>,
    ) -> Result<Self, RuntimeError> {
        match self {
            Self::Call(c) => c.call(interpreter, arguments),
            Self::ClassDeclaration(c) => {
                // add reference to self in the end because
                // we need a reference to main class in instance
                arguments.push(self.clone());
                // NOTE: we do not need to push an environment here
                // because I build the instance on top of the
                // class environment
                // but we need it when we call the init
                let value = c.call(interpreter, arguments.clone())?;
                match &value {
                    Self::ClassInstance(i) => {
                        arguments.pop();
                        i.bind(value.clone());
                        interpreter.push_environment();
                        let e = i.init(interpreter, arguments);
                        interpreter.pop_environment()?;
                        e?;
                    }
                    _ => unreachable!(
                        "call to class declaration should produce its instance"
                    ),
                }
                Ok(value)
            }
            _ => Err(RuntimeError::NotCallable(self.to_string())),
        }
    }
    fn environment(&self) -> InterpreterEnvironment {
        match self {
            Self::ClassInstance(c) => c.environment(),
            e => unreachable!("you must not call environment on {e}"),
        }
    }
}

pub type InterpreterEnvironment = Rc<RefCell<Environment>>;

pub struct Interpreter {
    pub environment: InterpreterEnvironment,
    print_expression: bool,
    locals: HashMap<usize, usize>,
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("error operating on values of expression {expression}")]
    ValueError {
        expression: Expression,
        source: ValueError,
    },
    #[error("variable {0} not found in environment")]
    VariableNotFound(String),
    #[error("code tried to remove root environment")]
    RootEnvironmentRemoved,
    #[error("value is not callable: {0}")]
    NotCallable(String),
    #[error(
        "incorrect number of arguments received for {callee_name}. Expected {expected_arguments}, Received {received_arguments}"
    )]
    IncorrectNumberOfArguments {
        callee_name: String,
        expected_arguments: usize,
        received_arguments: usize,
    },
    #[error("time error: {0}")]
    IllegalTime(#[from] SystemTimeError),
    #[error("return called from somewhere unknown")]
    Return(Value),
    #[error("no properties on types other than instance: {0}")]
    NoProperties(Value),
    #[error("no property named {0} on instance: {1}")]
    PropertyNotFound(VariableToken, String),
    #[error("can not set properties on instance: {0}")]
    NotSettable(String),
    #[error("environment not found")]
    EnvironmentNotFound,
    #[error("expected class value for superclass for class {0}")]
    InvalidSuperClass(VariableToken),
    #[error("invalid super state: {0}")]
    InvalidSuperState(String),
}

impl Interpreter {
    pub fn new(print_expression: bool) -> Self {
        Interpreter {
            environment: Rc::new(RefCell::new(Environment::new())),
            print_expression,
            locals: HashMap::new(),
        }
    }
    pub fn evaluate(
        &mut self,
        expression: &Expression,
    ) -> Result<Value, RuntimeError> {
        expression.accept(self)
    }
    pub fn interpret(
        &mut self,
        statements: &Vec<Rc<Statement>>,
    ) -> Result<(), RuntimeError> {
        let mut last_value = Value::Nil;
        for statement in statements {
            last_value = self.execute(statement.clone())?;
        }
        // print last value if expression
        if self.print_expression
            && let Some(statement) = statements.last()
            && let Statement::Expression(e) = &**statement
            && !matches!(
                e.expression,
                Expression::Assignment(_) | Expression::Call(_),
            )
        {
            println!("{last_value}");
        }
        Ok(())
    }
    fn execute(
        &mut self,
        statement: Rc<Statement>,
    ) -> Result<Value, RuntimeError> {
        Statement::accept(statement, self)
    }
    pub fn resolve(&mut self, location: usize, depth: usize) {
        self.locals.insert(location, depth);
    }

    pub fn assign_environment(
        &mut self,
        environment: InterpreterEnvironment,
    ) -> InterpreterEnvironment {
        let old_environment = self.environment.clone();
        self.environment = environment;
        old_environment
    }
    pub fn push_environment(&mut self) {
        self.environment =
            Rc::new(RefCell::new(Environment::child(self.environment.clone())));
    }
    pub fn pop_environment(&mut self) -> Result<(), RuntimeError> {
        let parent = self.environment.borrow().parent.clone();
        match parent {
            None => Err(RuntimeError::RootEnvironmentRemoved),
            Some(p) => {
                self.environment = p;
                Ok(())
            }
        }
    }
}

impl ExpressionVisitor<Result<Value, RuntimeError>> for Interpreter {
    fn visit_literal(
        &self,
        expression: &LiteralExpression,
    ) -> Result<Value, RuntimeError> {
        match expression.literal {
            LiteralToken::Nil => Ok(Value::Nil),
            LiteralToken::Number(n) => Ok(Value::Number(n)),
            LiteralToken::String(ref s) => Ok(Value::String(s.to_owned())),
            LiteralToken::True => Ok(Value::Bool(true)),
            LiteralToken::False => Ok(Value::Bool(false)),
        }
    }
    fn visit_grouping(
        &mut self,
        expression: &GroupingExpression,
    ) -> Result<Value, RuntimeError> {
        self.evaluate(expression.expression.as_ref())
    }
    fn visit_unary(
        &mut self,
        expression: &UnaryExpression,
    ) -> Result<Value, RuntimeError> {
        let value = self.evaluate(&expression.expression)?;
        value.do_unary_operation(expression.operator).map_err(|e| {
            RuntimeError::ValueError {
                expression: expression.clone().wrap(),
                source: e,
            }
        })
    }
    fn visit_binary(
        &mut self,
        expression: &BinaryExpression,
    ) -> Result<Value, RuntimeError> {
        let left_value = self.evaluate(&expression.left_expression)?;
        let right_value = self.evaluate(&expression.right_expression)?;
        left_value
            .do_binary_operation(right_value, expression.operator)
            .map_err(|e| RuntimeError::ValueError {
                expression: expression.clone().wrap(),
                source: e,
            })
    }
    fn visit_variable(
        &mut self,
        expression: &VariableExpression,
    ) -> Result<Value, RuntimeError> {
        let addr = expression.addr();
        self.environment
            .borrow()
            .lookup(&expression.variable, self.locals.get(&addr).cloned())
            .ok_or_else(|| {
                RuntimeError::VariableNotFound(expression.variable.to_string())
            })
    }
    fn visit_assignment(
        &mut self,
        expression: &AssignmentExpression,
    ) -> Result<Value, RuntimeError> {
        let value = self.evaluate(&expression.expression)?;
        self.environment
            .borrow_mut()
            .assign(&expression.variable, value.clone())?;
        Ok(value)
    }
    fn visit_logical(
        &mut self,
        expression: &LogicalExpression,
    ) -> Result<Value, RuntimeError> {
        let left_value = self.evaluate(&expression.left_expression)?;
        match expression.operator {
            LogicalOperator::Or => {
                if left_value.is_truthy() {
                    return Ok(left_value);
                }
            }
            LogicalOperator::And => {
                if !left_value.is_truthy() {
                    return Ok(left_value);
                }
            }
        }
        self.evaluate(&expression.right_expression)
    }
    fn visit_call(
        &mut self,
        expression: &CallExpression,
    ) -> Result<Value, RuntimeError> {
        let callee = self.evaluate(&expression.callee)?;
        let arguments: Vec<Value> = expression
            .arguments
            .iter()
            .map(|arg| self.evaluate(arg))
            .collect::<Result<Vec<Value>, RuntimeError>>()?;
        callee.call(self, arguments)
    }
    fn visit_get(
        &mut self,
        expression: &GetExpression,
    ) -> Result<Value, RuntimeError> {
        let callee = self.evaluate(&expression.callee)?;
        let Value::ClassInstance(object) = &callee else {
            return Err(RuntimeError::NoProperties(callee));
        };
        object.get(&expression.property)
    }
    fn visit_set(
        &mut self,
        expression: &SetExpression,
    ) -> Result<Value, RuntimeError> {
        let callee = self.evaluate(&expression.callee)?;
        let Value::ClassInstance(object) = callee else {
            return Err(RuntimeError::NotSettable(callee.to_string()));
        };
        let value = self.evaluate(&expression.value)?;
        object.set(expression.property.clone(), value.clone());
        Ok(value)
    }
    fn visit_this(
        &mut self,
        expression: &ThisExpression,
    ) -> Result<Value, RuntimeError> {
        let addr = expression.addr();
        self.environment
            .borrow()
            .lookup(&expression.token, self.locals.get(&addr).cloned())
            .ok_or_else(|| {
                RuntimeError::VariableNotFound(expression.token.to_string())
            })
    }
    fn visit_super(
        &mut self,
        expression: &SuperExpression,
    ) -> Result<Value, RuntimeError> {
        let addr = expression.addr();
        let distance = self.locals.get(&addr).expect(
            "super to be defined in environment when visiting super expression",
        ).to_owned();
        let Value::ClassDeclaration(superclass) = self
            .environment
            .borrow()
            .lookup(&VariableToken::Super, Some(distance))
            .ok_or(RuntimeError::InvalidSuperState(
                "value of super to be defined when visiting super expression "
                    .to_string(),
            ))?
        else {
            return Err(RuntimeError::InvalidSuperState(
                "super in environment is not a class".to_string(),
            ));
        };
        let this_object = self
            .environment
            .borrow()
            .lookup(&VariableToken::This, Some(distance - 1))
            .ok_or(RuntimeError::InvalidSuperState(
                "value of super's this to be defined when visiting super expression".to_string(),
            ))?;
        let method = superclass.get_method(&expression.method).ok_or(
            RuntimeError::InvalidSuperState(format!(
                "method on super ({}) is not defined {}",
                superclass, expression.method
            )),
        )?;
        Ok(Value::Call(LoxFunction::new(
            method.clone(),
            this_object.environment(),
            method
                .name
                .eq(&VariableToken::Identifier("init".to_string())),
        )))
    }
}

impl StatementVisitor<Result<Value, RuntimeError>> for Interpreter {
    fn visit_print(
        &mut self,
        statement: Rc<PrintStatement>,
    ) -> Result<Value, RuntimeError> {
        let value = self.evaluate(&statement.expression)?;
        println!("{value}");
        Ok(value)
    }
    fn visit_expression(
        &mut self,
        statement: Rc<ExpressionStatement>,
    ) -> Result<Value, RuntimeError> {
        self.evaluate(&statement.expression)
    }
    fn visit_variable_declaration(
        &mut self,
        statement: Rc<VariableDeclarationStatement>,
    ) -> Result<Value, RuntimeError> {
        let value = statement
            .expression
            .as_ref()
            .map(|ex| self.evaluate(ex))
            .transpose()?
            .unwrap_or(Value::Uninitialized);
        self.environment
            .borrow_mut()
            .define(statement.name.clone(), value.clone());
        Ok(value)
    }
    fn visit_block(
        &mut self,
        statement: Rc<BlockStatement>,
    ) -> Result<Value, RuntimeError> {
        // self.environment.borrow_mut().push_child();
        self.push_environment();
        for statement in &statement.statements {
            match self.execute(statement.clone()) {
                Ok(_) => {}
                Err(e) => {
                    // how much I wish there was a defer
                    self.pop_environment()?;
                    return Err(e);
                }
            }
        }
        // self.environment.borrow_mut().pop_child()?;
        self.pop_environment()?;
        Ok(Value::Nil)
    }
    fn visit_if(
        &mut self,
        statement: Rc<IfStatement>,
    ) -> Result<Value, RuntimeError> {
        if self.evaluate(&statement.condition)?.is_truthy() {
            self.execute(statement.then_branch.clone())
        } else if let Some(ref else_branch) = statement.else_branch {
            self.execute(else_branch.clone())
        } else {
            Ok(Value::Nil)
        }
    }
    fn visit_while(
        &mut self,
        statement: Rc<WhileStatement>,
    ) -> Result<Value, RuntimeError> {
        while self.evaluate(&statement.condition)?.is_truthy() {
            self.execute(statement.body.clone())?;
        }
        Ok(Value::Nil)
    }
    fn visit_function(
        &mut self,
        statement: Rc<FunctionDeclarationStatement>,
    ) -> Result<Value, RuntimeError> {
        self.environment.borrow_mut().define(
            statement.name.clone(),
            Value::Call(LoxFunction::new(
                statement.clone(),
                self.environment.clone(),
                false,
            )),
        );
        Ok(Value::Nil)
    }
    fn visit_return(
        &mut self,
        statement: Rc<ReturnStatement>,
    ) -> Result<Value, RuntimeError> {
        let value = statement
            .value
            .as_ref()
            .map(|expr| self.evaluate(expr))
            .transpose()?
            .unwrap_or(Value::Nil);
        Err(RuntimeError::Return(value))
    }
    fn visit_class(
        &mut self,
        statement: Rc<ClassDeclarationStatement>,
    ) -> Result<Value, RuntimeError> {
        self.environment
            .borrow_mut()
            .define(statement.name.clone(), Value::Nil);
        let mut superclass: Option<Rc<LoxClass>> = None;
        if let Some(superclass_expression) = statement.superclass.as_ref() {
            let value = self.visit_variable(superclass_expression)?;
            match &value {
                Value::ClassDeclaration(s) => superclass = Some(s.clone()),
                _ => {
                    return Err(RuntimeError::InvalidSuperClass(
                        statement.name.clone(),
                    ));
                }
            }
            self.push_environment();
            self.environment
                .borrow_mut()
                .define(VariableToken::Super, value);
        }
        let superclass_defined = superclass.is_some();
        let class = Value::ClassDeclaration(Rc::new(LoxClass::new(
            statement.clone(),
            self.environment.clone(),
            superclass,
        )));
        if superclass_defined {
            self.pop_environment()?;
        }
        self.environment
            .borrow_mut()
            .assign(&statement.name, class)?;
        Ok(Value::Nil)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn value_is_truthy() {
        let cases = vec![
            (Value::Nil, false),
            (Value::Bool(false), false),
            (Value::Bool(true), true),
            (Value::Number(0.0), true),
            (Value::Number(1.0), true),
            (Value::Number(-1.0), true),
            (Value::Number(f64::NAN), true),
            (Value::String("lkj".to_owned()), true),
        ];

        for (value, expected) in cases {
            assert_eq!(value.is_truthy(), expected)
        }
    }
}
