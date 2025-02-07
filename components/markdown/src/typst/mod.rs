use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    io::Write,
    path::PathBuf,
    sync::Mutex,
};
use twox_hash::XxHash64;

use typst::{
    diag::{eco_format, FileError, FileResult, PackageError, PackageResult},
    foundations::{Bytes, Datetime, Label},
    syntax::{package::PackageSpec, FileId, Source},
    text::{Font, FontBook},
    utils::LazyHash,
    Library, World,
};

fn fonts() -> Vec<Font> {
    typst_assets::fonts()
        .flat_map(|bytes| {
            let buffer = Bytes::from_static(bytes);
            let face_count = ttf_parser::fonts_in_collection(&buffer).unwrap_or(1);
            (0..face_count).map(move |face| {
                Font::new(buffer.clone(), face).expect("failed to load font from typst-assets")
            })
        })
        .collect()
}

mod format;
mod svgo;
mod templates;

pub use format::*;
pub use svgo::*;

use crate::cache::GenericCache;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypstRenderMode {
    Display,
    Inline,
    Raw,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypstMinify<'a> {
    Yes(Option<&'a str>),
    No,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypstCacheEntry {
    content: String,
    align: Option<f64>,
}

pub type TypstCache = GenericCache<String, TypstCacheEntry>;

/// Compiler
///
/// This is the compiler which has all the necessary fields except the source
pub struct TypstCompiler {
    pub library: LazyHash<Library>,
    pub book: LazyHash<FontBook>,
    pub fonts: Vec<Font>,

    pub packages_cache: PathBuf,
    pub files: Mutex<HashMap<FileId, TypstFile>>,
    pub render_cache: Arc<TypstCache>,
}

impl TypstCompiler {
    pub fn new(cache: Arc<TypstCache>) -> Self {
        let fonts = fonts();

        Self {
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(FontBook::from_fonts(&fonts)),
            fonts,
            packages_cache: PathBuf::from(".cache/packages"),
            files: Mutex::new(HashMap::new()),
            render_cache: cache,
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
        let path = self.packages_cache.join(package_subdir);

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
                let entry =
                    files.entry(id).or_insert(TypstFile { bytes: contents.into(), source: None });
                return Ok(map(entry));
            }
        }

        Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
    }

    pub fn render(
        &self,
        source: &str,
        mode: TypstRenderMode,
        minify: TypstMinify,
    ) -> Result<(String, Option<f64>), String> {
        // Prepare source based on mode
        let source = match mode {
            TypstRenderMode::Display => templates::display_math(&source),
            TypstRenderMode::Inline => templates::inline_math(&source),
            TypstRenderMode::Raw => templates::raw(&source),
        };

        // Generate cache key
        let key = {
            let mut hasher = XxHash64::with_seed(42);
            source.hash(&mut hasher);
            mode.hash(&mut hasher);
            minify.hash(&mut hasher);
            format!("{:x}", hasher.finish())
        };

        // Check cache first
        if let Some(entry) = self.render_cache.get(&key) {
            return Ok((entry.content.clone(), entry.align));
        }

        // Compile the source
        let world = self.wrap_source(source);
        let document = typst::compile(&world);
        let warnings = document.warnings;

        if !warnings.is_empty() {
            return Err(format!("{:?}", warnings));
        }

        let document = document.output.map_err(|diags| format!("{:?}", diags))?;
        let page = document.pages.first().ok_or("no pages")?;
        let image = typst_svg::svg(page);

        // Minify if requested
        let minified = match minify {
            TypstMinify::Yes(config) => {
                let svgo = Svgo::default();
                svgo.minify(&image, config.as_deref())
                    .map_err(|e| format!("Failed to minify svg: {}", e))?
            }
            TypstMinify::No => image,
        };

        // Get alignment (for math modes)
        let align = if mode != TypstRenderMode::Raw {
            let query = document.introspector.query_label(Label::construct("label".into()));
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
        } else {
            None
        };

        // Cache and return
        self.render_cache.insert(
            key,
            TypstCacheEntry { content: minified.clone(), align: align.map(Some).unwrap_or(None) },
        );

        Ok((minified, align))
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
