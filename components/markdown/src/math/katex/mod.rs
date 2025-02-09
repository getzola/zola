use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use twox_hash::XxHash64;

use crate::cache::GenericCache;

use super::{Compiler, ShouldMinify};

pub type KatexCache = GenericCache<String, String>;

pub struct KatexCompiler {
    pub cache: Option<Arc<KatexCache>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KatexRenderMode {
    Inline,
    Display,
}

impl KatexCompiler {
    pub fn new() -> Self {
        Self { cache: None }
    }
}

impl Compiler<KatexRenderMode, String> for KatexCompiler {
    fn set_cache(&mut self, cache: Arc<KatexCache>) {
        self.cache = Some(cache);
    }

    fn write_cache(&self) -> Result<(), String> {
        if let Some(cache) = self.cache.as_ref() {
            cache.write().map_err(|e| e.to_string())?
        }
        Ok(())
    }

    fn compile(
        &self,
        tex: &str,
        mode: KatexRenderMode,
        minify: &ShouldMinify,
    ) -> Result<String, String> {
        let mut opts = katex::Opts::builder();

        match mode {
            KatexRenderMode::Inline => opts.display_mode(false),
            KatexRenderMode::Display => opts.display_mode(true),
        };

        let opts = opts.build().map_err(|e| e.to_string())?;
        // Generate cache key
        let key = {
            let mut hasher = XxHash64::with_seed(42);
            tex.hash(&mut hasher);
            mode.hash(&mut hasher);
            minify.hash(&mut hasher);
            format!("{:x}", hasher.finish())
        };

        if let Some(entry) = self.cache.as_ref().and_then(|e| e.get(&key)) {
            return Ok(entry.clone());
        }

        let rendered = katex::render_with_opts(tex, &opts).map_err(|e| e.to_string())?;

        if let Some(cache) = self.cache.as_ref() {
            cache.insert(key, rendered.clone());
        }

        Ok(rendered)
    }
}
