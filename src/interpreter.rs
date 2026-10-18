use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::HashMap,
    ops::{Add, Div, Mul, Sub},
    rc::Rc,
};

use crate::{
    class::{LoxClass, LoxInstance},
    environment::Environment,
    error::RuntimeError,
    expression::{
        AssignmentExpression, BinaryExpression, BinaryOperator, CallExpression,
        Expression, ExpressionVisitor, GetExpression, GroupingExpression,
        LiteralExpression, LiteralToken, LogicalExpression, LogicalOperator,
        SetExpression, SuperExpression, ThisExpression, UnaryExpression,
        UnaryOperator, VariableExpression, VariableToken,
    },
    function::{LoxCallable, LoxFunction},
    resolver::Address,
    statement::{
        BlockStatement, ClassDeclarationStatement, ExpressionStatement,
        FunctionDeclarationStatement, IfStatement, PrintStatement,
        ReturnStatement, Statement, StatementVisitor,
        VariableDeclarationStatement, WhileStatement,
    },
    token::Token,
};

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    String(String),
    Uninitialized,
    Call(Rc<dyn LoxCallable>),
    ClassDeclaration(Rc<LoxClass>),
    ClassInstance(Rc<LoxInstance>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Self::Bool(b) => format!("{b}"),
            Self::Nil => "nil".to_string(),
            Self::Number(n) => format!("{n}"),
            Self::String(s) => s.to_owned(),
            Self::Uninitialized => "uinitialized".to_string(),
            Self::Call(c) => format!("{c}"),
            Self::ClassDeclaration(c) => format!("{c}"),
            Self::ClassInstance(c) => format!("{c}"),
        };
        write!(f, "{}", string)
    }
}

impl Value {
    fn is_truthy(&self) -> bool {
        !matches!(self, Value::Uninitialized | Value::Nil | Value::Bool(false))
    }
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

// we have to specify the == here for each pair because
// (l, r) => l.eq(r) does not work
// it just loops back this function again, causing infinite recursion
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Bool(l), Value::Bool(r)) => l == r,
            (Value::Nil, Value::Nil) => true,
            (Value::Number(l), Value::Number(r)) => l == r,
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Uninitialized, Value::Uninitialized) => true,
            (Value::Call(l), Value::Call(r)) => Rc::ptr_eq(l, r),
            _ => false,
        }
    }
}

// we do not face the same problem as PartialEq because we use Ord for
// all the ordering that we do. Hence, I believe this is never called.
// I also checked this via lldb when I couldn't set a breakpoint in this
// block but could set it for Ord block
impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// research why can't we derive Eq?
// we can't derive eq because of f64 and later Rc
// f64 is not eq, because it is not reflexive
// NaN == NaN in rust comes out as false when it should be true
// when we manually implement it we are saying that we are okay with it
impl Eq for Value {}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Value::Bool(l), Value::Bool(r)) => l.cmp(r),
            // NOTE: this total_cmp is different in behavior from partial_cmp
            (Value::Number(l), Value::Number(r)) => l.total_cmp(r),
            (Value::String(l), Value::String(r)) => l.cmp(r),
            // NOTE: solution to challenge 1
            //
            // I can easily bring conversions between numbers and strings
            // and same for bool
            // (Value::Number(l), Value::String(r)) => l.to_string().cmp(r),
            // (Value::String(l), Value::Number(r)) => l.cmp(&r.to_string()),
            // But I would not introduce these
            // because it brings confusion in the language
            // ex, "02" < 1 because "0" < "1"
            // Python also gives us type error when we do int vs str comparison
            (Value::Nil, Value::Nil) => Ordering::Equal,
            _ => todo!("value order not implemented"),
        }
    }
}

impl TryFrom<Value> for LiteralExpression {
    type Error = RuntimeError;
    fn try_from(value: Value) -> Result<Self, RuntimeError> {
        match value {
            Value::String(s) => Ok(LiteralExpression {
                literal: LiteralToken::String(s),
            }),
            Value::Bool(b) => {
                let literal = if b {
                    LiteralToken::True
                } else {
                    LiteralToken::False
                };
                Ok(LiteralExpression { literal })
            }
            Value::Number(n) => Ok(LiteralExpression {
                literal: LiteralToken::Number(n),
            }),
            Value::Nil => Ok(LiteralExpression {
                literal: LiteralToken::Nil,
            }),
            Value::Uninitialized => {
                Err(RuntimeError::UninitializedVariableUsed("?".to_owned()))
            }
            Value::Call(_)
            | Value::ClassInstance(_)
            | Value::ClassDeclaration(_) => {
                todo!("value call and classes to literal expression")
            }
        }
    }
}

