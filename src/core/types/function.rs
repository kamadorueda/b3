// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::path::PathBuf;
use std::rc::Rc;

use crate::core::types::scope::Scope;

#[derive(Clone, Debug)]
pub(crate) struct Function {
    pub(crate) path: Rc<PathBuf>,
    pub(crate) scope: Rc<Scope>,
    pub(crate) function: Rc<nixel::Function>,
}
