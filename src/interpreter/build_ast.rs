// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use nixel::ast::AST;
use nixel::deps::santiago::grammar::Grammar;
use nixel::deps::santiago::lexer::lex;
use nixel::deps::santiago::lexer::LexerRules;
use nixel::deps::santiago::parser::parse;

use crate::interpreter::error::Error;

pub(crate) fn build_ast(
    lexer_rules: &LexerRules,
    grammar: &Grammar<AST>,
    path: &str,
) -> Result<AST, Error> {
    log::trace!("read_to_string");
    let input = std::fs::read_to_string(path)?;
    log::trace!("lex");
    let lexemes = lex(lexer_rules, &input)?;
    log::trace!("parse");
    let parse_trees = parse(grammar, &lexemes)?;
    log::trace!("as_abstract_syntax_tree");
    let ast = parse_trees[0].as_abstract_syntax_tree();

    Ok(ast)
}
