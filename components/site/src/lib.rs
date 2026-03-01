pub mod feeds;
pub mod link_checking;
mod md_render;
mod minify;
mod queue;
pub mod sass;
pub mod sitemap;
pub mod tpls;

use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;

use rayon::prelude::*;
use tera::Tera;
use walkdir::{DirEntry, WalkDir};

use crate::queue::Queue;
use config::{Config, IndexFormat, get_config};
use content::{Library, Page, Section, Taxonomy};
use errors::{Result, anyhow, bail};
use relative_path::RelativePathBuf;
use render::{RenderCache, Renderer};
use templates::load_tera;
use utils::fs::{
    clean_site_output_folder, copy_directory, copy_file_if_needed, create_directory, create_file,
};
use utils::net::get_available_port;
use utils::types::InsertAnchor;

pub static SITE_CONTENT: LazyLock<Arc<RwLock<HashMap<RelativePathBuf, String>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Where are we building the site
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BuildMode {
    /// On the filesystem -> `zola build`, The path is the `output_path`
    Disk,
    /// In memory for the content -> `zola serve`
    Memory,
    /// Both on the filesystem and in memory
    Both,
}

#[derive(Debug)]
pub struct Site {
    /// The base path of the zola site
    pub base_path: PathBuf,
    /// The parsed config for the site
    pub config: Config,
    pub tera: Tera,
    imageproc: Arc<Mutex<imageproc::Processor>>,
    // the live reload port to be used if there is one
    pub live_reload: Option<u16>,
    pub output_path: PathBuf,
    content_path: PathBuf,
    pub sass_path: PathBuf,
    pub static_path: PathBuf,
    pub templates_path: PathBuf,
    pub taxonomies: Vec<Taxonomy>,
    /// A map of all .md files (section and pages) and their permalink
    /// We need that if there are relative links in the content that need to be resolved
    pub permalinks: HashMap<String, String>,
    /// Contains all pages and sections of the site
    pub library: Arc<Library>,
    /// Pre-serialized render cache
    pub cache: Arc<RenderCache>,
    /// Whether to load draft pages
    include_drafts: bool,
    build_mode: BuildMode,
    /// Whether to check external links
    check_external_links: bool,
}

impl Site {
    /// Parse a site at the given path. Defaults to the current dir
    /// Passing in a path is used in tests and when --root argument is passed
    pub fn new<P: AsRef<Path>, P2: AsRef<Path>>(path: P, config_file: P2) -> Result<Site> {
        let path = path.as_ref();
        let config_file = config_file.as_ref();
        let mut config = get_config(&path.join(config_file))?;

        if let Some(theme) = config.theme.clone() {
            // Grab data from the extra section of the theme
            config.merge_with_theme(path.join("themes").join(&theme).join("theme.toml"), &theme)?;
        }

        let tera = load_tera(path, &config)?;

        let content_path = path.join("content");
        let sass_path = path.join("sass");
        let static_path = path.join("static");
        let templates_path = path.join("templates");
        let imageproc = imageproc::Processor::new(path.to_path_buf(), &config);
        let output_path = path.join(config.output_dir.clone());

        let site = Site {
            base_path: path.to_path_buf(),
            config,
            tera,
            imageproc: Arc::new(Mutex::new(imageproc)),
            live_reload: None,
            output_path,
            content_path,
            sass_path,
            static_path,
            templates_path,
            taxonomies: Vec::new(),
            permalinks: HashMap::new(),
            include_drafts: false,
            // We will allocate it properly later on
            library: Arc::new(Library::default()),
            cache: Arc::new(RenderCache::default()),
            build_mode: BuildMode::Disk,
            check_external_links: true,
        };

        Ok(site)
    }

    /// Enable some `zola serve` related options
    pub fn enable_serve_mode(&mut self, build_mode: BuildMode) {
        SITE_CONTENT.write().unwrap().clear();
        self.config.enable_serve_mode();
        self.build_mode = build_mode;
    }

    /// Set the site to load the drafts.
    /// Needs to be called before loading it
    pub fn include_drafts(&mut self) {
        self.include_drafts = true;
    }

    /// Set the site checker to skip external links check.
    pub fn skip_external_links_check(&mut self) {
        self.check_external_links = false;
    }

    /// The index sections are ALWAYS at those paths
    /// There are one index section for the default language + 1 per language
    fn index_section_paths(&self) -> Vec<(PathBuf, Option<&str>)> {
        let mut res = vec![(self.content_path.join("_index.md"), None)];
        for (code, _) in self.config.other_languages() {
            res.push((self.content_path.join(format!("_index.{}.md", code)), Some(code)));
        }
        res
    }

