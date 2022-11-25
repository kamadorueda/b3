use std::rc::Rc;

pub(crate) trait RcUnwrapOrClone {
    fn unwrap_or_clone(self: Rc<Self>) -> Self
    where
        Self: Clone,
    {
        Rc::try_unwrap(self).unwrap_or_else(|rc| (*rc).clone())
    }
}

impl<T> RcUnwrapOrClone for T where T: Clone {}
