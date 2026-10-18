use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    error::RuntimeError,
    expression::VariableToken,
    function::{LoxCallable, LoxFunction},
    interpreter::{Interpreter, InterpreterEnvironment, Value},
    statement::ClassDeclarationStatement,
};

#[derive(Debug)]
pub struct LoxClass {
    name: VariableToken,
    methods: HashMap<VariableToken, Rc<LoxFunction>>,
    superclass: Option<Rc<LoxClass>>,
}
impl LoxClass {
    pub fn new(
        declaration: Rc<ClassDeclarationStatement>,
        environment: InterpreterEnvironment,
        superclass: Option<Rc<LoxClass>>,
    ) -> Self {
        let mut methods = HashMap::new();
        for method in &declaration.functions {
            methods.insert(
                method.name.clone(),
                LoxFunction::new(
                    method.clone(),
                    environment.clone(),
                    method
                        .name
                        .eq(&VariableToken::Identifier("init".to_string())),
                ),
            );
        }
        Self {
            name: declaration.name.clone(),
            methods,
            superclass,
        }
    }
    fn get_method(&self, name: &VariableToken) -> Option<Rc<LoxFunction>> {
        self.methods.get(name).cloned().or_else(|| {
            self.superclass.as_ref().and_then(|c| c.get_method(name))
        })
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} class", self.name)
    }
}

impl LoxCallable for LoxClass {
    fn arity(&self) -> usize {
        self.methods
            .get(&VariableToken::Identifier("init".to_string()))
            .map(|f| f.arity())
            .unwrap_or(0)
    }
    fn call(
        &self,
        _interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        // extra 1 because of reference to self at last
        if self.arity() + 1 != arguments.len() {
            return Err(RuntimeError::IncorrectNumberOfArguments {
                callee_name: self.name.to_string(),
                expected_arguments: self.arity(),
                received_arguments: arguments.len(),
            });
        }
        let Some(Value::ClassDeclaration(self_reference)) = arguments.last()
        else {
            unreachable!("last element of class call should be self reference");
        };
        Ok(Value::ClassInstance(Rc::new(LoxInstance::new(
            self_reference.clone(),
        ))))
    }
}

#[derive(Debug)]
pub struct LoxInstance {
    fields: RefCell<HashMap<VariableToken, Value>>,
    class: Rc<LoxClass>,
}

impl LoxInstance {
    pub fn new(class: Rc<LoxClass>) -> Self {
        Self {
            fields: RefCell::new(HashMap::new()),
            class,
        }
    }
    pub fn get(
        &self,
        property: &VariableToken,
        this_value: Value,
        environment: InterpreterEnvironment,
    ) -> Result<Value, RuntimeError> {
        self.fields
            .borrow()
            .get(property)
            .cloned()
            .or_else(|| {
                self.class.get_method(property).map(|f| {
                    environment
                        .borrow_mut()
                        .define(VariableToken::This, this_value);
                    Value::Call(f.using(environment))
                })
            })
            .ok_or(RuntimeError::PropertyNotFound(
                property.clone(),
                self.to_string(),
            ))
    }
    pub fn set(&self, property: VariableToken, value: Value) {
        self.fields.borrow_mut().insert(property, value);
    }
    pub fn init(
        &self,
        this_value: Value,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<(), RuntimeError> {
        match self.get(
            &VariableToken::Identifier("init".to_string()),
            this_value,
            interpreter.environment.clone(),
        ) {
            Ok(call) => {
                call.call(interpreter, arguments)?;
                Ok(())
            }
            Err(RuntimeError::PropertyNotFound(_, _)) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fields_string = self
            .fields
            .borrow()
            .iter()
            .map(|(variable, value)| format!("{variable} : {value} "))
            .collect::<Vec<String>>()
            .join(", ");
        write!(
            f,
            "{} instance with fields {{ {}}}",
            self.class.name, fields_string
        )
    }
}