    /// We avoid the port the server is going to use as it's not bound yet
    /// when calling this function and we could end up having tried to bind
    /// both http and websocket server to the same port
    pub fn enable_live_reload(&mut self, interface: IpAddr, port_to_avoid: u16) {
        self.live_reload = get_available_port(interface, port_to_avoid);
    }

    /// Only used in `zola serve` to re-use the initial websocket port
    pub fn enable_live_reload_with_port(&mut self, live_reload_port: u16) {
        self.live_reload = Some(live_reload_port);
    }

    /// Reloads the templates and rebuild the site without re-markdown the Markdown.
    pub fn reload_templates(&mut self) -> Result<()> {
        self.tera = load_tera(&self.base_path, &self.config)?;
        tpls::register_early_global_fns(self);
        // TODO: be smarter than that, no need to recompile sass for example
        self.build()
    }

    pub fn set_base_url(&mut self, base_url: String) {
        self.config.base_url = base_url;
        let mut imageproc = self.imageproc.lock().expect("Couldn't lock imageproc (set_base_url)");
        imageproc.set_base_url(&self.config);
    }

    pub fn set_output_path<P: AsRef<Path>>(&mut self, path: P) {
        self.output_path = path.as_ref().to_path_buf();
    }

    pub fn minify(&mut self) {
        self.config.minify_html = true;
    }

