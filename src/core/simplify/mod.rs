// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::path::PathBuf;
use std::rc::Rc;

use anyhow::Context;

use crate::core::types::expression::Expression;
use crate::core::types::integer::Integer;

use crate::core::types::scope::Scopes;
use crate::errors::snippet;

#[derive(Debug)]
pub(crate) struct Simplified {
    pub(crate) scopes: Scopes,
    pub(crate) expression: Rc<Expression>,
}

pub(crate) fn simplify(mut path: PathBuf) -> anyhow::Result<Simplified> {
    if std::fs::metadata(&path)
        .with_context(|| format!("Query metadata of: {path:?}"))?
        .is_dir()
    {
        path.push("default.nix");
    }

    let input = std::fs::read_to_string(&path)
        .with_context(|| format!("Read file at: {path:?}"))?;

    let (mut scopes, scope_index) = Scopes::root();
    let path: Rc<PathBuf> = Rc::new(path);
    let expression = crate::core::parse::parse(input);

    simplify_expression(&mut scopes, scope_index, path.clone(), expression)
        .with_context(|| format!("Simplify expression at: {path:?}"))
        .map(|expression| Simplified { scopes, expression })
}

fn simplify_expression(
    scopes: &mut Scopes,
    scope_index: usize,
    path: Rc<PathBuf>,
    expression: Box<nixel::Expression>,
) -> anyhow::Result<Rc<Expression>> {
    match *expression {
        nixel::Expression::BinaryOperation(binary_operation) => {
            simplify_binary_operation(
                scopes,
                scope_index,
                path,
                binary_operation,
            )
        },
        nixel::Expression::Integer(integer) => simplify_integer(path, integer),
        nixel::Expression::LetIn(let_in) => {
            simplify_let_in(scopes, scope_index, path, let_in)
        },
        expression => unimplemented!("{expression:#?}"),
    }
}

fn simplify_binary_operation(
    scopes: &mut Scopes,
    scope_index: usize,
    path: Rc<PathBuf>,
    binary_operation: nixel::BinaryOperation,
) -> anyhow::Result<Rc<Expression>> {
    let nixel::BinaryOperation { left, operator, right } = binary_operation;

    let left_span = left.span();
    let operator_span = nixel::Span {
        start: Box::new(left.end()),
        end: Box::new(right.start()),
    };
    let right_span = right.span();

    let left = simplify_expression(scopes, scope_index, path.clone(), left)
        .with_context(snippet(
            "Evaluate left operand of an addition".to_owned(),
            path.clone(),
            left_span,
        ))?
        .unwrap_or_clone();

    let right = simplify_expression(scopes, scope_index, path.clone(), right)
        .with_context(snippet(
            "Evaluate right operand of an addition".to_owned(),
            path.clone(),
            right_span,
        ))?
        .unwrap_or_clone();

    match operator {
        nixel::BinaryOperator::Addition => {
            match (left, right) {
                (Expression::Integer(left), Expression::Integer(right)) => {
                    simplify_binary_operation_integer_integer(
                        path.clone(),
                        left.value,
                        operator_span.clone(),
                        right.value,
                    )
                    .with_context(snippet(
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

fn simplify_binary_operation_integer_integer(
    path: Rc<PathBuf>,
    left: i64,
    operator_span: nixel::Span,
    right: i64,
) -> anyhow::Result<Rc<Expression>> {
    Ok(Rc::new(Expression::Integer(Integer {
        value: left.checked_add(right).with_context(snippet(
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

fn simplify_integer(
    path: Rc<PathBuf>,
    integer: nixel::Integer,
) -> anyhow::Result<Rc<Expression>> {
    let nixel::Integer { value, span } = integer;

    Ok(Rc::new(Expression::Integer(Integer {
        value: value.parse::<i64>().with_context(snippet(
            format!("Integer must be >= {} and <= {}", i64::MIN, i64::MAX),
            path,
            *span,
        ))?,
    })))
}

fn simplify_let_in(
    scopes: &mut Scopes,
    scope_index: usize,
    path: Rc<PathBuf>,
    let_in: nixel::LetIn,
) -> anyhow::Result<Rc<Expression>> {
    let nixel::LetIn { bindings, target, .. } = let_in;

    let scope_index = scopes.specialize(scope_index);
    bind_many(scopes, scope_index, bindings);

    let target_span = target.span();

    simplify_expression(scopes, scope_index, path.clone(), target).with_context(
        snippet(
            "Evaluate the target of a Let In".to_owned(),
            path,
            target_span,
        ),
    )
}

pub(crate) fn bind_many(
    scopes: &mut Scopes,
    scope_index: usize,
    bindings: Box<[nixel::Binding]>,
) -> anyhow::Result<()> {
    for binding in Vec::from(bindings) {
        bind(scopes, scope_index, binding)?;
    }

    Ok(())
}

pub(crate) fn bind(
    scopes: &mut Scopes,
    scope_index: usize,
    binding: nixel::Binding,
) -> anyhow::Result<()> {
    match binding {
        nixel::Binding::KeyValue(binding_key_value) => {
            bind_key_value(scopes, scope_index, binding_key_value)
        },
        nixel::Binding::Inherit(binding_inherit) => {
            unimplemented!("{binding_inherit:?}")
        },
    }
}

pub(crate) fn bind_key_value(
    _scopes: &mut Scopes,
    _scope_index: usize,
    nixel::BindingKeyValue { from: _, to: _ }: nixel::BindingKeyValue,
) -> anyhow::Result<()> {
    // let this = scope.this();

    // for part in Vec::from(from) {
    //     match part {
    //         nixel::Part::Expression(part_expression) => {
    //             unimplemented!("{part_expression:?}")
    //         },
    //         nixel::Part::Interpolation(part_interpolation) => {
    //             unimplemented!("{part_interpolation:?}")
    //         },
    //         nixel::Part::Raw(part_raw) => {
    //             scope
    //                 .bindings
    //                 .entry(part_raw.content)
    //                 // .and_modify(|expression| {})
    //                 .or_insert_with(|| {
    //                     Rc::new(Expression::Pending(Pending {
    //                         expression: to,
    //                         scope: scope.this(),
    //                     }))
    //                 });
    //         },
    //     }
    // }

    Ok(())
}
