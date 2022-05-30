// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::collections::LinkedList;
use std::rc::Rc;

use nixel::ast::StringPart;

use super::runtime_stack_frame::RuntimeStackFrame;
use crate::interpreter::error::Error;
use crate::interpreter::location::Location;
use crate::interpreter::scope::ScopeKind;
use crate::interpreter::value::Value;



macro_rules! build_math_fn {
    ($name:tt, $method:tt, $action:literal, $symbol:literal) => {
        fn $name(
            &mut self,
            mut args: Vec<Rc<Value>>,
            location: &Location,
        ) -> Result<Rc<Value>, Error> {
            let rhs = args.remove(1);
            let lhs = args.remove(0);

            let lhs = self.advance_monotonically(lhs)?;
            let rhs = self.advance_monotonically(rhs)?;

            let action = $action;
            let symbol = $symbol;

            match (&*lhs, &*rhs) {
                (Value::Int(lhs_value), Value::Int(rhs_value)) => {
                    //match lhs_value.checked_add(*rhs_value) {
                    //match lhs_value.checked_mul(*rhs_value) {
                    match lhs_value.$method(*rhs_value) {
                        Some(value) => Ok(Rc::new(Value::Int(value))),
                        None => Err(Error::Interpreter {
                            description: format!(
                                "integer overflow while {action} {lhs_value} and \
                                {rhs_value}"
                            ),
                            location:    location.clone(),
                            stack:       self.stack.clone(),
                        }),
                    }
                }
                _ => Err(Error::Interpreter {
                    description: format!(
                        "built-in {symbol} is not implemented for operands of type {:?} \
                        and {:?}",
                        lhs.kind(),
                        rhs.kind(),
                    ),
                    location:    location.clone(),
                    stack:       self.stack.clone(),
                }),
            }

        }
    };
}



#[derive(Debug)]
pub(crate) struct Runtime {
    pub(crate) stack: LinkedList<RuntimeStackFrame>,
}

impl Runtime {
    pub(crate) fn new() -> Runtime {
        Runtime { stack: LinkedList::new() }
    }

    fn add_stack_frame(&mut self, description: String, location: Location) {
        let stack_frame = RuntimeStackFrame { description, location };
        log::trace!("stack += {stack_frame}");
        self.stack.push_back(stack_frame);
    }

