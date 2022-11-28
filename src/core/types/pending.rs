use std::path::PathBuf;
use std::rc::Rc;

use crate::core::types::scope::Scope;

#[derive(Clone, Debug)]
pub(crate) struct Pending {
    pub(crate) path: Rc<PathBuf>,
    pub(crate) scope: Rc<Scope>,
    pub(crate) expression: Rc<nixel::Expression>,
}
