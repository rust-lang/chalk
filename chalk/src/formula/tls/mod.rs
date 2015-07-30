thread_local! {
    terms: RefCell<Option<Terms>> = RefCell::new(None)
}