fn get_binary_expression(
    left: Value,
    right: Value,
    operator: Token,
) -> Expression {
    BinaryExpression::build(
        Expression::Literal(left.try_into().expect("left value to be initialized")),
        operator,
        Expression::Literal(right.try_into().expect("right value to be initialized")),
    )
    .expect("value to literal conversion failed or invalid operator passed for building binary expression for error")
}

impl Add for Value {
    type Output = Result<Value, RuntimeError>;
    fn add(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            // (Value::Number(l), Value::Number(r)) => {}
            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
            // NOTE: solution to challenge 2
            (Value::String(l), Value::String(r)) => {
                Ok(Value::String(format!("{l}{r}")))
            }
            (Value::String(l), Value::Number(r)) => {
                Ok(Value::String(format!("{l}{r}")))
            }
            (Value::Number(l), Value::String(r)) => {
                Ok(Value::String(format!("{l}{r}")))
            }
            (Value::Uninitialized, _) | (_, Value::Uninitialized) => {
                Err(RuntimeError::UninitializedVariableUsed("?".to_owned()))
            }
            _ => Err(RuntimeError::InvalidOperation {
                operation: "Add".to_owned(),
                expression: get_binary_expression(self, rhs, Token::Plus),
            }),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, RuntimeError>;
    fn sub(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
            (Value::Uninitialized, _) | (_, Value::Uninitialized) => {
                Err(RuntimeError::UninitializedVariableUsed("?".to_owned()))
            }
            _ => Err(RuntimeError::InvalidOperation {
                operation: "Sub".to_owned(),
                expression: get_binary_expression(self, rhs, Token::Minus),
            }),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value, RuntimeError>;
    fn mul(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
            (Value::Uninitialized, _) | (_, Value::Uninitialized) => {
                Err(RuntimeError::UninitializedVariableUsed("?".to_owned()))
            }
            _ => Err(RuntimeError::InvalidOperation {
                operation: "Mul".to_owned(),
                expression: get_binary_expression(self, rhs, Token::Star),
            }),
        }
    }
}

impl Div for Value {
    type Output = Result<Value, RuntimeError>;
    fn div(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Value::Number(l), Value::Number(r)) => {
                // NOTE: solution to challenge 3
                //
                // previously when divided by zero, due to rust internals,
                // the number becamse infinite
                // now we will give a runtime error
                if r.eq(&0.0) {
                    return Err(RuntimeError::DivisionByZero(
                        get_binary_expression(self, rhs, Token::Slash),
                    ));
                }
                Ok(Value::Number(l / r))
            }
            (Value::Uninitialized, _) | (_, Value::Uninitialized) => {
                Err(RuntimeError::UninitializedVariableUsed("?".to_owned()))
            }
            _ => Err(RuntimeError::InvalidOperation {
                operation: "Div".to_owned(),
                expression: get_binary_expression(self, rhs, Token::Slash),
            }),
        }
    }
}

pub type InterpreterEnvironment = Rc<RefCell<Environment>>;

pub struct Interpreter {
    pub environment: InterpreterEnvironment,
    print_expression: bool,
    locals: HashMap<usize, usize>,
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
        let mut value = self.evaluate(&expression.expression)?;
        value = match expression.operator {
            UnaryOperator::Bang => Value::Bool(!value.is_truthy()),
            UnaryOperator::Minus => {
                (Value::Number(-1.0) * value).map_err(|_| {
                    RuntimeError::InvalidOperation {
                        operation: UnaryOperator::Minus.to_string(),
                        expression: expression.clone().wrap(),
                    }
                })?
            }
        };
        Ok(value)
    }
    fn visit_binary(
        &mut self,
        expression: &BinaryExpression,
    ) -> Result<Value, RuntimeError> {
        let left_value = self.evaluate(&expression.left_expression)?;
        let right_value = self.evaluate(&expression.right_expression)?;

        match expression.operator {
            BinaryOperator::EqualEqual => {
                Ok(Value::Bool(left_value.eq(&right_value)))
            }
            BinaryOperator::BangEqual => {
                Ok(Value::Bool(left_value.ne(&right_value)))
            }
            BinaryOperator::Less => {
                Ok(Value::Bool(left_value.cmp(&right_value).is_lt()))
            }
            BinaryOperator::LessEqual => {
                Ok(Value::Bool(left_value.cmp(&right_value).is_le()))
            }
            BinaryOperator::Greater => {
                Ok(Value::Bool(left_value.cmp(&right_value).is_gt()))
            }
            BinaryOperator::GreaterEqual => {
                Ok(Value::Bool(left_value.cmp(&right_value).is_ge()))
            }
            BinaryOperator::Plus => left_value + right_value,
            BinaryOperator::Minus => left_value - right_value,
            BinaryOperator::Star => left_value * right_value,
            BinaryOperator::Slash => left_value / right_value,
        }
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
