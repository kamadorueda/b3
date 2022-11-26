use std::collections::HashMap;
use std::rc::Rc;

use crate::core::types::expression::Expression;

#[derive(Debug)]
pub(crate) struct Scopes {
    pub(crate) scopes: Vec<Scope>,
}

impl Scopes {
    pub(crate) fn root() -> (Self, usize) {
        (
            Self {
                scopes: vec![Scope {
                    bindings: HashMap::default(),
                    parent: None,
                }],
            },
            0,
        )
    }

    pub(crate) fn specialize(&mut self, scope_index: usize) -> usize {
        let index = self.scopes.len();

        self.scopes.push(Scope {
            bindings: HashMap::default(),
            parent: Some(scope_index),
        });

        index
    }
}

#[derive(Debug)]
pub(crate) struct Scope {
    pub(crate) bindings: HashMap<Box<str>, Rc<Expression>>,
    pub(crate) parent: Option<usize>,
}
