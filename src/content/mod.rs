// TODO: move section/page and maybe pagination in this mod
// Not sure where pagination stands if I add a render mod

mod page;
mod pagination;
mod section;

pub use self::page::{Page, sort_pages, populate_previous_and_next_pages};
pub use self::section::{Section};
pub use self::pagination::{Paginator, Pager};
