use config::{BoolWithPath, ImageFormat};
use errors::{Context, Error};
use std::sync::Arc;
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    io::Write,
    path::PathBuf,
    sync::Mutex,
};
use twox_hash::XxHash64;
use typst::layout::PagedDocument;

use typst::{
    diag::{eco_format, FileError, FileResult, PackageError, PackageResult},
    foundations::{Bytes, Datetime, Label},
    syntax::{package::PackageSpec, FileId, Source},
    text::{Font, FontBook},
    utils::LazyHash,
    Library, World,
};

mod format;
mod templates;

pub use format::*;

use super::svgo::Svgo;
use super::{MathCache, MathCompiler, MathRenderMode};
use crate::context::CACHE_DIR;
use crate::Result;

fn fonts() -> Vec<Font> {
    typst_assets::fonts()
        .flat_map(|bytes| {
            let buffer = Bytes::new(bytes);
            let face_count = ttf_parser::fonts_in_collection(&buffer).unwrap_or(1);
            (0..face_count).map(move |face| {
                Font::new(buffer.clone(), face).expect("failed to load font from typst-assets")
            })
        })
        .collect()
}

/// Fake file
///
/// This is a fake file which wrap the real content takes from the md math block
pub struct TypstFile {
    bytes: Bytes,

    source: Option<Source>,
}

impl TypstFile {
    fn source(&mut self, id: FileId) -> FileResult<Source> {
        let source = match &self.source {
            Some(source) => source,
            None => {
                let contents =
                    std::str::from_utf8(&self.bytes).map_err(|_| FileError::InvalidUtf8)?;
                let source = Source::new(id, contents.into());
                self.source.insert(source)
            }
        };
        Ok(source.clone())
    }
}

/// Compiler
///
/// This is the compiler which has all the necessary fields except the source
pub struct TypstCompiler {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    packages_cache_path: PathBuf,
    files: Mutex<HashMap<FileId, TypstFile>>,
    render_cache: Option<Arc<MathCache>>,
    addon: Option<String>,
    styles: Option<String>,
}

impl TypstCompiler {
    pub fn new(
        base_cache_path: Option<PathBuf>,
        addon: Option<String>,
        styles: Option<String>,
    ) -> Self {
        let fonts = fonts();

        Self {
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(FontBook::from_fonts(&fonts)),
            fonts,
            packages_cache_path: base_cache_path
                .unwrap_or(CACHE_DIR.to_path_buf())
                .join("packages"),
            files: Mutex::new(HashMap::new()),
            render_cache: None,
            addon,
            styles,
        }
    }

    pub fn wrap_source(&self, source: impl Into<String>) -> WrapSource<'_> {
        WrapSource {
            compiler: self,
            source: Source::detached(source),
            time: time::OffsetDateTime::now_local().unwrap_or(time::OffsetDateTime::now_utc()),
        }
    }

    /// Get the package directory or download if not exists
    fn package(&self, package: &PackageSpec) -> PackageResult<PathBuf> {
        let package_subdir = format!("{}/{}/{}", package.namespace, package.name, package.version);
        let path = self.packages_cache_path.join(package_subdir);

        if path.exists() {
            return Ok(path);
        }

        // Download the package
        let package_url = format!(
            "https://packages.typst.org/{}/{}-{}.tar.gz",
            package.namespace, package.name, package.version
        );

        let mut response = libs::reqwest::blocking::get(package_url).map_err(|e| {
            PackageError::NetworkFailed(Some(eco_format!(
                "Failed to download package {}: {}",
                package.name,
                e
            )))
        })?;

        let mut compressed = Vec::new();
        response.copy_to(&mut compressed).map_err(|e| {
            PackageError::NetworkFailed(Some(eco_format!(
                "Failed to save package {}: {}",
                package.name,
                e
            )))
        })?;

        let mut decompressed = Vec::new();
        let mut decoder = flate2::write::GzDecoder::new(decompressed);
        decoder.write_all(&compressed).map_err(|e| {
            PackageError::MalformedArchive(Some(eco_format!(
                "Failed to decompress package {}: {}",
                package.name,
                e
            )))
        })?;
        decoder.try_finish().map_err(|e| {
            PackageError::MalformedArchive(Some(eco_format!(
                "Failed to decompress package {}: {}",
                package.name,
                e
            )))
        })?;
        decompressed = decoder.finish().map_err(|e| {
            PackageError::MalformedArchive(Some(eco_format!(
                "Failed to decompress package {}: {}",
                package.name,
                e
            )))
        })?;

        let mut archive = tar::Archive::new(decompressed.as_slice());
        archive.unpack(&path).map_err(|e| {
            std::fs::remove_dir_all(&path).ok();
            PackageError::MalformedArchive(Some(eco_format!(
                "Failed to unpack package {}: {}",
                package.name,
                e
            )))
        })?;

        Ok(path)
    }

    // Weird pattern because mapping a MutexGuard is not stable yet.
    fn file<T>(&self, id: FileId, map: impl FnOnce(&mut TypstFile) -> T) -> FileResult<T> {
        let mut files = self.files.lock().unwrap();
        if let Some(entry) = files.get_mut(&id) {
            return Ok(map(entry));
        }
        // `files` must stay locked here so we don't download the same package multiple times.
        // TODO proper multithreading, maybe with typst-kit.

        'x: {
            if let Some(package) = id.package() {
                let package_dir = self.package(package)?;
                let Some(path) = id.vpath().resolve(&package_dir) else {
                    break 'x;
                };
                let contents =
                    std::fs::read(&path).map_err(|error| FileError::from_io(error, &path))?;
                let entry = files
                    .entry(id)
                    .or_insert(TypstFile { bytes: Bytes::new(contents), source: None });
                return Ok(map(entry));
            }
        }

        Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
    }
}

