use std::rc::Rc;

use crate::core::types::scope::Scope;

#[derive(Clone, Debug)]
pub(crate) struct Map {
    pub(crate) scope: Rc<Scope>,
}
