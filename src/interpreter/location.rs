// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::io::Read;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub(crate) enum Location {
    InFileFragment(LocationInFileFragment),
}

#[derive(Clone, Debug)]
pub(crate) struct LocationInFileFragment {
    pub(crate) column: usize,
    pub(crate) line:   usize,
    pub(crate) path:   Rc<String>,
}

impl Location {
    pub(crate) fn as_path(&self) -> &str {
        match &self {
            Location::InFileFragment(in_file_fragment) => {
                in_file_fragment.path.as_str()
            }
        }
    }

    pub(crate) fn snippet(&self, context: usize) -> Option<String> {
        match &self {
            Location::InFileFragment(fragment) => {
                let path = fragment.path.as_str();
                let mut file = match std::fs::File::open(path) {
                    Ok(file) => file,
                    Err(_) => return None,
                };

                let mut string = String::new();
                match file.read_to_string(&mut string) {
                    Ok(_) => {}
                    Err(_) => return None,
                };

                let mut snippet = String::new();
                let mut line_no = fragment.line.saturating_sub(context);
                let width =
                    ((line_no + context) as f64).log10().ceil() as usize;

                for line in string.lines().skip(line_no).take(context) {
                    line_no += 1;
                    snippet.push_str(&format!(
                        "{0} {1:>2$} | {3}\n",
                        if line_no == fragment.line { ">" } else { " " },
                        line_no,
                        width,
                        line.replace("\t", " ")
                    ))
                }

                if snippet.is_empty() {
                    None
                } else {
                    for _ in 0..(fragment.column + width + 4) {
                        snippet.push_str(" ");
                    }
                    snippet.push_str("^\n");

                    Some(snippet)
                }
            }
        }
    }
}