    /// Reads all .md files in the `content` directory and create pages/sections
    /// out of them
    pub fn load(&mut self) -> Result<()> {
        self.library = Arc::new(Library::new(&self.config));

        // not the most elegant loop, but this is necessary to use skip_current_dir
        // which we can only decide to use after we've deserialised the section
        // so it's kinda necessary
        let mut dir_walker =
            WalkDir::new(self.base_path.join("content")).follow_links(true).into_iter();
        let mut allowed_index_filenames: HashSet<_> = self
            .config
            .other_languages()
            .keys()
            .map(|code| format!("_index.{}.md", code))
            .collect();
        allowed_index_filenames.insert("_index.md".to_string());

        // We will insert colocated pages (those with an index.md filename)
        // at the end to detect pages that are actually errors:
        // when there is both a _index.md and index.md in the same folder
        let mut page_paths = Vec::new();
        let mut sections = HashSet::new();

        loop {
            let entry: DirEntry = match dir_walker.next() {
                None => break,
                Some(Err(_)) => continue,
                Some(Ok(entry)) => entry,
            };
            let path = entry.path();
            let file_name = match path.file_name() {
                None => continue,
                Some(name) => name.to_str().unwrap(),
            };

            // ignore excluded content
            if let Some(gs) = &self.config.ignored_content_globset
                && gs.is_match(path)
            {
                continue;
            }

            // we process a section when we encounter the dir
            // so we can process it before any of the pages
            // therefore we should skip the actual file to avoid duplication
            if file_name.starts_with("_index.") {
                continue;
            }

            // skip hidden files and non md files
            if !path.is_dir() && (!file_name.ends_with(".md") || file_name.starts_with('.')) {
                continue;
            }

            // is it a section or not?
            if path.is_dir() {
                // if we are processing a section we have to collect
                // index files for all languages and process them simultaneously
                // before any of the pages
                let index_files = WalkDir::new(path)
                    .follow_links(true)
                    .max_depth(1)
                    .into_iter()
                    .filter_map(|e| match e {
                        Err(_) => None,
                        Ok(f) => {
                            let path_str = f.path().file_name().unwrap().to_str().unwrap();
                            // https://github.com/getzola/zola/issues/1244
                            if f.path().is_file() && allowed_index_filenames.contains(path_str) {
                                Some(f)
                            } else {
                                None
                            }
                        }
                    })
                    .collect::<Vec<DirEntry>>();

                for index_file in index_files {
                    let section =
                        Section::from_file(index_file.path(), &self.config, &self.base_path)?;
                    sections.insert(section.components.join("/"));

                    // if the section is drafted we can skip the entire dir
                    if section.meta.draft && !self.include_drafts {
                        dir_walker.skip_current_dir();
                        continue;
                    }

                    self.add_section(section, false)?;
                }
            } else {
                page_paths.push(path.to_path_buf());
            }
        }
        let results: Vec<(PathBuf, Result<Page>)> = page_paths
            .par_iter()
            .map(|p| (p.clone(), Page::from_file(p, &self.config, &self.base_path)))
            .collect();
        let (pages, errors): (Vec<_>, Vec<_>) = results.into_iter().partition(|(_, r)| r.is_ok());

        if !errors.is_empty() {
            let mut errors: Vec<_> = errors.into_iter().map(|(p, r)| (p, r.unwrap_err())).collect();
            // sort by path for deterministic output
            errors.sort_by(|(a, _), (b, _)| a.cmp(b));

            let msg = errors
                .iter()
                .map(|(p, e)| format!("  - {}: {e}", p.display()))
                .collect::<Vec<_>>()
                .join("\n");
            return Err(anyhow!("Failed to parse {} page(s):\n{msg}", errors.len()));
        }

        let pages: Vec<Page> = pages.into_iter().map(|(_, r)| r.unwrap()).collect();
        self.create_default_index_sections()?;

        for page in pages {
            // should we skip drafts?
            if page.meta.draft && !self.include_drafts {
                continue;
            }

            // We are only checking it on load and not in add_page since we have access to
            // all the components there.
            if page.file.filename == "index.md" {
                let is_invalid = match page.components.last() {
                    Some(_) => sections.contains(&page.components.join("/")),
                    // content/index.md is always invalid, but content/colocated/index.md is ok
                    None => page.file.colocated_path.is_none(),
                };

                if is_invalid {
                    bail!(
                        "We can't have a page called `index.md` in the same folder as an index section in {:?}",
                        page.file.parent
                    );
                }
            }

            self.add_page(page, false)?;
        }

        let collisions = self.library.find_path_collisions(&self.config);
        if !collisions.is_empty() {
            let mut msg = String::from("Found path collisions:\n");
            for c in collisions {
                let row = format!("- {c}");
                msg.push_str(&row);
            }
            return Err(anyhow!(msg));
        }

        self.cache = Arc::new(RenderCache::new(&self.config));
        // taxonomy Tera fns are loaded in `register_early_global_fns`
        // so we do need to populate it first.
        self.populate_taxonomies()?;
        tpls::register_early_global_fns(self);
        self.populate_sections();
        self.render_markdown()?;
        Arc::make_mut(&mut self.library).fill_backlinks();
        Arc::make_mut(&mut self.cache).build(&self.library, &self.taxonomies, &self.tera);
        tpls::register_tera_global_fns(self);

        // Needs to be done after rendering markdown as we only get the anchors at that point
        let internal_link_messages = link_checking::check_internal_links_with_anchors(self);

        // log any broken internal links and error out if needed
        if !internal_link_messages.is_empty() {
            let messages: Vec<String> = internal_link_messages
                .iter()
                .enumerate()
                .map(|(i, msg)| format!("  {}. {}", i + 1, msg))
                .collect();
            let msg = format!(
                "Found {} broken internal anchor link(s)\n{}",
                messages.len(),
                messages.join("\n")
            );
            match self.config.link_checker.internal_level {
                config::LinkCheckerLevel::Warn => log::warn!("{msg}"),
                config::LinkCheckerLevel::Error => return Err(anyhow!(msg)),
            }
        }

        // check external links, log the results, and error out if needed
        if self.config.is_in_check_mode() && self.check_external_links {
            let external_link_messages = link_checking::check_external_links(self);
            if !external_link_messages.is_empty() {
                let messages: Vec<String> = external_link_messages
                    .iter()
                    .enumerate()
                    .map(|(i, msg)| format!("  {}. {}", i + 1, msg))
                    .collect();
                let msg = format!(
                    "Found {} broken external link(s)\n{}",
                    messages.len(),
                    messages.join("\n")
                );
                match self.config.link_checker.external_level {
                    config::LinkCheckerLevel::Warn => log::warn!("{msg}"),
                    config::LinkCheckerLevel::Error => return Err(anyhow!(msg)),
                }
            }
        }

        Ok(())
    }

