// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::collections::hash_map::Entry;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::Context;

use crate::core::types::expression::Expression;
use crate::core::types::function::Function;
use crate::core::types::integer::Integer;
use crate::core::types::map::Map;
use crate::core::types::pending::Pending;
use crate::core::types::scope::Scope;
use crate::errors::lazy_snippet;
use crate::errors::snippet;
use crate::extras::Extras;

pub(crate) fn simplify(mut path: PathBuf) -> anyhow::Result<Rc<Expression>> {
    if std::fs::metadata(&path)
        .with_context(|| format!("Query metadata of: {path:?}"))?
        .is_dir()
    {
        path.push("default.nix");
    }

    let input = std::fs::read_to_string(&path)
        .with_context(|| format!("Read file at: {path:?}"))?;

    let scope = Scope::root();
    let path = Rc::new(path);
    let expression = crate::core::parse::parse(input);

    simplify_expression(&path, &scope, *expression)
        .with_context(|| format!("Simplify expression at: {path:?}"))
}

fn simplify_expression(
    path: &Rc<PathBuf>,
    scope: &Rc<Scope>,
    expression: nixel::Expression,
) -> anyhow::Result<Rc<Expression>> {
    match expression {
        nixel::Expression::BinaryOperation(binary_operation) => {
            simplify_binary_operation(path, scope, binary_operation)
        },
        nixel::Expression::Function(function) => {
            simplify_function(path, scope, function)
        },
        nixel::Expression::FunctionApplication(function_application) => {
            simplify_function_application(path, scope, function_application)
        },
        nixel::Expression::Integer(integer) => simplify_integer(path, integer),
        nixel::Expression::LetIn(let_in) => {
            simplify_let_in(path, scope, let_in)
        },
        nixel::Expression::Identifier(identifier) => {
            simplify_identifier(path, scope, identifier)
        },
        expression => unimplemented!("{expression:#?}"),
    }
}

fn simplify_binary_operation(
    path: &Rc<PathBuf>,
    scope: &Rc<Scope>,
    binary_operation: nixel::BinaryOperation,
) -> anyhow::Result<Rc<Expression>> {
    let nixel::BinaryOperation {
        left,
        operator,
        right,
    } = binary_operation;

    let left_span = left.span();
    let operator_span = nixel::Span {
        start: Box::new(left.end()),
        end: Box::new(right.start()),
    };
    let right_span = right.span();

    let left = simplify_expression(path, scope, *left)
        .with_context(lazy_snippet(
            "Simplify left operand of an addition".to_owned(),
            path,
            left_span,
        ))?
        .unwrap_or_clone();

    let right = simplify_expression(path, scope, *right)
        .with_context(lazy_snippet(
            "Simplify right operand of an addition".to_owned(),
            path,
            right_span,
        ))?
        .unwrap_or_clone();

    match operator {
        nixel::BinaryOperator::Addition => {
            match (left, right) {
                (Expression::Integer(left), Expression::Integer(right)) => {
                    integer_plus_integer(
                        path,
                        left.value,
                        operator_span.clone(),
                        right.value,
                    )
                    .with_context(lazy_snippet(
                        format!(
                            "Add Integer ({}) to Integer ({})",
                            left.value, right.value,
                        ),
                        path,
                        operator_span,
                    ))
                },
                (left, right) => unimplemented!("{left:?} {right:?}"),
            }
        },
        operator => unimplemented!("{operator:?}"),
    }
}

fn simplify_identifier(
    path: &Rc<PathBuf>,
    scope: &Rc<Scope>,
    identifier: nixel::Identifier,
) -> anyhow::Result<Rc<Expression>> {
    let nixel::Identifier { id, span } = identifier;

    let expression = scope
        .lookup(&id)
        .with_context(lazy_snippet(
            format!("Identifier not found: {id}"),
            path,
            *span,
        ))?
        .unwrap_or_clone();

    if let Expression::Pending(Pending {
        path,
        scope,
        expression,
    }) = expression
    {
        simplify_expression(&path, &scope, expression.unwrap_or_clone())
    } else {
        Ok(Rc::new(expression))
    }
}

