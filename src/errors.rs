// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::path::PathBuf;
use std::rc::Rc;

pub fn print<Closure>(closure: Closure)
where
    Closure: FnOnce() -> anyhow::Result<()>,
{
    if let Err(error) = closure() {
        if let Some(error) = error.downcast_ref::<clap::Error>() {
            log::error!(target: "stderr", "{error}");
        } else {
            log::error!("An error ocurred. You'll find the context below:");
            error
                .chain()
                .enumerate()
                .map(|(step, cause)| (step + 1, cause))
                .for_each(|(step, cause)| log::error!("{step}. {cause}"));
        }
    }
}

pub(crate) fn lazy_snippet(
    message: String,
    path: &Rc<PathBuf>,
    span: nixel::Span,
) -> Box<dyn FnOnce() -> String> {
    let path = path.clone();

    Box::new(move || snippet(message, &path, span))
}

pub(crate) fn snippet(
    message: String,
    path: &Rc<PathBuf>,
    span: nixel::Span,
) -> String {
    let nixel::Span { start, end } = span;
    let nixel::Position {
        line: start_line,
        column: start_column,
    } = *start;
    let nixel::Position {
        line: end_line,
        column: end_column,
    } = *end;

    format!(
        "{message}. In {path:?}, \
         L{start_line}C{start_column}-L{end_line}C{end_column}"
    )
}
