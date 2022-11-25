use std::path::PathBuf;
use std::rc::Rc;

pub fn print<Closure>(closure: Closure)
where
    Closure: FnOnce() -> anyhow::Result<()>,
{
    if let Err(error) = closure() {
        if let Some(error) = error.downcast_ref::<clap::Error>() {
            log::error!(target: "stderr", "{error}");
        } else {
            log::error!("An error ocurred. You'll find the context below:");
            error
                .chain()
                .enumerate()
                .map(|(step, cause)| (step + 1, cause))
                .for_each(|(step, cause)| log::error!("{step}. {cause}"));
        }
    }
}

pub(crate) fn snippet(
    message: String,
    path: Rc<PathBuf>,
    nixel::Span { start, end }: nixel::Span,
) -> Box<dyn FnOnce() -> String> {
    let nixel::Position { line: start_line, column: start_column } = *start;
    let nixel::Position { line: end_line, column: end_column } = *end;

    Box::new(move || {
        format!(
            "{message}. In {path:?}, \
             L{start_line}C{start_column}-L{end_line}C{end_column}"
        )
    })
}