fn simplify_function(
    path: &Rc<PathBuf>,
    scope: &Rc<Scope>,
    function: nixel::Function,
) -> anyhow::Result<Rc<Expression>> {
    Ok(Rc::new(Expression::Function(Function {
        path: path.clone(),
        scope: scope.clone(),
        function: Rc::new(function),
    })))
}

fn simplify_function_application(
    path: &Rc<PathBuf>,
    scope: &Rc<Scope>,
    function_application: nixel::FunctionApplication,
) -> anyhow::Result<Rc<Expression>> {
    let nixel::FunctionApplication {
        function,
        arguments,
    } = function_application;

    let mut result_span = function.span();
    let mut result_start = function.start();
    let mut result_variant_name = function.variant_name();
    let mut result = simplify_expression(path, scope, *function)
        .with_context(lazy_snippet(
            "Simplify the function of a FunctionApplication".to_owned(),
            path,
            result_span.clone(),
        ))?
        .unwrap_or_clone();

    for argument in Vec::from(arguments) {
        let argument_span = argument.span();
        let argument_variant_name = argument.variant_name();

        let argument = simplify_expression(path, scope, argument)
            .with_context(lazy_snippet(
                "Simplify the argument of a FunctionApplication".to_owned(),
                path,
                argument_span.clone(),
            ))?
            .unwrap_or_clone();

        result = match result {
            Expression::Function(function) => {
                apply_function_argument(function, argument)
            },
            _ => {
                return None.context(snippet(
                    format!(
                        "Type {result_variant_name} cannot be used as a \
                         function",
                    ),
                    path,
                    result_span,
                ));
            },
        }
        .with_context(lazy_snippet(
            format!(
                "Call {result_variant_name} with {argument_variant_name} as \
                 argument",
            ),
            path,
            nixel::Span {
                start: Box::new(result_start),
                end: argument_span.end.clone(),
            },
        ))?
        .unwrap_or_clone();
        result_start = argument_span.start.as_ref().clone();
        result_span = argument_span;
        result_variant_name = argument_variant_name;
    }

    Ok(Rc::new(result))
}

fn simplify_integer(
    path: &Rc<PathBuf>,
    integer: nixel::Integer,
) -> anyhow::Result<Rc<Expression>> {
    let nixel::Integer { value, span } = integer;

    Ok(Rc::new(Expression::Integer(Integer {
        value: value.parse::<i64>().with_context(lazy_snippet(
            format!("Integer must be >= {} and <= {}", i64::MIN, i64::MAX),
            path,
            *span,
        ))?,
    })))
}

fn simplify_let_in(
    path: &Rc<PathBuf>,
    scope: &Rc<Scope>,
    let_in: nixel::LetIn,
) -> anyhow::Result<Rc<Expression>> {
    let nixel::LetIn {
        bindings, target, ..
    } = let_in;

    let scope = scope.clone().specialize();
    bind_many(&path, &scope, bindings)?;

    let target_span = target.span();

    simplify_expression(&path, &scope, *target).with_context(lazy_snippet(
        "Simplify the target of a Let In".to_owned(),
        path,
        target_span,
    ))
}

// UTILITIES

fn integer_plus_integer(
    path: &Rc<PathBuf>,
    left: i64,
    operator_span: nixel::Span,
    right: i64,
) -> anyhow::Result<Rc<Expression>> {
    Ok(Rc::new(Expression::Integer(Integer {
        value: left.checked_add(right).with_context(lazy_snippet(
            format!(
                "The result of the addition would overflow, an Integer must \
                 be <= {}",
                i64::MAX
            ),
            path,
            operator_span,
        ))?,
    })))
}

