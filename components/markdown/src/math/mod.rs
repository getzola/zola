use std::{hash::Hash, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::cache::GenericCache;

pub mod katex;
pub mod svgo;
pub mod typst;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShouldMinify<'a> {
    Yes(Option<&'a str>), // config
    No,
}

pub trait Compiler<M: Copy + Eq + Hash, R: for<'de> Deserialize<'de> + Serialize> {
    fn compile(&self, input: &str, mode: M, minify: &ShouldMinify) -> Result<R, String>;
    fn set_cache(&mut self, cache: Arc<GenericCache<String, R>>);
    fn write_cache(&self) -> Result<(), String>;
}
