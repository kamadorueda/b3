use crate::core::types::integer::Integer;

#[derive(Clone, Debug)]
pub(crate) enum Expression {
    Integer(Integer),
}
