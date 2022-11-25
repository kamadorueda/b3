// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::Context;

use crate::core::types::expression::Expression;
use crate::core::types::integer::Integer;
use crate::core::types::source::Source;

pub(crate) fn simplify(path: &PathBuf) -> anyhow::Result<Rc<Expression>> {
    let mut path = path.clone();

    if std::fs::metadata(&path)
        .with_context(|| format!("Unable to query metadata: {path:?}"))?
        .is_dir()
    {
        path.push("default.nix");
    }

    let input = std::fs::read_to_string(&*path)
        .with_context(|| format!("Unable to read file: {path:?}"))?;

    let expression = crate::core::parse::parse(input);

    simplify_expression(path.into(), expression)
}

fn simplify_expression(
    path: Rc<Path>,
    expression: Box<nixel::Expression>,
) -> anyhow::Result<Rc<Expression>> {
    Ok(Rc::new(match *expression {
        nixel::Expression::Integer(nixel::Integer { value, span }) => {
            Expression::Integer(Integer {
                value: value.parse::<i64>().with_context(|| {
                    format!(
                        "Integers must be >= {} and <= {}",
                        i64::MIN,
                        i64::MAX
                    )
                })?,
                source: Source::FileSystem { path, span },
            })
        },
        expression => unimplemented!("{expression:#?}"),
    }))
}
