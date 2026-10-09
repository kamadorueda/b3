// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use crate::core::types::function::Function;
use crate::core::types::integer::Integer;
use crate::core::types::map::Map;
use crate::core::types::pending::Pending;

#[derive(Clone, Debug)]
pub(crate) enum Expression {
    Integer(Integer),
    Function(Function),
    Map(Map),
    Pending(Pending),
}

impl Expression {
    pub(crate) fn variant_name(&self) -> &'static str {
        match self {
            Self::Integer(_) => "Integer",
            Self::Function(_) => "Function",
            Self::Map(_) => "Map",
            Self::Pending(_) => "Pending",
        }
    }
}
