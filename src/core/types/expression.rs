use std::rc::Rc;

use crate::core::types::integer::Integer;
use crate::core::types::map::Map;
use crate::core::types::pending::Pending;

#[derive(Clone, Debug)]
pub(crate) enum Expression {
    Integer(Integer),
    Map(Map),
    Pending(Pending),
}
