use std::rc::Rc;

use crate::core::types::scope::Scope;

#[derive(Clone, Debug)]
pub(crate) struct Pending {
    pub(crate) expression: Box<nixel::Expression>,
    pub(crate) scope: Rc<Scope>,
}
