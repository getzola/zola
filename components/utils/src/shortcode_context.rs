use std::cell::RefCell;
use libs::tera::Context;

thread_local! {
    /// Thread-local storage for the current rendering context
    /// This allows shortcode functions to access page/section/config during rendering
    static CURRENT_CONTEXT: RefCell<Option<Context>> = const { RefCell::new(None) };
}

/// Set the current rendering context for this thread
/// Should be called before rendering a template that might contain shortcodes
pub fn set_context(context: Context) {
    CURRENT_CONTEXT.with(|c| {
        *c.borrow_mut() = Some(context);
    });
}

/// Clear the current rendering context
/// Should be called after template rendering is complete
pub fn clear_context() {
    CURRENT_CONTEXT.with(|c| {
        *c.borrow_mut() = None;
    });
}

/// Get a copy of the current rendering context
pub fn get_context() -> Option<Context> {
    CURRENT_CONTEXT.with(|c| c.borrow().clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_set_and_get_context() {
        let mut context = Context::new();
        context.insert("test", &"value");

        set_context(context.clone());

        let retrieved = get_context();
        assert!(retrieved.is_some());
        assert_eq!(
            retrieved.unwrap().get("test").unwrap().as_str().unwrap(),
            "value"
        );

        clear_context();
        assert!(get_context().is_none());
    }
}
