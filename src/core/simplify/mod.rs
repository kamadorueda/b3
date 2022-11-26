// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::collections::hash_map::Entry;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::Context;

use crate::core::types::expression::Expression;
use crate::core::types::integer::Integer;
use crate::core::types::map::Map;
use crate::core::types::pending::Pending;
use crate::core::types::scope::Scope;
use crate::errors::lazy_snippet;
use crate::errors::snippet;

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

    simplify_expression(&scope, &path, expression)
        .with_context(|| format!("Simplify expression at: {path:?}"))
}

fn simplify_expression(
    scope: &Rc<Scope>,
    path: &Rc<PathBuf>,
    expression: Box<nixel::Expression>,
) -> anyhow::Result<Rc<Expression>> {
    match *expression {
        nixel::Expression::BinaryOperation(binary_operation) => {
            simplify_binary_operation(scope, path, binary_operation)
        },
        nixel::Expression::Integer(integer) => simplify_integer(path, integer),
        nixel::Expression::LetIn(let_in) => {
            simplify_let_in(scope, path, let_in)
        },
        nixel::Expression::Identifier(identifier) => {
            simplify_identifier(scope, path, identifier)
        },
        expression => unimplemented!("{expression:#?}"),
    }
}

fn simplify_binary_operation(
    scope: &Rc<Scope>,
    path: &Rc<PathBuf>,
    binary_operation: nixel::BinaryOperation,
) -> anyhow::Result<Rc<Expression>> {
    let nixel::BinaryOperation { left, operator, right } = binary_operation;

    let left_span = left.span();
    let operator_span = nixel::Span {
        start: Box::new(left.end()),
        end: Box::new(right.start()),
    };
    let right_span = right.span();

    let left = simplify_expression(scope, path, left)
        .with_context(lazy_snippet(
            "Evaluate left operand of an addition".to_owned(),
            path,
            left_span,
        ))?
        .unwrap_or_clone();

    let right = simplify_expression(scope, path, right)
        .with_context(lazy_snippet(
            "Evaluate right operand of an addition".to_owned(),
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
    scope: &Rc<Scope>,
    path: &Rc<PathBuf>,
    identifier: nixel::Identifier,
) -> anyhow::Result<Rc<Expression>> {
    let nixel::Identifier { id, span } = identifier;

    Ok(scope.lookup(&id).with_context(lazy_snippet(
        format!("Identifier not found: {id}"),
        path,
        *span,
    ))?)
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
    scope: &Rc<Scope>,
    path: &Rc<PathBuf>,
    let_in: nixel::LetIn,
) -> anyhow::Result<Rc<Expression>> {
    let nixel::LetIn { bindings, target, .. } = let_in;

    let scope = scope.clone().specialize();
    bind_many(&scope, path, bindings)?;

    let target_span = target.span();

    simplify_expression(&scope, path, target).with_context(lazy_snippet(
        "Evaluate the target of a Let In".to_owned(),
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

fn bind_many(
    scope: &Rc<Scope>,
    path: &Rc<PathBuf>,
    bindings: Box<[nixel::Binding]>,
) -> anyhow::Result<()> {
    for binding in Vec::from(bindings) {
        bind(scope, path, binding)?;
    }

    Ok(())
}

fn bind(
    scope: &Rc<Scope>,
    path: &Rc<PathBuf>,
    binding: nixel::Binding,
) -> anyhow::Result<()> {
    match binding {
        nixel::Binding::KeyValue(binding_key_value) => {
            bind_key_value(scope.clone(), path, binding_key_value)
        },
        nixel::Binding::Inherit(binding_inherit) => {
            unimplemented!("{binding_inherit:?}")
        },
    }
}

fn bind_key_value(
    mut scope: Rc<Scope>,
    path: &Rc<PathBuf>,
    nixel::BindingKeyValue { from, to }: nixel::BindingKeyValue,
) -> anyhow::Result<()> {
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
                                    expression: Rc::from(to),
                                    scope: scope.clone().specialize(),
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
                                    return Err(anyhow::Error::msg(snippet(
                                        format!(
                                            "Binding {key:?} was previously \
                                             defined as Map, but you want to \
                                             bind it again as {}",
                                            to.variant_name(),
                                        ),
                                        path,
                                        span.as_ref().clone(),
                                    ))
                                    .context(snippet(
                                        format!("Bind {key:?}"),
                                        path,
                                        *span,
                                    )));
                                } else {
                                    map.scope.clone()
                                }
                            },
                            Expression::Pending(pending) => {
                                let key = entry.key();

                                return Err(anyhow::Error::msg(snippet(
                                    format!(
                                        "Binding {key:?} was previously \
                                         defined as {}, but you want to bind \
                                         it again as {}",
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
                                )));
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
