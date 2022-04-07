// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::value::Value;

#[derive(Clone)]
pub(crate) struct Scope {
    kind:     ScopeKind,
    bindings: Rc<RefCell<HashMap<String, Rc<Value>>>>,
    parent:   Option<Box<Scope>>,
}

#[derive(Clone, Debug)]
pub(crate) enum ScopeKind {
    Plain,
}

impl Scope {
    pub(crate) fn empty() -> Scope {
        Scope {
            kind:     ScopeKind::Plain,
            bindings: Rc::new(RefCell::new(HashMap::new())),
            parent:   None,
        }
    }

    pub(crate) fn derive(&self, kind: ScopeKind) -> Scope {
        Scope {
            kind,
            bindings: Rc::new(RefCell::new(HashMap::new())),
            parent: Some(Box::new(self.clone())),
        }
    }

    pub(crate) fn bind(&self, identifier: String, value: Rc<Value>) {
        self.bindings.borrow_mut().insert(identifier, value);
    }

    pub(crate) fn lookup(&self, identifier: &String) -> Option<Rc<Value>> {
        let mut scope = self;

        loop {
            if let Some(value) = scope.bindings.borrow().get(identifier) {
                return Some(value.clone());
            }

            if let Some(parent_scope) = &scope.parent {
                scope = parent_scope;
            } else {
                break;
            }
        }

        None
    }
}

impl std::fmt::Debug for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Scope {{ kind: {:?}, bindings: {:?} }}",
            self.kind,
            self.bindings.borrow().len()
        )
    }
}
