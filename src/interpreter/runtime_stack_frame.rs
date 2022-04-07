// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::location::Location;

#[derive(Clone, Debug)]
pub(crate) struct RuntimeStackFrame {
    pub(crate) description: String,
    pub(crate) location:    Location,
}

impl std::fmt::Display for RuntimeStackFrame {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        writeln!(f, "At {:?}, {}", self.location.as_path(), &self.description)?;

        if let Some(snippet) = self.location.snippet(1) {
            for snippet_line in snippet.lines() {
                writeln!(f, "  {snippet_line}")?;
            }
        }

        Ok(())
    }
}
