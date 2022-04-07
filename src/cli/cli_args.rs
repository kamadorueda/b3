// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use log::LevelFilter;

use super::cli_action::CliAction;

#[derive(Debug)]
pub(crate) struct CliArgs {
    pub(crate) action:    CliAction,
    pub(crate) log_level: LevelFilter,
}
