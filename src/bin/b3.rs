// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

fn main() {
    b3::logger::initialize();
    b3::cli::main(std::env::args()).unwrap_or_else(|error| error.exit())
}
