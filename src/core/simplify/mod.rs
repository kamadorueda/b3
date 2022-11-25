// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::path::PathBuf;
use std::rc::Rc;

use anyhow::Context;

use crate::core::types::expression::Expression;
use crate::core::types::integer::Integer;
use crate::errors::snippet;
use crate::mem::RcUnwrapOrClone;

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

    let path: Rc<PathBuf> = path.into();

    simplify_expression(path.clone(), expression)
        .with_context(|| format!("Simplify expression at: {path:?}"))
}

fn simplify_expression(
    path: Rc<PathBuf>,
    expression: Box<nixel::Expression>,
) -> anyhow::Result<Rc<Expression>> {
    match *expression {
        nixel::Expression::BinaryOperation(binary_operation) => {
            simplify_binary_operation(path, binary_operation)
        },
        nixel::Expression::Integer(integer) => simplify_integer(path, integer),
        expression => unimplemented!("{expression:#?}"),
    }
}

fn simplify_binary_operation(
    path: Rc<PathBuf>,
    binary_operation: nixel::BinaryOperation,
) -> anyhow::Result<Rc<Expression>> {
    let nixel::BinaryOperation { left, operator, right } = binary_operation;

    let left_span = left.span();
    let operator_span =
        nixel::Span { start: left.end().into(), end: right.start().into() };
    let right_span = right.span();

    let left = simplify_expression(path.clone(), left)
        .with_context(snippet(
            "Evaluate left operand of an addition".to_owned(),
            path.clone(),
            left_span,
        ))?
        .unwrap_or_clone();

    let right = simplify_expression(path.clone(), right)
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