fn apply_function_argument(
    function: Function,
    argument: Expression,
) -> anyhow::Result<Rc<Expression>> {
    let Function {
        path,
        scope,
        function,
    } = function;
    let nixel::Function { head, body, .. } = function.unwrap_or_clone();

    let scope = scope.specialize();

    match head {
        nixel::FunctionHead::Simple(nixel::FunctionHeadSimple {
            identifier,
        }) => {
            scope
                .bindings
                .borrow_mut()
                .insert(identifier, Rc::new(argument));
        },
        nixel::FunctionHead::Destructured(function_head_destructured) => {
            unreachable!("{function_head_destructured:?}")
        },
    }

    simplify_expression(&path, &scope, *body)
}

fn bind_many(
    path: &Rc<PathBuf>,
    scope: &Rc<Scope>,
    bindings: Box<[nixel::Binding]>,
) -> anyhow::Result<()> {
    for binding in Vec::from(bindings) {
        bind(path, scope, binding)?;
    }

    Ok(())
}

fn bind(
    path: &Rc<PathBuf>,
    scope: &Rc<Scope>,
    binding: nixel::Binding,
) -> anyhow::Result<()> {
    match binding {
        nixel::Binding::KeyValue(binding_key_value) => {
            bind_key_value(path, scope, binding_key_value)
        },
        nixel::Binding::Inherit(binding_inherit) => {
            unimplemented!("{binding_inherit:?}")
        },
    }
}

fn bind_key_value(
    path: &Rc<PathBuf>,
    scope: &Rc<Scope>,
    nixel::BindingKeyValue { from, to }: nixel::BindingKeyValue,
) -> anyhow::Result<()> {
    let mut scope = scope.clone();

    let parts = Vec::from(from);
    let mut parts = parts.into_iter().peekable();

    while let Some(part) = parts.next() {
        let is_last_part = parts.peek().is_none();

        scope = match part {
            nixel::Part::Expression(part_expression) => {
                unimplemented!("{part_expression:?}")
            },
            nixel::Part::Interpolation(part_interpolation) => {
                unimplemented!("{part_interpolation:?}")
            },
            nixel::Part::Raw(nixel::PartRaw { content, span }) => {
                match scope.clone().bindings.try_borrow_mut()?.entry(content) {
                    Entry::Vacant(entry) => {
                        if is_last_part {
                            entry.insert(Rc::new(Expression::Pending(
                                Pending {
                                    path: path.clone(),
                                    scope: scope.clone().specialize(),
                                    expression: Rc::from(to),
                                },
                            )));
                            break;
                        } else {
                            scope = scope.clone().specialize();
                            entry.insert(Rc::new(Expression::Map(Map {
                                scope: scope.clone(),
                            })));
                            scope
                        }
                    },
                    Entry::Occupied(entry) => {
                        match entry.get().as_ref() {
                            Expression::Map(map) => {
                                let key = entry.key();

                                if is_last_part {
                                    return None
                                        .context(snippet(
                                            format!(
                                                "Binding {key:?} was \
                                                 previously defined as Map, \
                                                 but you want to bind it \
                                                 again as {}",
                                                to.variant_name(),
                                            ),
                                            path,
                                            span.as_ref().clone(),
                                        ))
                                        .context(snippet(
                                            format!("Bind {key:?}"),
                                            path,
                                            *span,
                                        ))?;
                                } else {
                                    map.scope.clone()
                                }
                            },
                            Expression::Pending(pending) => {
                                let key = entry.key();

                                return None
                                    .context(snippet(
                                        format!(
                                            "Binding {key:?} was previously \
                                             defined as {}, but you want to \
                                             bind it again as {}",
                                            pending.expression.variant_name(),
                                            if is_last_part {
                                                to.variant_name()
                                            } else {
                                                "Map"
                                            },
                                        ),
                                        path,
                                        span.as_ref().clone(),
                                    ))
                                    .context(snippet(
                                        format!("Bind {key:?}"),
                                        path,
                                        *span,
                                    ));
                            },
                            _ => unreachable!(),
                        }
                    },
                }
            },
        };
    }

    Ok(())
}
