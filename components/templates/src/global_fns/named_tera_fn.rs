use std::collections::HashMap;
use tera::{Function as TeraFn, Result, Value, Error};

pub trait NamedTeraFn {
    const NAME: &'static str;
}

#[derive(Debug)]
pub struct WrappedTeraFn<F> {
    f: F
}

impl <F> WrappedTeraFn<F> {
    pub fn new(f: F) -> WrappedTeraFn<F> {
        WrappedTeraFn { f }
    }
}

impl <F> NamedTeraFn for WrappedTeraFn<F> where F: NamedTeraFn {
    const NAME: &'static str = F::NAME;
}

impl <F> TeraFn for WrappedTeraFn<F> where F: NamedTeraFn + TeraFn {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        self.f.call(args).map_err(|e| {
            Error::chain(format!("Failed in tera function call to `{}`", F::NAME), e)
        })
    }
}
