use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use config::{BoolWithPath, ImageFormat};
use errors::{Context, Error};
use libs::pulldown_cmark::CowStr;
use twox_hash::XxHash64;

use super::{MathCache, MathCompiler, MathRenderMode};
use crate::Result;

pub struct KatexCompiler {
    cache: Option<Arc<MathCache>>,
    addon: Option<String>,
}

impl KatexCompiler {
    pub fn new(addon: Option<String>) -> Self {
        Self { cache: None, addon }
    }
}

impl MathCompiler for KatexCompiler {
    fn set_cache(&mut self, cache: Arc<MathCache>) {
        self.cache = Some(cache);
    }

    fn write_cache(&self) -> Result<()> {
        if let Some(ref cache) = self.cache {
            cache.write().context("Failed to write KaTeX cache")?;
        }
        Ok(())
    }

    fn compile(
        &self,
        tex: &str,
        mode: MathRenderMode,
        _format: ImageFormat,
        minify: &BoolWithPath,
    ) -> Result<String> {
        let tex: CowStr = if let Some(addon) = self.addon.as_ref() {
            CowStr::Boxed(format!("{}{}", tex, addon).into())
        } else {
            CowStr::Borrowed(tex)
        };
        let mut opts = katex::Opts::builder();

        match mode {
            MathRenderMode::Inline => opts.display_mode(false),
            MathRenderMode::Display => opts.display_mode(true),
            MathRenderMode::Raw => return Err(Error::msg("Raw mode is not supported by KaTeX")),
        };

        let opts = opts.build().map_err(|e| Error::msg(e.to_string()))?;
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

        let rendered = katex::render_with_opts(&tex, &opts)
            .map_err(|e| Error::msg(format!("Failed to render KaTeX: {}", e)))?;

        if let Some(cache) = self.cache.as_ref() {
            cache.insert(key, rendered.clone());
        }

        Ok(rendered)
    }
}
