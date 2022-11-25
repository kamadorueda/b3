use crate::core::types::source::Source;

#[derive(Debug)]
pub(crate) struct Integer {
    pub(crate) value: i64,
    pub(crate) source: Source,
}
