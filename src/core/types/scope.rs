use std::collections::HashMap;
use std::rc::Rc;
use std::rc::Weak;

use derivative::Derivative;

use crate::core::types::expression::Expression;

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct Scope {
    #[derivative(Debug = "ignore")]
    pub(crate) bindings: HashMap<Box<str>, Rc<Expression>>,
    #[derivative(Debug = "ignore")]
    pub(crate) parent: Option<Rc<Scope>>,
    #[derivative(Debug = "ignore")]
    pub(crate) this: Weak<Scope>,
}

impl Scope {
    pub(crate) fn root() -> Rc<Self> {
        Rc::new_cyclic(|this| {
            Self { this: this.clone(), parent: None, bindings: HashMap::new() }
        })
    }

    pub(crate) fn into_rc(self) -> Rc<Self> {
        let Self { bindings, parent, .. } = self;
        Rc::new_cyclic(|this| Self { this: this.clone(), parent, bindings })
    }

    pub(crate) fn this(&self) -> Rc<Self> {
        self.this.upgrade().unwrap()
    }

    pub(crate) fn specialize(self: Rc<Self>) -> Self {
        Rc::new_cyclic(|this| {
            Self {
                this: this.clone(),
                parent: Some(self),
                bindings: HashMap::new(),
            }
        })
    }
}

// a = 4;
// a.b // error
// b = {};
// b.c = 4;
