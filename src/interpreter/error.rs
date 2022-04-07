// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::collections::LinkedList;

use nixel::ast::AST;
use nixel::deps::santiago::lexer::LexerError;
use nixel::deps::santiago::parser::ParseError;

use super::location::Location;
use super::runtime_stack_frame::RuntimeStackFrame;

#[derive(Debug)]
pub(crate) enum Error {
    Interpreter {
        description: String,
        location:    Location,
        stack:       LinkedList<RuntimeStackFrame>,
    },
    IO(std::io::Error),
    Lexer(LexerError),
    Parser(ParseError<AST>),
}

impl std::convert::From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::IO(error)
    }
}

impl std::convert::From<LexerError> for Error {
    fn from(error: LexerError) -> Error {
        Error::Lexer(error)
    }
}

impl std::convert::From<ParseError<AST>> for Error {
    fn from(error: ParseError<AST>) -> Error {
        Error::Parser(error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        match &self {
            Error::Interpreter { description, location, stack } => {
                writeln!(f, "{} error, most recent action last:", self.kind())?;
                writeln!(f)?;

                let mut stack = stack.clone();

                stack.push_back(RuntimeStackFrame {
                    description: description.clone(),
                    location:    location.clone(),
                });

                for frame in stack {
                    writeln!(f, "{frame}")?;
                }

                Ok(())
            }
            error => {
                writeln!(f, "{} error: {:?}", self.kind(), error)
            }
        }
    }
}

impl Error {
    fn kind(&self) -> &str {
        match &self {
            Error::Interpreter { .. } => "Interpreter",
            Error::IO(_) => "Input/Output",
            Error::Lexer(_) => "Lexer",
            Error::Parser(_) => "Parser",
        }
    }
}