    /// Insert a default index section for each language if necessary so we don't need to create
    /// a _index.md to render the index page at the root of the site
    pub fn create_default_index_sections(&mut self) -> Result<()> {
        let mut missing_sections = Vec::new();
        for (index_path, lang) in self.index_section_paths() {
            if let Some(index_section) = self.library.sections.get(&index_path)
                && self.config.build_search_index
                && !index_section.meta.in_search_index
            {
                bail!(
                    "You have enabled search in the config but disabled it in the index section: \
                    either turn off the search in the config or remove `in_search_index = true` from the \
                    section front-matter."
                )
            }
            // Not in else because of borrow checker
            if !self.library.sections.contains_key(&index_path) {
                missing_sections.push((index_path, lang.map(|l| l.to_string())));
            }
        }

        if missing_sections.is_empty() {
            return Ok(());
        }

        let library = Arc::make_mut(&mut self.library);
        for (index_path, lang) in missing_sections {
            if !library.sections.contains_key(&index_path) {
                let mut index_section = Section::default();
                index_section.file.parent = self.content_path.clone();
                index_section.file.filename =
                    index_path.file_name().unwrap().to_string_lossy().to_string();
                if let Some(ref l) = lang {
                    index_section.file.name = format!("_index.{}", l);
                    index_section.path = format!("{}/", l);
                    index_section.permalink = self.config.make_permalink(l);
                    let filename = format!("_index.{}.md", l);
                    index_section.file.path = self.content_path.join(&filename);
                    index_section.file.relative = filename;
                    index_section.file.canonical = self.content_path.join(format!("_index.{}", l));
                } else {
                    index_section.file.name = "_index".to_string();
                    index_section.permalink = self.config.make_permalink("");
                    index_section.file.path = self.content_path.join("_index.md");
                    index_section.file.relative = "_index.md".to_string();
                    index_section.file.canonical = self.content_path.join("_index");
                    index_section.path = "/".to_string();
                }
                index_section.lang = index_section.file.find_language(
                    &self.config.default_language,
                    &self.config.other_languages_codes(),
                )?;
                library.insert_section(index_section);
            }
        }

        Ok(())
    }

    /// Render the markdown of all pages/sections
    /// Used in a build and in `serve` if a template has changed
    pub fn render_markdown(&mut self) -> Result<()> {
        // Another silly thing needed to not borrow &self in parallel and
        // make the borrow checker happy
        let permalinks = &self.permalinks;
        let tera = &self.tera;
        let config = &self.config;

        // This is needed in the first place because of silly borrow checker
        let mut pages_insert_anchors = HashMap::new();
        for (_, p) in &self.library.pages {
            pages_insert_anchors.insert(
                p.file.path.clone(),
                self.find_parent_section_insert_anchor(&p.file.parent.clone(), &p.lang),
            );
        }

        // For rendering content, we do not need the real library since it's not going to be filled
        let dummy_library = Library::default();
        let renderer = Renderer { tera, config, library: &dummy_library, cache: &self.cache };

        let library = Arc::make_mut(&mut self.library);
        library
            .pages
            .values_mut()
            .collect::<Vec<_>>()
            .par_iter_mut()
            .map(|page| {
                let insert_anchor = pages_insert_anchors[&page.file.path];
                md_render::render_page(
                    page,
                    renderer.clone(),
                    permalinks,
                    tera,
                    config,
                    insert_anchor,
                )
            })
            .collect::<Result<()>>()?;

        library
            .sections
            .values_mut()
            .collect::<Vec<_>>()
            .par_iter_mut()
            .map(|section| {
                md_render::render_section(section, renderer.clone(), permalinks, tera, config)
            })
            .collect::<Result<()>>()?;

        Ok(())
    }

    /// Add a page to the site
    /// The `render` parameter is used in the serve command with --fast, when rebuilding a page.
    pub fn add_page(&mut self, mut page: Page, render_md: bool) -> Result<()> {
        for taxa_name in page.meta.taxonomies.keys() {
            if !self.config.has_taxonomy(taxa_name, &page.lang) {
                bail!(
                    "Page `{}` has taxonomy `{}` which is not defined in config.toml",
                    page.file.path.display(),
                    taxa_name
                );
            }
        }

        self.permalinks.insert(page.file.relative.clone(), page.permalink.clone());
        if render_md {
            let insert_anchor =
                self.find_parent_section_insert_anchor(&page.file.parent, &page.lang);
            md_render::render_page(
                &mut page,
                self.renderer(),
                &self.permalinks,
                &self.tera,
                &self.config,
                insert_anchor,
            )?;
        }

        let library = Arc::make_mut(&mut self.library);
        library.pages.remove(&page.file.path);
        library.insert_page(page);

        Ok(())
    }

    /// Adds a page to the site and render it
    /// Only used in `zola serve --fast`
    pub fn add_and_render_page(&mut self, path: &Path) -> Result<()> {
        let page = Page::from_file(path, &self.config, &self.base_path)?;
        self.add_page(page, true)?;
        let page = self.library.pages.get(path).unwrap();
        Queue::single_page(self, page).process()
    }

