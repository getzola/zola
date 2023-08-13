pub mod feed;
pub mod link_checking;
mod minify;
pub mod sass;
pub mod sitemap;
pub mod tpls;

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use libs::once_cell::sync::Lazy;
use libs::rayon::prelude::*;
use libs::tera::{Context, Tera};
use libs::walkdir::{DirEntry, WalkDir};

use config::{get_config, Config, IndexFormat};
use content::{Library, Page, Paginator, Section, Taxonomy};
use errors::{anyhow, bail, Result};
use libs::relative_path::RelativePathBuf;
use std::time::Instant;
use templates::{load_tera, render_redirect_template};
use utils::fs::{
    clean_site_output_folder, copy_directory, copy_file_if_needed, create_directory, create_file,
    ensure_directory_exists,
};
use utils::net::{get_available_port, is_external_link};
use utils::templates::{render_template, ShortcodeDefinition};
use utils::types::InsertAnchor;

pub static SITE_CONTENT: Lazy<Arc<RwLock<HashMap<RelativePathBuf, String>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Where are we building the site
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BuildMode {
    /// On the filesystem -> `zola build`, The path is the `output_path`
    Disk,
    /// In memory for the content -> `zola serve`
    Memory,
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
    pub static_path: PathBuf,
    pub taxonomies: Vec<Taxonomy>,
    /// A map of all .md files (section and pages) and their permalink
    /// We need that if there are relative links in the content that need to be resolved
    pub permalinks: HashMap<String, String>,
    /// Contains all pages and sections of the site
    pub library: Arc<RwLock<Library>>,
    /// Whether to load draft pages
    include_drafts: bool,
    build_mode: BuildMode,
    shortcode_definitions: HashMap<String, ShortcodeDefinition>,
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
        let shortcode_definitions = utils::templates::get_shortcodes(&tera);

        let content_path = path.join("content");
        let static_path = path.join("static");
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
            static_path,
            taxonomies: Vec::new(),
            permalinks: HashMap::new(),
            include_drafts: false,
            // We will allocate it properly later on
            library: Arc::new(RwLock::new(Library::default())),
            build_mode: BuildMode::Disk,
            shortcode_definitions,
        };

        Ok(site)
    }

    /// Enable some `zola serve` related options
    pub fn enable_serve_mode(&mut self) {
        SITE_CONTENT.write().unwrap().clear();
        self.config.enable_serve_mode();
        self.build_mode = BuildMode::Memory;
    }

    /// Set the site to load the drafts.
    /// Needs to be called before loading it
    pub fn include_drafts(&mut self) {
        self.include_drafts = true;
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
    pub fn enable_live_reload(&mut self, port_to_avoid: u16) {
        self.live_reload = get_available_port(port_to_avoid);
    }

    /// Only used in `zola serve` to re-use the initial websocket port
    pub fn enable_live_reload_with_port(&mut self, live_reload_port: u16) {
        self.live_reload = Some(live_reload_port);
    }

    /// Reloads the templates and rebuild the site without re-markdown the Markdown.
    pub fn reload_templates(&mut self) -> Result<()> {
        self.tera.full_reload()?;
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

    /// Reads all .md files in the `content` directory and create pages/sections
    /// out of them
    pub fn load(&mut self) -> Result<()> {
        self.library = Arc::new(RwLock::new(Library::new(&self.config)));
        let mut pages_insert_anchors = HashMap::new();

        // not the most elegant loop, but this is necessary to use skip_current_dir
        // which we can only decide to use after we've deserialised the section
        // so it's kinda necessecary
        let mut dir_walker =
            WalkDir::new(self.base_path.join("content")).follow_links(true).into_iter();
        let mut allowed_index_filenames: Vec<_> = self
            .config
            .other_languages()
            .keys()
            .map(|code| format!("_index.{}.md", code))
            .collect();
        allowed_index_filenames.push("_index.md".to_string());

        // We will insert colocated pages (those with a index.md filename)
        // at the end to detect pages that are actually errors:
        // when there is both a _index.md and index.md in the same folder
        let mut pages = Vec::new();
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
            match &self.config.ignored_content_globset {
                Some(gs) => {
                    if gs.is_match(path) {
                        continue;
                    }
                }

                None => (),
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
                            if f.path().is_file()
                                && allowed_index_filenames.iter().any(|s| s == path_str)
                            {
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
                let page = Page::from_file(path, &self.config, &self.base_path)?;
                pages.push(page);
            }
        }
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
                    bail!("We can't have a page called `index.md` in the same folder as an index section in {:?}", page.file.parent);
                }
            }

            pages_insert_anchors.insert(
                page.file.path.clone(),
                self.find_parent_section_insert_anchor(&page.file.parent.clone(), &page.lang),
            );
            self.add_page(page, false)?;
        }

        {
            let library = self.library.read().unwrap();
            let collisions = library.find_path_collisions();
            if !collisions.is_empty() {
                let mut msg = String::from("Found path collisions:\n");
                for (path, filepaths) in collisions {
                    let row = format!("- `{}` from files {:?}\n", path, filepaths);
                    msg.push_str(&row);
                }
                return Err(anyhow!(msg));
            }
        }

        // taxonomy Tera fns are loaded in `register_early_global_fns`
        // so we do need to populate it first.
        self.populate_taxonomies()?;
        tpls::register_early_global_fns(self)?;
        self.populate_sections();
        self.render_markdown()?;
        {
            let mut lib = self.library.write().unwrap();
            lib.fill_backlinks();
        }
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
                config::LinkCheckerLevel::Warn => console::warn(&msg),
                config::LinkCheckerLevel::Error => return Err(anyhow!(msg)),
            }
        }

        // check external links, log the results, and error out if needed
        if self.config.is_in_check_mode() {
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
                    config::LinkCheckerLevel::Warn => console::warn(&msg),
                    config::LinkCheckerLevel::Error => return Err(anyhow!(msg)),
                }
            }
        }

        Ok(())
    }

    /// Insert a default index section for each language if necessary so we don't need to create
    /// a _index.md to render the index page at the root of the site
    pub fn create_default_index_sections(&mut self) -> Result<()> {
        for (index_path, lang) in self.index_section_paths() {
            if let Some(index_section) = self.library.read().unwrap().sections.get(&index_path) {
                if self.config.build_search_index && !index_section.meta.in_search_index {
                    bail!(
                    "You have enabled search in the config but disabled it in the index section: \
                    either turn off the search in the config or remove `in_search_index = true` from the \
                    section front-matter."
                    )
                }
            }
            let mut library = self.library.write().expect("Get lock for load");
            // Not in else because of borrow checker
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
    /// Used in a build and in `serve` if a shortcode has changed
    pub fn render_markdown(&mut self) -> Result<()> {
        // Another silly thing needed to not borrow &self in parallel and
        // make the borrow checker happy
        let permalinks = &self.permalinks;
        let tera = &self.tera;
        let config = &self.config;

        // This is needed in the first place because of silly borrow checker
        let mut pages_insert_anchors = HashMap::new();
        for (_, p) in &self.library.read().unwrap().pages {
            pages_insert_anchors.insert(
                p.file.path.clone(),
                self.find_parent_section_insert_anchor(&p.file.parent.clone(), &p.lang),
            );
        }

        let mut library = self.library.write().expect("Get lock for render_markdown");
        library
            .pages
            .values_mut()
            .collect::<Vec<_>>()
            .par_iter_mut()
            .map(|page| {
                let insert_anchor = pages_insert_anchors[&page.file.path];
                page.render_markdown(
                    permalinks,
                    tera,
                    config,
                    insert_anchor,
                    &self.shortcode_definitions,
                )
            })
            .collect::<Result<()>>()?;

        library
            .sections
            .values_mut()
            .collect::<Vec<_>>()
            .par_iter_mut()
            .map(|section| {
                section.render_markdown(permalinks, tera, config, &self.shortcode_definitions)
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
            page.render_markdown(
                &self.permalinks,
                &self.tera,
                &self.config,
                insert_anchor,
                &self.shortcode_definitions,
            )?;
        }

        let mut library = self.library.write().expect("Get lock for add_page");
        library.pages.remove(&page.file.path);
        library.insert_page(page);

        Ok(())
    }

    /// Adds a page to the site and render it
    /// Only used in `zola serve --fast`
    pub fn add_and_render_page(&mut self, path: &Path) -> Result<()> {
        let page = Page::from_file(path, &self.config, &self.base_path)?;
        self.add_page(page, true)?;
        self.populate_sections();
        self.populate_taxonomies()?;
        let library = self.library.read().unwrap();
        let page = library.pages.get(path).unwrap();
        self.render_page(page)
    }

    /// Add a section to the site
    /// The `render` parameter is used in the serve command with --fast, when rebuilding a page.
    pub fn add_section(&mut self, mut section: Section, render_md: bool) -> Result<()> {
        self.permalinks.insert(section.file.relative.clone(), section.permalink.clone());
        if render_md {
            section.render_markdown(
                &self.permalinks,
                &self.tera,
                &self.config,
                &self.shortcode_definitions,
            )?;
        }
        let mut library = self.library.write().expect("Get lock for add_section");
        library.sections.remove(&section.file.path);
        library.insert_section(section);

        Ok(())
    }

    /// Adds a section to the site and render it
    /// Only used in `zola serve --fast`
    pub fn add_and_render_section(&mut self, path: &Path) -> Result<()> {
        let section = Section::from_file(path, &self.config, &self.base_path)?;
        self.add_section(section, true)?;
        self.populate_sections();
        let library = self.library.read().unwrap();
        let section = library.sections.get(path).unwrap();
        self.render_section(section, true)
    }

    /// Finds the insert_anchor for the parent section of the directory at `path`.
    /// Defaults to `AnchorInsert::None` if no parent section found
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
        match self.library.read().unwrap().sections.get(&parent) {
            Some(s) => s.meta.insert_anchor_links,
            None => InsertAnchor::None,
        }
    }

    /// Find out the direct subsections of each subsection if there are some
    /// as well as the pages for each section
    pub fn populate_sections(&mut self) {
        let mut library = self.library.write().expect("Get lock for populate_sections");
        library.populate_sections(&self.config, &self.content_path);
    }

    /// Find all the tags and categories if it's asked in the config
    pub fn populate_taxonomies(&mut self) -> Result<()> {
        self.taxonomies = self.library.read().unwrap().find_taxonomies(&self.config);
        Ok(())
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
            if let Some(gs) = &self.config.ignored_static_globset {
                copy_directory(
                    &self.static_path,
                    &self.output_path,
                    self.config.hard_link_static,
                    Some(gs),
                )?;
            } else {
                copy_directory(
                    &self.static_path,
                    &self.output_path,
                    self.config.hard_link_static,
                    None,
                )?;
            }
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
        imageproc.do_process()
    }

    /// Deletes the `public` directory if it exists and the `preserve_dotfiles_in_output` option is set to false,
    /// or if set to true: its contents except for the dotfiles at the root level.
    pub fn clean(&self) -> Result<()> {
        clean_site_output_folder(&self.output_path, self.config.preserve_dotfiles_in_output)
    }

    /// Handles whether to write to disk or to memory
    pub fn write_content(
        &self,
        components: &[&str],
        filename: &str,
        content: String,
        create_dirs: bool,
    ) -> Result<PathBuf> {
        let write_dirs = self.build_mode == BuildMode::Disk || create_dirs;
        ensure_directory_exists(&self.output_path)?;

        let mut site_path = RelativePathBuf::new();
        let mut current_path = self.output_path.to_path_buf();

        for component in components {
            current_path.push(component);
            site_path.push(component);

            if !current_path.exists() && write_dirs {
                create_directory(&current_path)?;
            }
        }

        if write_dirs {
            create_directory(&current_path)?;
        }

        let final_content = if !filename.ends_with("html") || !self.config.minify_html {
            content
        } else {
            match minify::html(content) {
                Ok(minified_content) => minified_content,
                Err(error) => bail!(error),
            }
        };

        match self.build_mode {
            BuildMode::Disk => {
                let end_path = current_path.join(filename);
                create_file(&end_path, &final_content)?;
            }
            BuildMode::Memory => {
                let site_path =
                    if filename != "index.html" { site_path.join(filename) } else { site_path };

                SITE_CONTENT.write().unwrap().insert(site_path, final_content);
            }
        }

        Ok(current_path)
    }

    fn copy_asset(&self, src: &Path, dest: &Path) -> Result<()> {
        copy_file_if_needed(src, dest, self.config.hard_link_static)
    }

    /// Renders a single content page
    pub fn render_page(&self, page: &Page) -> Result<()> {
        let output = page.render_html(&self.tera, &self.config, &self.library.read().unwrap())?;
        let content = self.inject_livereload(output);
        let components: Vec<&str> = page.path.split('/').collect();
        let current_path =
            self.write_content(&components, "index.html", content, !page.assets.is_empty())?;

        // Copy any asset we found previously into the same directory as the index.html
        for asset in &page.assets {
            let asset_path = asset.as_path();
            self.copy_asset(
                asset_path,
                &current_path.join(
                    asset_path
                        .strip_prefix(page.file.path.parent().unwrap())
                        .expect("Couldn't get filename from page asset"),
                ),
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

        // Render aliases first to allow overwriting
        self.render_aliases()?;
        start = log_time(start, "Rendered aliases");
        self.render_sections()?;
        start = log_time(start, "Rendered sections");
        self.render_orphan_pages()?;
        start = log_time(start, "Rendered orphan pages");
        self.render_sitemap()?;
        start = log_time(start, "Rendered sitemap");

        let library = self.library.read().unwrap();
        if self.config.generate_feed {
            let is_multilingual = self.config.is_multilingual();
            let pages: Vec<_> = if is_multilingual {
                library.pages.values().filter(|p| p.lang == self.config.default_language).collect()
            } else {
                library.pages.values().collect()
            };
            self.render_feed(pages, None, &self.config.default_language, |c| c)?;
            start = log_time(start, "Generated feed in default language");
        }

        for (code, language) in &self.config.other_languages() {
            if !language.generate_feed {
                continue;
            }
            let pages: Vec<_> = library.pages.values().filter(|p| &p.lang == code).collect();
            self.render_feed(pages, Some(&PathBuf::from(code)), code, |c| c)?;
            start = log_time(start, "Generated feed in other language");
        }
        self.render_themes_css()?;
        start = log_time(start, "Rendered themes css");
        self.render_404()?;
        start = log_time(start, "Rendered 404");
        self.render_robots()?;
        start = log_time(start, "Rendered robots.txt");
        self.render_taxonomies()?;
        start = log_time(start, "Rendered taxonomies");
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
        ensure_directory_exists(&self.static_path)?;

        for t in &self.config.markdown.highlight_themes_css {
            let p = self.static_path.join(&t.filename);
            if !p.exists() {
                let content = &self.config.markdown.export_theme_css(&t.theme)?;
                create_file(&p, content)?;
            }
        }

        Ok(())
    }

    fn index_for_lang(&self, lang: &str) -> Result<()> {
        let index_json = search::build_index(lang, &self.library.read().unwrap(), &self.config)?;
        let (path, content) = match &self.config.search.index_format {
            IndexFormat::ElasticlunrJson => {
                let path = self.output_path.join(format!("search_index.{}.json", lang));
                (path, index_json)
            }
            IndexFormat::ElasticlunrJavascript => {
                let path = self.output_path.join(format!("search_index.{}.js", lang));
                let content = format!("window.searchIndex = {};", index_json);
                (path, content)
            }
        };
        create_file(&path, &content)
    }

    pub fn build_search_index(&self) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;
        // TODO: add those to the SITE_CONTENT map

        // index first
        self.index_for_lang(&self.config.default_language)?;

        for (code, language) in &self.config.other_languages() {
            if code != &self.config.default_language && language.build_search_index {
                self.index_for_lang(code)?;
            }
        }

        // then elasticlunr.min.js
        create_file(&self.output_path.join("elasticlunr.min.js"), search::ELASTICLUNR_JS)?;

        Ok(())
    }

    fn render_alias(&self, alias: &str, permalink: &str) -> Result<()> {
        let mut split = alias.split('/').collect::<Vec<_>>();

        // If the alias ends with an html file name, use that instead of mapping
        // as a path containing an `index.html`
        let page_name = match split.pop() {
            Some(part) if part.ends_with(".html") => part,
            Some(part) => {
                split.push(part);
                "index.html"
            }
            None => "index.html",
        };
        let content = render_redirect_template(permalink, &self.tera)?;
        self.write_content(&split, page_name, content, false)?;
        Ok(())
    }

    /// Renders all the aliases for each page/section: a magic HTML template that redirects to
    /// the canonical one
    pub fn render_aliases(&self) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;
        let library = self.library.read().unwrap();
        for (_, page) in &library.pages {
            for alias in &page.meta.aliases {
                self.render_alias(alias, &page.permalink)?;
            }
        }
        for (_, section) in &library.sections {
            for alias in &section.meta.aliases {
                self.render_alias(alias, &section.permalink)?;
            }
        }
        Ok(())
    }

    /// Renders 404.html
    pub fn render_404(&self) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;
        let mut context = Context::new();
        context.insert("config", &self.config.serialize(&self.config.default_language));
        context.insert("lang", &self.config.default_language);
        let output = render_template("404.html", &self.tera, context, &self.config.theme)?;
        let content = self.inject_livereload(output);
        self.write_content(&[], "404.html", content, false)?;
        Ok(())
    }

    /// Renders robots.txt
    pub fn render_robots(&self) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;
        let mut context = Context::new();
        context.insert("config", &self.config.serialize(&self.config.default_language));
        let content = render_template("robots.txt", &self.tera, context, &self.config.theme)?;
        self.write_content(&[], "robots.txt", content, false)?;
        Ok(())
    }

    /// Renders all taxonomies
    pub fn render_taxonomies(&self) -> Result<()> {
        for taxonomy in &self.taxonomies {
            if !taxonomy.kind.render {
                continue;
            }
            self.render_taxonomy(taxonomy)?;
        }

        Ok(())
    }

    fn render_taxonomy(&self, taxonomy: &Taxonomy) -> Result<()> {
        if taxonomy.items.is_empty() {
            return Ok(());
        }

        ensure_directory_exists(&self.output_path)?;

        let mut components = Vec::new();
        if taxonomy.lang != self.config.default_language {
            components.push(taxonomy.lang.as_ref());
        }

        components.push(taxonomy.slug.as_ref());

        let list_output =
            taxonomy.render_all_terms(&self.tera, &self.config, &self.library.read().unwrap())?;
        let content = self.inject_livereload(list_output);
        self.write_content(&components, "index.html", content, false)?;

        let library = self.library.read().unwrap();
        taxonomy
            .items
            .par_iter()
            .map(|item| {
                let mut comp = components.clone();
                comp.push(&item.slug);

                if taxonomy.kind.is_paginated() {
                    self.render_paginated(
                        comp.clone(),
                        &Paginator::from_taxonomy(
                            taxonomy,
                            item,
                            &library,
                            &self.tera,
                            &self.config.theme,
                        ),
                    )?;
                } else {
                    let single_output =
                        taxonomy.render_term(item, &self.tera, &self.config, &library)?;
                    let content = self.inject_livereload(single_output);
                    self.write_content(&comp, "index.html", content, false)?;
                }

                if taxonomy.kind.feed {
                    let tax_path = if taxonomy.lang == self.config.default_language {
                        PathBuf::from(format!("{}/{}", taxonomy.slug, item.slug))
                    } else {
                        PathBuf::from(format!("{}/{}/{}", taxonomy.lang, taxonomy.slug, item.slug))
                    };
                    self.render_feed(
                        item.pages.iter().map(|p| library.pages.get(p).unwrap()).collect(),
                        Some(&tax_path),
                        &taxonomy.lang,
                        |mut context: Context| {
                            context.insert("taxonomy", &taxonomy.kind);
                            context
                                .insert("term", &feed::SerializedFeedTaxonomyItem::from_item(item));
                            context
                        },
                    )
                } else {
                    Ok(())
                }
            })
            .collect::<Result<()>>()
    }

    /// What it says on the tin
    pub fn render_sitemap(&self) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;

        let library = self.library.read().unwrap();
        let all_sitemap_entries =
            { sitemap::find_entries(&library, &self.taxonomies[..], &self.config) };
        let sitemap_limit = 30000;

        if all_sitemap_entries.len() < sitemap_limit {
            // Create single sitemap
            let mut context = Context::new();
            context.insert("entries", &all_sitemap_entries);
            let sitemap = render_template("sitemap.xml", &self.tera, context, &self.config.theme)?;
            self.write_content(&[], "sitemap.xml", sitemap, false)?;
            return Ok(());
        }

        // Create multiple sitemaps (max 30000 urls each)
        let mut sitemap_index = Vec::new();
        for (i, chunk) in
            all_sitemap_entries.iter().collect::<Vec<_>>().chunks(sitemap_limit).enumerate()
        {
            let mut context = Context::new();
            context.insert("entries", &chunk);
            let sitemap = render_template("sitemap.xml", &self.tera, context, &self.config.theme)?;
            let file_name = format!("sitemap{}.xml", i + 1);
            self.write_content(&[], &file_name, sitemap, false)?;
            let mut sitemap_url = self.config.make_permalink(&file_name);
            sitemap_url.pop(); // Remove trailing slash
            sitemap_index.push(sitemap_url);
        }

        // Create main sitemap that reference numbered sitemaps
        let mut main_context = Context::new();
        main_context.insert("sitemaps", &sitemap_index);
        let sitemap = render_template(
            "split_sitemap_index.xml",
            &self.tera,
            main_context,
            &self.config.theme,
        )?;
        self.write_content(&[], "sitemap.xml", sitemap, false)?;

        Ok(())
    }

    /// Renders a feed for the given path and at the given path
    /// If both arguments are `None`, it will render only the feed for the whole
    /// site at the root folder.
    pub fn render_feed(
        &self,
        all_pages: Vec<&Page>,
        base_path: Option<&PathBuf>,
        lang: &str,
        additional_context_fn: impl Fn(Context) -> Context,
    ) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;

        let feed = match feed::render_feed(self, all_pages, lang, base_path, additional_context_fn)?
        {
            Some(v) => v,
            None => return Ok(()),
        };
        let feed_filename = &self.config.feed_filename;

        if let Some(base) = base_path {
            let mut components = Vec::new();
            for component in base.components() {
                components.push(component.as_os_str().to_string_lossy());
            }
            self.write_content(
                &components.iter().map(|x| x.as_ref()).collect::<Vec<_>>(),
                feed_filename,
                feed,
                false,
            )?;
        } else {
            self.write_content(&[], feed_filename, feed, false)?;
        }
        Ok(())
    }

    /// Renders a single section
    pub fn render_section(&self, section: &Section, render_pages: bool) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;
        let mut output_path = self.output_path.clone();
        let mut components: Vec<&str> = Vec::new();
        let create_directories = self.build_mode == BuildMode::Disk || !section.assets.is_empty();

        if section.lang != self.config.default_language {
            components.push(&section.lang);
            output_path.push(&section.lang);

            if !output_path.exists() && create_directories {
                create_directory(&output_path)?;
            }
        }

        for component in &section.file.components {
            components.push(component);
            output_path.push(component);

            if !output_path.exists() && create_directories {
                create_directory(&output_path)?;
            }
        }

        if section.meta.generate_feed {
            let library = &self.library.read().unwrap();
            let pages = section.pages.iter().map(|k| library.pages.get(k).unwrap()).collect();
            self.render_feed(
                pages,
                Some(&PathBuf::from(&section.path[1..])),
                &section.lang,
                |mut context: Context| {
                    context.insert("section", &section.serialize(library));
                    context
                },
            )?;
        }

        // Copy any asset we found previously into the same directory as the index.html
        for asset in &section.assets {
            let asset_path = asset.as_path();
            self.copy_asset(
                asset_path,
                &output_path.join(
                    asset_path
                        .strip_prefix(section.file.path.parent().unwrap())
                        .expect("Failed to get asset filename for section"),
                ),
            )?;
        }

        if render_pages {
            section
                .pages
                .par_iter()
                .map(|k| self.render_page(self.library.read().unwrap().pages.get(k).unwrap()))
                .collect::<Result<()>>()?;
        }

        if !section.meta.render {
            return Ok(());
        }

        if let Some(ref redirect_to) = section.meta.redirect_to {
            let permalink: Cow<String> = if is_external_link(redirect_to) {
                Cow::Borrowed(redirect_to)
            } else {
                Cow::Owned(self.config.make_permalink(redirect_to))
            };
            self.write_content(
                &components,
                "index.html",
                render_redirect_template(&permalink, &self.tera)?,
                create_directories,
            )?;

            return Ok(());
        }

        if section.meta.is_paginated() {
            self.render_paginated(
                components,
                &Paginator::from_section(section, &self.library.read().unwrap()),
            )?;
        } else {
            let output =
                section.render_html(&self.tera, &self.config, &self.library.read().unwrap())?;
            let content = self.inject_livereload(output);
            self.write_content(&components, "index.html", content, false)?;
        }

        Ok(())
    }

    /// Renders all sections
    pub fn render_sections(&self) -> Result<()> {
        self.library
            .read()
            .unwrap()
            .sections
            .par_iter()
            .map(|(_, s)| self.render_section(s, true))
            .collect::<Result<()>>()
    }

    /// Renders all pages that do not belong to any sections
    pub fn render_orphan_pages(&self) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;
        let library = self.library.read().unwrap();
        for page in library.get_all_orphan_pages() {
            self.render_page(page)?;
        }

        Ok(())
    }

    /// Renders a list of pages when the section/index is wanting pagination.
    pub fn render_paginated<'a>(
        &self,
        components: Vec<&'a str>,
        paginator: &'a Paginator,
    ) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;

        let index_components = components.clone();

        paginator
            .pagers
            .par_iter()
            .map(|pager| {
                let mut pager_components = index_components.clone();
                pager_components.push(&paginator.paginate_path);
                let pager_path = format!("{}", pager.index);
                pager_components.push(&pager_path);
                let output = paginator.render_pager(
                    pager,
                    &self.config,
                    &self.tera,
                    &self.library.read().unwrap(),
                )?;
                let content = self.inject_livereload(output);

                if pager.index > 1 {
                    self.write_content(&pager_components, "index.html", content, false)?;
                } else {
                    self.write_content(&index_components, "index.html", content, false)?;
                    self.write_content(
                        &pager_components,
                        "index.html",
                        render_redirect_template(&paginator.permalink, &self.tera)?,
                        false,
                    )?;
                }

                Ok(())
            })
            .collect::<Result<()>>()
    }
}

fn log_time(start: Instant, message: &str) -> Instant {
    let do_print = std::env::var("ZOLA_PERF_LOG").is_ok();
    let now = Instant::now();
    if do_print {
        println!("{} took {}ms", message, now.duration_since(start).as_millis());
    }
    now
}
