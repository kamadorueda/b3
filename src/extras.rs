use std::rc::Rc;

pub trait Extras {
    fn unwrap_or_clone(self: Rc<Self>) -> Self
    where
        Self: Clone,
    {
        Rc::try_unwrap(self).unwrap_or_else(|rc| (*rc).clone())
    }
}

impl<T> Extras for T where Self: Clone {}
