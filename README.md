<!--
SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>

SPDX-License-Identifier: AGPL-3.0-only
-->

<h1 align="center">🐂 Toros</h2>

<p align="center">An implementation of Nix in Rust.</p>

<p align="center">
  <a href="https://buildkite.com/kamadorueda/toros">
    <img
      alt="CI/CD"
      src="https://badge.buildkite.com/e6a10842c4ea84190bee67360062b18a7e0c548f66ed0886a6.svg?branch=main"
    >
    </img>
  </a>
  <a href="https://docs.rs/toros">
    <img
      alt="Documentation"
      src="https://img.shields.io/docsrs/toros?color=brightgreen"
    >
    </img>
  </a>
  <a href="https://coveralls.io/github/kamadorueda/toros?branch=main">
    <img
      alt="Coverage"
      src="https://coveralls.io/repos/github/kamadorueda/toros/badge.svg?branch=main"
    >
    </img>
  </a>
  <a href="https://crates.io/crates/toros">
    <img
      alt="Version"
      src="https://img.shields.io/crates/v/toros?color=brightgreen"
    >
    </img>
  </a>
  <a href="https://spdx.org/licenses/AGPL-3.0-only.html">
    <img
      alt="License"
      src="https://img.shields.io/crates/l/toros?color=brightgreen"
    >
    </img>
  </a>

</p>

- Syntax support:
  - [x] With [NixEL](https://github.com/kamadorueda/nixel)
- Interpreter support:
  - [x] Int
  - [x] Binding (aliasing)
  - [x] Let-in
        (flat bindings without interpolation like `a = 123;`)
  - [x] Function (without destructuring and ellipsis)
  - [x] Function Application
  - [x] Deferred Values (Laziness)
- Built-ins:
  - [x] Addition (+)
  - [x] Subtraction (-)
  - [x] Multiplication (*)
- Store interface:
  - [ ] Rust trait
- Store implementations:
  - [ ] On Disk
  - [ ] S3-like
  - [ ] IPFS
- Good error messages (location, message, call stack)
  - [x] In CLI options/commands
  - [ ] Lexing/parsing errors
  - [x] Evaluation errors
