// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::path::PathBuf;
use std::rc::Rc;

use anyhow::Context;

use crate::core::types::expression::Expression;
use crate::core::types::integer::Integer;
use crate::core::types::map::Map;
use crate::core::types::pending::Pending;
use crate::core::types::scope::Scope;
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

    let expression = crate::core::parse::parse(input);

    let path: Rc<PathBuf> = Rc::new(path);

    simplify_expression(Scope::root(), path.clone(), expression)
        .with_context(|| format!("Simplify expression at: {path:?}"))
}

fn simplify_expression(
    scope: Rc<Scope>,
    path: Rc<PathBuf>,
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
        expression => unimplemented!("{expression:#?}"),
    }
}

fn simplify_binary_operation(
    scope: Rc<Scope>,
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

    let left = simplify_expression(scope.clone(), path.clone(), left)
        .with_context(snippet(
            "Evaluate left operand of an addition".to_owned(),
            path.clone(),
            left_span,
        ))?
        .unwrap_or_clone();

    let right = simplify_expression(scope.clone(), path.clone(), right)
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
                        path.clone(),
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
            path.clone(),
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
            path.clone(),
            *span,
        ))?,
    })))
}

fn simplify_let_in(
    scope: Rc<Scope>,
    path: Rc<PathBuf>,
    let_in: nixel::LetIn,
) -> anyhow::Result<Rc<Expression>> {
    let nixel::LetIn { bindings, target, .. } = let_in;

    let mut scope = scope.specialize();
    bind_many(&mut scope, bindings);
    let scope = scope.into_rc();

    let target_span = target.span();

    simplify_expression(scope, path.clone(), target).with_context(snippet(
        "Evaluate the target of a Let In".to_owned(),
        path,
        target_span,
    ))
}

pub(crate) fn bind_many(
    scope: &mut Scope,
    bindings: Box<[nixel::Binding]>,
) -> anyhow::Result<()> {
    for binding in bindings.to_vec() {
        bind(scope, binding)?;
    }

    Ok(())
}

pub(crate) fn bind(
    scope: &mut Scope,
    binding: nixel::Binding,
) -> anyhow::Result<()> {
    match binding {
        nixel::Binding::KeyValue(key_value) => unimplemented!("{key_value:?}"),
        nixel::Binding::Inherit(inherit) => unimplemented!("{inherit:?}"),
    }
}

pub(crate) fn bind_key_value(
    mut scope: &mut Scope,
    nixel::BindingKeyValue { from, to }: nixel::BindingKeyValue,
) -> anyhow::Result<()> {
    let this = scope.this();

    for part in from.to_vec() {
        match part {
            nixel::Part::Expression(part_expression) => {
                unimplemented!("{part_expression:?}")
            },
            nixel::Part::Interpolation(part_interpolation) => {
                unimplemented!("{part_interpolation:?}")
            },
            nixel::Part::Raw(part_raw) => {
                scope
                    .bindings
                    .entry(part_raw.content)
                    // .and_modify(|expression| {})
                    .or_insert_with(|| {
                        Rc::new(Expression::Pending(Pending {
                            expression: to,
                            scope: scope.this(),
                        }))
                    });
            },
        }
    }

    Ok(())
}