impl MathCompiler for TypstCompiler {
    fn set_cache(&mut self, cache: Arc<MathCache>) {
        self.render_cache = Some(cache);
    }

    fn raw_extensions(&self) -> &'static [&'static str] {
        &["typ", "typst"]
    }

    fn write_cache(&self) -> Result<()> {
        if let Some(ref render_cache) = self.render_cache {
            render_cache.write().context("Failed to write typst cache")?;
        }
        Ok(())
    }

    fn compile(
        &self,
        source: &str,
        mode: MathRenderMode,
        format: ImageFormat,
        minify: &BoolWithPath,
    ) -> Result<String> {
        // Prepare source based on mode
        let source = match mode {
            MathRenderMode::Display => templates::display_math(&source, self.addon.as_deref()),
            MathRenderMode::Inline => templates::inline_math(&source, self.addon.as_deref()),
            MathRenderMode::Raw => templates::raw(&source, self.addon.as_deref()),
        };

        // Generate cache key
        let key = {
            let mut hasher = XxHash64::with_seed(42);
            source.hash(&mut hasher);
            mode.hash(&mut hasher);
            minify.hash(&mut hasher);
            format.hash(&mut hasher);
            format!("{:x}", hasher.finish())
        };

        // Check cache first
        if let Some(entry) = self.render_cache.as_ref().and_then(|e| e.get(&key)) {
            return Ok(entry.clone());
        }

        // Compile the source
        let world = self.wrap_source(source);
        let document = typst::compile(&world);
        let warnings = document.warnings;
        let has_error = warnings.iter().any(|w| w.severity == typst::diag::Severity::Error);
        if has_error {
            return Err(Error::msg(format!("{:?}", warnings)));
        }

        let document: PagedDocument =
            document.output.map_err(|diags| Error::msg(format!("{:?}", diags)))?;
        let page = document.pages.first().ok_or(Error::msg("No pages found"))?;
        let image = match format {
            ImageFormat::Svg => {
                let svg = typst_svg::svg(page);

                // Minify if requested
                let minified = match minify {
                    BoolWithPath::True(config) => {
                        let svgo = Svgo::default();
                        svgo.minify(&svg, config.as_deref())
                            .map_err(|e| Error::msg(format!("Failed to minify SVG: {}", e)))?
                    }
                    BoolWithPath::False => svg,
                };

                // Get alignment (for inline mode)
                let align = match mode {
                    MathRenderMode::Inline => {
                        let query =
                            document.introspector.query_label(Label::construct("label".into()));
                        Some(
                            query
                                .map(|it| {
                                    let field = it.clone().field_by_name("value").unwrap();
                                    if let typst::foundations::Value::Length(value) = field {
                                        value.abs.to_pt()
                                    } else {
                                        0.0
                                    }
                                })
                                .unwrap_or(0.0),
                        )
                    }
                    MathRenderMode::Raw | MathRenderMode::Display => None,
                };

                format_svg(&minified, align, mode, self.styles.as_deref())
            }
            ImageFormat::Webp => {
                // let _pixmap = typst_render::render(page, 300.0);
                // TODO: svg2webp
                unimplemented!("WebP is not supported yet")
            }
        };

        // Cache and return
        if let Some(ref render_cache) = self.render_cache {
            render_cache.insert(key, image.clone());
        }

        Ok(image)
    }
}

/// Wrap source
///
/// This is a wrapper for the source which provides ref to the compiler
pub struct WrapSource<'a> {
    compiler: &'a TypstCompiler,
    source: Source,
    time: time::OffsetDateTime,
}

impl World for WrapSource<'_> {
    fn library(&self) -> &LazyHash<Library> {
        &self.compiler.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.compiler.book
    }

    fn main(&self) -> FileId {
        self.source.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            self.compiler.file(id, |file| file.source(id))?
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.compiler.file(id, |file| file.bytes.clone())
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.compiler.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        Some(Datetime::Date(self.time.date()))
    }
}