    pub(crate) fn advance(
        &mut self,
        value: &Rc<Value>,
    ) -> Result<Rc<Value>, Error> {
        let value = match &**value {
            Value::DeferredValue { ast, path, scope } => {
                Ok(Rc::new(Value::from_ast(path.clone(), ast.clone(), scope)))
            }

            Value::FunctionApplication {
                argument_index,
                arguments,
                function,
                location,
            } => {
                let function = self.advance_monotonically(function.clone())?;

                match &*function {
                    Value::BuiltInFunction {
                        expected_arguments,
                        identifier,
                    } => {
                        self.add_stack_frame(
                            format!("evaluating {identifier:?}"),
                            location.clone(),
                        );

                        if *expected_arguments == arguments.len() {
                            let function = match identifier.as_str() {
                                "built-in +" => Runtime::built_in_addition,
                                "built-in -" => Runtime::built_in_subtraction,
                                "built-in *" => Runtime::built_in_multiplication,
                                //"built-in ++" => Runtime::built_in_concat,
                                _ => unreachable!(),
                            };

                            function(self, arguments.clone(), location)
                        } else {
                            Ok(value.clone())
                        }
                    }
                    Value::Function {
                        bind_to,
                        implementation,
                        path,
                        scope,
                        ..
                    } => {
                        let arguments = arguments.clone();
                        let argument = arguments[*argument_index].clone();
                        let argument = self.advance_monotonically(argument)?;
                        self.add_stack_frame(
                            format!(
                                "calling a {:?} with argument #{} of kind {:?}",
                                function.kind(),
                                argument_index + 1,
                                argument.kind(),
                            ),
                            location.clone(),
                        );

                        let scope = scope.derive(ScopeKind::Plain);

                        if let Some(bind_to) = bind_to {
                            scope.bind(bind_to.clone(), argument);
                        }

                        let value = Value::DeferredValue {
                            ast:   implementation.clone(),
                            path:  path.clone(),
                            scope: scope.clone(),
                        };
                        let value = if argument_index + 1 == arguments.len() {
                            value
                        } else {
                            Value::FunctionApplication {
                                argument_index: argument_index + 1,
                                arguments,
                                function: Rc::new(value),
                                location: location.clone(),
                            }
                        };
                        let value = Rc::new(value);
                        let value = self.advance_monotonically(value)?;

                        Ok(value)
                    }
                    _ => Err(Error::Interpreter {
                        description: format!(
                            "attempt to call something which is not a function but a {:?}",
                            function.kind(),
                        ),
                        location:    location.clone(),
                        stack:       self.stack.clone(),
                    }),
                }
            }

            Value::Variable { identifier, location, scope } => {
                match scope.lookup(identifier) {
                    Some(value) => Ok(value),
                    None => match identifier.as_str() {
                        "built-in +" => {
                            Ok(Rc::new(Value::FunctionApplication {
                                argument_index: 0,
                                arguments:      Vec::new(),
                                function:       Rc::new(
                                    Value::BuiltInFunction {
                                        expected_arguments: 2,
                                        identifier:         identifier.clone(),
                                    },
                                ),
                                location:       location.clone(),
                            }))
                        }
                        "false" => Ok(Rc::new(Value::Boolean(false))),
                        "true" => Ok(Rc::new(Value::Boolean(true))),
                        _ => Err(Error::Interpreter {
                            description: format!(
                                "undefined variable {identifier:?}"
                            ),
                            location:    location.clone(),
                            stack:       self.stack.clone(),
                        }),
                    },
                }
            }

            _ => Ok(value.clone()),
        };

        value
    }

    pub(crate) fn advance_monotonically(
        &mut self,
        mut value: Rc<Value>,
    ) -> Result<Rc<Value>, Error> {
        let mut old_value;

        loop {
            old_value = value;

            value = self.advance(&old_value)?;

            if Rc::ptr_eq(&value, &old_value) {
                break;
            }
        }

        Ok(value)
    }

    build_math_fn!(
        built_in_addition,
        checked_add,
        "adding",
        "+"
    );

    build_math_fn!(
        built_in_subtraction,
        checked_sub,
        "subtracting",
        "-"
    );

    build_math_fn!(
        built_in_multiplication,
        checked_mul,
        "multiplying",
        "*"
    );

    fn built_in_concat(
        &mut self,
        mut args: Vec<Rc<Value>>,
        location: &Location,
    ) -> Result<Rc<Value>, Error> {
        let rhs = args.remove(1);
        let lhs = args.remove(0);

        let lhs = self.advance_monotonically(lhs)?;
        let rhs = self.advance_monotonically(rhs)?;

        match (&*lhs, &*rhs) {
            (Value::String { parts: lhs_value }, Value::String { parts: rhs_value }) => {
                let mut new_value = LinkedList::<StringPart>::new();
                for val in lhs_value {
                    new_value.push_back(*val);
                    // error[E0507]: cannot move out of `*val` which is behind a shared reference
                    // move occurs because `*val` has type `StringPart`, which does not implement the `Copy` trait
                }
                for val in rhs_value {
                    new_value.push_back(*val);
                }
                Ok(Rc::new(Value::String { parts: new_value }))
            }
            // TODO list + list
            _ => Err(Error::Interpreter {
                description: format!(
                    "built-in + is not implemented for operands of type {:?} \
                    and {:?}",
                    lhs.kind(),
                    rhs.kind(),
                ),
                location:    location.clone(),
                stack:       self.stack.clone(),
            }),
        }
    }
}