    /// Add a section to the site
    /// The `render` parameter is used in the serve command with --fast, when rebuilding a page.
    pub fn add_section(&mut self, mut section: Section, render_md: bool) -> Result<()> {
        self.permalinks.insert(section.file.relative.clone(), section.permalink.clone());
        if render_md {
            md_render::render_section(
                &mut section,
                self.renderer(),
                &self.permalinks,
                &self.tera,
                &self.config,
            )?;
        }
        let library = Arc::make_mut(&mut self.library);
        library.sections.remove(&section.file.path);
        library.insert_section(section);

        Ok(())
    }

    /// Adds a section to the site and render it
    /// Only used in `zola serve --fast`
    pub fn add_and_render_section(&mut self, path: &Path) -> Result<()> {
        let section = Section::from_file(path, &self.config, &self.base_path)?;
        let old_meta = self.library.sections.get(path).map(|s| s.meta.clone());
        self.add_section(section, true)?;
        self.populate_sections();
        let section = self.library.sections.get(path).unwrap();
        let render_pages = old_meta.map(|m| section.needs_pages_render(&m)).unwrap_or(true);
        Queue::single_section(self, section, render_pages).process()
    }

    /// Finds the insert_anchor for the parent section of the directory at `path`.
    /// Defaults to the global setting if no parent section found
    pub fn find_parent_section_insert_anchor(
        &self,
        parent_path: &Path,
        lang: &str,
    ) -> InsertAnchor {
        let parent = if lang != self.config.default_language {
            parent_path.join(format!("_index.{}.md", lang))
        } else {
            parent_path.join("_index.md")
        };
        self.library
            .sections
            .get(&parent)
            .and_then(|s| s.meta.insert_anchor_links)
            .unwrap_or(self.config.markdown.insert_anchor_links)
    }

    /// Find out the direct subsections of each subsection if there are some
    /// as well as the pages for each section
    pub fn populate_sections(&mut self) {
        let library = Arc::make_mut(&mut self.library);
        library.populate_sections(&self.config, &self.content_path);
    }

    /// Find all the tags and categories if it's asked in the config
    pub fn populate_taxonomies(&mut self) -> Result<()> {
        self.taxonomies = self.library.find_taxonomies(&self.config);
        Ok(())
    }

    /// Rebuild the render cache (needed after modifying library or taxonomies)
    pub fn rebuild_cache(&mut self) {
        let mut cache = RenderCache::new(&self.config);
        cache.build(&self.library, &self.taxonomies, &self.tera);
        self.cache = Arc::new(cache);
    }

