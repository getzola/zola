use std::{hash::Hash, sync::Arc};

use crate::cache::GenericCache;
use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathRenderMode {
    Inline,
    Display,
    Raw,
}

pub mod katex;
pub mod svgo;
pub mod typst;

pub trait MathCompiler {
    fn compile(
        &self,
        input: &str,
        mode: MathRenderMode,
        format: config::ImageFormat,
        svgo: &config::BoolWithPath,
    ) -> Result<String>;
    fn raw_extensions(&self) -> &'static [&'static str] {
        &[]
    }
    fn set_cache(&mut self, cache: Arc<GenericCache<String, String>>);
    fn write_cache(&self) -> Result<()>;
}

pub type MathCache = GenericCache<String, String>;
