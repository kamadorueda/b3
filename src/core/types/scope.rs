use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use derivative::Derivative;

use crate::core::types::expression::Expression;

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct Scope {
    pub(crate) bindings: RefCell<HashMap<Box<str>, Rc<Expression>>>,
    #[derivative(Debug = "ignore")]
    pub(crate) parent: Option<Rc<Scope>>,
}

impl Scope {
    pub(crate) fn root() -> Rc<Self> {
        Rc::new(Self {
            bindings: RefCell::new(HashMap::default()),
            parent: None,
        })
    }

    pub(crate) fn specialize(self: Rc<Self>) -> Rc<Self> {
        Rc::new(Self {
            bindings: RefCell::new(HashMap::default()),
            parent: Some(self),
        })
    }

    pub(crate) fn lookup(
        self: &Rc<Self>,
        key: &Box<str>,
    ) -> Option<Rc<Expression>> {
        let mut maybe_scope = Some(self);

        while let Some(scope) = maybe_scope {
            let value = scope.bindings.borrow().get(key).cloned();

            if value.is_some() {
                return value;
            }

            maybe_scope = scope.parent.as_ref();
        }

        None
    }
}
