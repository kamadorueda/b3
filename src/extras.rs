// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::rc::Rc;

pub trait Extras {
    fn unwrap_or_clone(self: Rc<Self>) -> Self
    where
        Self: Clone,
    {
        Rc::try_unwrap(self).unwrap_or_else(|rc| (*rc).clone())
    }
}

impl<T> Extras for T where Self: Clone {}
