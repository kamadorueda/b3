// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::collections::LinkedList;
use std::rc::Rc;

use super::runtime_stack_frame::RuntimeStackFrame;
use crate::interpreter::error::Error;
use crate::interpreter::location::Location;
use crate::interpreter::scope::ScopeKind;
use crate::interpreter::value::Value;

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
                        let value = if argument_index < &arguments.len() {
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
                            "calling a {:?} is not currently possible",
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

        log::trace!("value = {:#?}", &value);

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

    fn built_in_addition(
        &mut self,
        mut args: Vec<Rc<Value>>,
        location: &Location,
    ) -> Result<Rc<Value>, Error> {
        let rhs = args.remove(1);
        let lhs = args.remove(0);

        let lhs = self.advance_monotonically(lhs)?;
        let rhs = self.advance_monotonically(rhs)?;

        match (&*lhs, &*rhs) {
            (Value::Int(lhs_value), Value::Int(rhs_value)) => {
                match lhs_value.checked_add(*rhs_value) {
                    Some(value) => Ok(Rc::new(Value::Int(value))),
                    None => Err(Error::Interpreter {
                        description: format!(
                            "integer overflow while adding {lhs_value} and \
                             {rhs_value}"
                        ),
                        location:    location.clone(),
                        stack:       self.stack.clone(),
                    }),
                }
            }
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
