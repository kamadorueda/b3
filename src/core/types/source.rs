use std::path::Path;
use std::rc::Rc;

#[derive(Debug)]
pub(crate) enum Source {
    FileSystem { path: Rc<Path>, span: Box<nixel::Span> },
}
