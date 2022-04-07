// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::ffi::OsString;
use std::rc::Rc;

use nixel::ast::AST;
use nixel::deps::santiago::grammar::Grammar;
use nixel::deps::santiago::lexer::LexerRules;
use nixel::grammar::grammar;
use nixel::lexer::lexer_rules;

use crate::cli::cli_action::CliAction;
use crate::cli::parse::parse;
use crate::interpreter::build_ast::build_ast;
use crate::interpreter::error::Error;
use crate::interpreter::runtime::Runtime;
use crate::interpreter::scope::Scope;
use crate::interpreter::value::Value;

pub fn main<I, T>(cli_args: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli_args = parse(cli_args);

    log::set_max_level(cli_args.log_level);
    log::debug!("{:#?}", cli_args);

    match cli_args.action {
        CliAction::Eval { entrypoint } => match main_eval(entrypoint) {
            Ok(value) => {
                log::info!("value = {:#?}", &value);
                0
            }
            Err(error) => {
                log::error!("{}", error);
                1
            }
        },
    }
}

fn main_eval(entrypoint: String) -> Result<Value, Error> {
    let lexer_rules: LexerRules = lexer_rules();
    let grammar: Grammar<AST> = grammar();
    let entrypoint: Rc<String> = Rc::new(entrypoint);

    let ast = build_ast(&lexer_rules, &grammar, &entrypoint)?;
    log::debug!("ast = {:#?}", &ast);

    let mut runtime = Runtime::new();
    let scope = Scope::empty();

    let value = Rc::new(Value::from_ast(entrypoint, ast, &scope));

    let value = runtime.advance_monotonically(value)?;

    let value = Rc::try_unwrap(value).unwrap();

    Ok(value)
}