    /// Create a Renderer for this site
    fn renderer(&self) -> Renderer<'_> {
        Renderer::new(&self.tera, &self.config, &self.library, &self.cache)
    }

    /// Inject live reload script tag if in live reload mode
    fn inject_livereload(&self, mut html: String) -> String {
        if let Some(port) = self.live_reload {
            let script =
                format!(r#"<script src="/livereload.js?port={}&amp;mindelay=10"></script>"#, port,);
            if let Some(index) = html.rfind("</body>") {
                html.insert_str(index, &script);
            } else {
                html.push_str(&script);
            }
        }

        html
    }

    /// Copy the main `static` folder and the theme `static` folder if a theme is used
    pub fn copy_static_directories(&self) -> Result<()> {
        // The user files will overwrite the theme files
        if let Some(ref theme) = self.config.theme {
            copy_directory(
                &self.base_path.join("themes").join(theme).join("static"),
                &self.output_path,
                false,
                None,
            )?;
        }
        // We're fine with missing static folders
        if self.static_path.exists() {
            copy_directory(
                &self.static_path,
                &self.output_path,
                self.config.hard_link_static,
                self.config.ignored_static_globset.as_ref(),
            )?;
        }

        Ok(())
    }

    pub fn num_img_ops(&self) -> usize {
        let imageproc = self.imageproc.lock().expect("Couldn't lock imageproc (num_img_ops)");
        imageproc.num_img_ops()
    }

    pub fn process_images(&self) -> Result<()> {
        let mut imageproc =
            self.imageproc.lock().expect("Couldn't lock imageproc (process_images)");
        imageproc.prune()?;
        imageproc.do_process(&self.config.image_encoders)
    }

    /// Deletes the `public` directory if it exists and the `preserve_dotfiles_in_output` option is set to false,
    /// or if set to true: its contents except for the dotfiles at the root level.
    pub fn clean(&self) -> Result<()> {
        clean_site_output_folder(&self.output_path, self.config.preserve_dotfiles_in_output)
    }

    fn copy_assets(&self, parent: &Path, assets: &[impl AsRef<Path>], dest: &Path) -> Result<()> {
        for asset in assets {
            let asset_path = asset.as_ref();
            copy_file_if_needed(
                asset_path,
                &dest.join(
                    asset_path.strip_prefix(parent).expect("Couldn't get filename from page asset"),
                ),
                self.config.hard_link_static,
            )?;
        }

        Ok(())
    }

    /// Deletes the `public` directory (only for `zola build`) and builds the site
    pub fn build(&self) -> Result<()> {
        let mut start = Instant::now();
        // Do not clean on `zola serve` otherwise we end up copying assets all the time
        if self.build_mode == BuildMode::Disk {
            self.clean()?;
        }
        start = log_time(start, "Cleaned folder");

        // Generate/move all assets before markdown any content
        if let Some(ref theme) = self.config.theme {
            let theme_path = self.base_path.join("themes").join(theme);
            if theme_path.join("sass").exists() {
                sass::compile_sass(&theme_path, &self.output_path)?;
                start = log_time(start, "Compiled theme Sass");
            }
        }

        if self.config.compile_sass {
            sass::compile_sass(&self.base_path, &self.output_path)?;
            start = log_time(start, "Compiled own Sass");
        }

        if self.config.build_search_index {
            self.build_search_index()?;
            start = log_time(start, "Built search index");
        }
        self.render_themes_css()?;
        start = log_time(start, "Rendered themes css");

        let queue = Queue::full_build(self);
        queue.process()?;

        // We process images at the end as we might have picked up images to process from markdown
        // or from templates
        self.process_images()?;
        start = log_time(start, "Processed images");
        // Processed images will be in static so the last step is to copy it
        self.copy_static_directories()?;
        log_time(start, "Copied static dir");

        Ok(())
    }

    pub fn render_themes_css(&self) -> Result<()> {
        let themes = self
            .config
            .markdown
            .highlighting
            .as_ref()
            .map(|x| x.generate_themes_css())
            .unwrap_or_default();
        if !themes.is_empty() {
            create_directory(&self.static_path)?;
        }

        for (filename, content) in themes {
            let p = self.static_path.join(filename);
            if !p.exists() {
                create_file(&p, content)?;
            }
        }

        Ok(())
    }

    fn index_for_lang(&self, lang: &str) -> Result<()> {
        let path = &self.output_path.join(self.config.search.index_format.filename(lang));
        let content = match &self.config.search.index_format {
            IndexFormat::ElasticlunrJavascript | IndexFormat::ElasticlunrJson => {
                search::build_elasticlunr(lang, &self.library, &self.config)?
            }
            IndexFormat::FuseJson | IndexFormat::FuseJavascript => {
                search::build_fuse(lang, &self.library, &self.config.search)?
            }
        };
        create_file(
            path,
            match self.config.search.index_format {
                IndexFormat::ElasticlunrJson | IndexFormat::FuseJson => content,
                IndexFormat::ElasticlunrJavascript | IndexFormat::FuseJavascript => {
                    format!("window.searchIndex = {}", content)
                }
            },
        )
    }

    pub fn build_search_index(&self) -> Result<()> {
        create_directory(&self.output_path)?;
        // TODO: add those to the SITE_CONTENT map

        // index first
        self.index_for_lang(&self.config.default_language)?;

        for (code, language) in &self.config.other_languages() {
            if code != &self.config.default_language && language.build_search_index {
                self.index_for_lang(code)?;
            }
        }

        match self.config.search.index_format {
            IndexFormat::ElasticlunrJavascript | IndexFormat::ElasticlunrJson => {
                // then elasticlunr.min.js
                create_file(&self.output_path.join("elasticlunr.min.js"), search::ELASTICLUNR_JS)?;
            }
            _ => {}
        }

        Ok(())
    }

    pub(crate) fn make_feed_url(&self, base_path: Option<&PathBuf>, feed_filename: &str) -> String {
        if let Some(base) = base_path {
            self.config
                .make_permalink(&base.join(feed_filename).to_string_lossy().replace('\\', "/"))
        } else {
            self.config.make_permalink(feed_filename)
        }
    }
}

fn log_time(start: Instant, message: &str) -> Instant {
    let now = Instant::now();
    log::debug!("{} took {}ms", message, now.duration_since(start).as_millis());
    now
}
