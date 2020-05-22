pub mod feed;
pub mod link_checking;
pub mod sass;
pub mod sitemap;
pub mod tpls;

use std::collections::HashMap;
use std::fs::{copy, remove_dir_all};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use glob::glob;
use rayon::prelude::*;
use tera::{Context, Tera};

use config::{get_config, Config};
use errors::{bail, Error, Result};
use front_matter::InsertAnchor;
use library::{find_taxonomies, Library, Page, Paginator, Section, Taxonomy};
use templates::render_redirect_template;
use utils::fs::{copy_directory, create_directory, create_file, ensure_directory_exists};
use utils::net::get_available_port;
use utils::templates::render_template;

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
}

impl Site {
    /// Parse a site at the given path. Defaults to the current dir
    /// Passing in a path is used in tests and when --root argument is passed
    pub fn new<P: AsRef<Path>, P2: AsRef<Path>>(path: P, config_file: P2) -> Result<Site> {
        let path = path.as_ref();
        let config_file = config_file.as_ref();
        let mut config = get_config(config_file);
        config.load_extra_syntaxes(path)?;

        if let Some(theme) = config.theme.clone() {
            // Grab data from the extra section of the theme
            config.merge_with_theme(&path.join("themes").join(&theme).join("theme.toml"))?;
        }

        let tera = tpls::load_tera(path, &config)?;

        let content_path = path.join("content");
        let static_path = path.join("static");
        let imageproc =
            imageproc::Processor::new(content_path.clone(), &static_path, &config.base_url);

        let site = Site {
            base_path: path.to_path_buf(),
            config,
            tera,
            imageproc: Arc::new(Mutex::new(imageproc)),
            live_reload: None,
            output_path: path.join("public"),
            content_path,
            static_path,
            taxonomies: Vec::new(),
            permalinks: HashMap::new(),
            include_drafts: false,
            // We will allocate it properly later on
            library: Arc::new(RwLock::new(Library::new(0, 0, false))),
        };

        Ok(site)
    }

    /// Set the site to load the drafts.
    /// Needs to be called before loading it
    pub fn include_drafts(&mut self) {
        self.include_drafts = true;
    }

    /// The index sections are ALWAYS at those paths
    /// There are one index section for the default language + 1 per language
    fn index_section_paths(&self) -> Vec<(PathBuf, Option<String>)> {
        let mut res = vec![(self.content_path.join("_index.md"), None)];
        for language in &self.config.languages {
            res.push((
                self.content_path.join(format!("_index.{}.md", language.code)),
                Some(language.code.clone()),
            ));
        }
        res
    }

    /// We avoid the port the server is going to use as it's not bound yet
    /// when calling this function and we could end up having tried to bind
    /// both http and websocket server to the same port
    pub fn enable_live_reload(&mut self, port_to_avoid: u16) {
        self.live_reload = get_available_port(port_to_avoid);
    }

    pub fn set_base_url(&mut self, base_url: String) {
        let mut imageproc = self.imageproc.lock().expect("Couldn't lock imageproc (set_base_url)");
        imageproc.set_base_url(&base_url);
        self.config.base_url = base_url;
    }

    pub fn set_output_path<P: AsRef<Path>>(&mut self, path: P) {
        self.output_path = path.as_ref().to_path_buf();
    }

    /// Reads all .md files in the `content` directory and create pages/sections
    /// out of them
    pub fn load(&mut self) -> Result<()> {
        let base_path = self.base_path.to_string_lossy().replace("\\", "/");
        let content_glob = format!("{}/{}", base_path, "content/**/*.md");

        let (section_entries, page_entries): (Vec<_>, Vec<_>) = glob(&content_glob)
            .expect("Invalid glob")
            .filter_map(|e| e.ok())
            .filter(|e| !e.as_path().file_name().unwrap().to_str().unwrap().starts_with('.'))
            .partition(|entry| {
                entry.as_path().file_name().unwrap().to_str().unwrap().starts_with("_index.")
            });

        self.library = Arc::new(RwLock::new(Library::new(
            page_entries.len(),
            section_entries.len(),
            self.config.is_multilingual(),
        )));

        let sections = {
            let config = &self.config;

            section_entries
                .into_par_iter()
                .map(|entry| {
                    let path = entry.as_path();
                    Section::from_file(path, config, &self.base_path)
                })
                .collect::<Vec<_>>()
        };

        let pages = {
            let config = &self.config;

            page_entries
                .into_par_iter()
                .filter(|entry| match &config.ignored_content_globset {
                    Some(gs) => !gs.is_match(entry.as_path()),
                    None => true,
                })
                .map(|entry| {
                    let path = entry.as_path();
                    Page::from_file(path, config, &self.base_path)
                })
                .collect::<Vec<_>>()
        };

        // Kinda duplicated code for add_section/add_page but necessary to do it that
        // way because of the borrow checker
        for section in sections {
            let s = section?;
            self.add_section(s, false)?;
        }

        self.create_default_index_sections()?;

        let mut pages_insert_anchors = HashMap::new();
        for page in pages {
            let p = page?;
            // Should draft pages be ignored?
            if p.meta.draft && !self.include_drafts {
                continue;
            }
            pages_insert_anchors.insert(
                p.file.path.clone(),
                self.find_parent_section_insert_anchor(&p.file.parent.clone(), &p.lang),
            );
            self.add_page(p, false)?;
        }

        {
            let library = self.library.read().unwrap();
            let collisions = library.check_for_path_collisions();
            if !collisions.is_empty() {
                return Err(Error::from_collisions(collisions));
            }
        }

        // taxonomy Tera fns are loaded in `register_early_global_fns`
        // so we do need to populate it first.
        self.populate_taxonomies()?;
        self.register_early_global_fns();
        self.populate_fluent()?;
        self.populate_sections();
        self.render_markdown()?;
        self.register_tera_global_fns();

        // Needs to be done after rendering markdown as we only get the anchors at that point
        link_checking::check_internal_links_with_anchors(&self)?;

        if self.config.is_in_check_mode() {
            link_checking::check_external_links(&self)?;
        }

        Ok(())
    }

    /// Insert a default index section for each language if necessary so we don't need to create
    /// a _index.md to render the index page at the root of the site
    pub fn create_default_index_sections(&mut self) -> Result<()> {
        for (index_path, lang) in self.index_section_paths() {
            if let Some(ref index_section) = self.library.read().unwrap().get_section(&index_path) {
                if self.config.build_search_index && !index_section.meta.in_search_index {
                    bail!(
                    "You have enabled search in the config but disabled it in the index section: \
                    either turn off the search in the config or remote `in_search_index = true` from the \
                    section front-matter."
                    )
                }
            }
            let mut library = self.library.write().expect("Get lock for load");
            // Not in else because of borrow checker
            if !library.contains_section(&index_path) {
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
                } else {
                    index_section.file.name = "_index".to_string();
                    index_section.permalink = self.config.make_permalink("");
                    index_section.file.path = self.content_path.join("_index.md");
                    index_section.file.relative = "_index.md".to_string();
                    index_section.path = "/".to_string();
                }
                index_section.lang = index_section.file.find_language(&self.config)?;
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
        for (_, p) in self.library.read().unwrap().pages() {
            pages_insert_anchors.insert(
                p.file.path.clone(),
                self.find_parent_section_insert_anchor(&p.file.parent.clone(), &p.lang),
            );
        }

        let mut library = self.library.write().expect("Get lock for render_markdown");
        library
            .pages_mut()
            .values_mut()
            .collect::<Vec<_>>()
            .par_iter_mut()
            .map(|page| {
                let insert_anchor = pages_insert_anchors[&page.file.path];
                page.render_markdown(permalinks, tera, config, insert_anchor)
            })
            .collect::<Result<()>>()?;

        library
            .sections_mut()
            .values_mut()
            .collect::<Vec<_>>()
            .par_iter_mut()
            .map(|section| section.render_markdown(permalinks, tera, config))
            .collect::<Result<()>>()?;

        Ok(())
    }

    // TODO: remove me in favour of the direct call to the fn once rebuild has changed
    pub fn register_early_global_fns(&mut self) {
        tpls::register_early_global_fns(self);
    }

    // TODO: remove me in favour of the direct call to the fn once rebuild has changed
    pub fn register_tera_global_fns(&mut self) {
        tpls::register_tera_global_fns(self);
    }

    /// Add a page to the site
    /// The `render` parameter is used in the serve command, when rebuilding a page.
    /// If `true`, it will also render the markdown for that page
    /// Returns the previous page struct if there was one at the same path
    pub fn add_page(&mut self, mut page: Page, render: bool) -> Result<Option<Page>> {
        self.permalinks.insert(page.file.relative.clone(), page.permalink.clone());
        if render {
            let insert_anchor =
                self.find_parent_section_insert_anchor(&page.file.parent, &page.lang);
            page.render_markdown(&self.permalinks, &self.tera, &self.config, insert_anchor)?;
        }
        let mut library = self.library.write().expect("Get lock for add_page");
        let prev = library.remove_page(&page.file.path);
        library.insert_page(page);

        Ok(prev)
    }

    /// Add a section to the site
    /// The `render` parameter is used in the serve command, when rebuilding a page.
    /// If `true`, it will also render the markdown for that page
    /// Returns the previous section struct if there was one at the same path
    pub fn add_section(&mut self, mut section: Section, render: bool) -> Result<Option<Section>> {
        self.permalinks.insert(section.file.relative.clone(), section.permalink.clone());
        if render {
            section.render_markdown(&self.permalinks, &self.tera, &self.config)?;
        }
        let mut library = self.library.write().expect("Get lock for add_section");
        let prev = library.remove_section(&section.file.path);
        library.insert_section(section);

        Ok(prev)
    }

    /// Finds the insert_anchor for the parent section of the directory at `path`.
    /// Defaults to `AnchorInsert::None` if no parent section found
    pub fn find_parent_section_insert_anchor(
        &self,
        parent_path: &PathBuf,
        lang: &str,
    ) -> InsertAnchor {
        let parent = if lang != self.config.default_language {
            parent_path.join(format!("_index.{}.md", lang))
        } else {
            parent_path.join("_index.md")
        };
        match self.library.read().unwrap().get_section(&parent) {
            Some(s) => s.meta.insert_anchor_links,
            None => InsertAnchor::None,
        }
    }

    pub fn populate_fluent(&mut self) -> Result<()> {
        if std::fs::metadata(&self.config.fluent_dir).is_ok() {
            let loader =
                fluent_templates::ArcLoader::builder(&self.config.fluent_dir, self.config.default_language.parse().map_err(|e| format!("{}", e))?)
                .shared_resources(Some(&*self.config.shared_fluent_resources))
                .build()
                .map_err(|e| e.to_string())?;
            self.tera.register_function("fluent", fluent_templates::FluentLoader::new(loader));
        }

        Ok(())
    }

    /// Find out the direct subsections of each subsection if there are some
    /// as well as the pages for each section
    pub fn populate_sections(&mut self) {
        let mut library = self.library.write().expect("Get lock for populate_sections");
        library.populate_sections(&self.config);
    }

    /// Find all the tags and categories if it's asked in the config
    pub fn populate_taxonomies(&mut self) -> Result<()> {
        if self.config.taxonomies.is_empty() {
            return Ok(());
        }

        self.taxonomies = find_taxonomies(&self.config, &self.library.read().unwrap())?;

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
            )?;
        }
        // We're fine with missing static folders
        if self.static_path.exists() {
            copy_directory(&self.static_path, &self.output_path, self.config.hard_link_static)?;
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

    /// Deletes the `public` directory if it exists
    pub fn clean(&self) -> Result<()> {
        if self.output_path.exists() {
            // Delete current `public` directory so we can start fresh
            remove_dir_all(&self.output_path)
                .map_err(|e| Error::chain("Couldn't delete output directory", e))?;
        }

        Ok(())
    }

    /// Renders a single content page
    pub fn render_page(&self, page: &Page) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;

        // Copy the nesting of the content directory if we have sections for that page
        let mut current_path = self.output_path.to_path_buf();

        for component in page.path.split('/') {
            current_path.push(component);

            if !current_path.exists() {
                create_directory(&current_path)?;
            }
        }

        // Make sure the folder exists
        create_directory(&current_path)?;

        // Finally, create a index.html file there with the page rendered
        let output = page.render_html(&self.tera, &self.config, &self.library.read().unwrap())?;
        create_file(&current_path.join("index.html"), &self.inject_livereload(output))?;

        // Copy any asset we found previously into the same directory as the index.html
        for asset in &page.assets {
            let asset_path = asset.as_path();
            copy(
                &asset_path,
                &current_path
                    .join(asset_path.file_name().expect("Couldn't get filename from page asset")),
            )?;
        }

        Ok(())
    }

    /// Deletes the `public` directory and builds the site
    pub fn build(&self) -> Result<()> {
        self.clean()?;

        // Generate/move all assets before rendering any content
        if let Some(ref theme) = self.config.theme {
            let theme_path = self.base_path.join("themes").join(theme);
            if theme_path.join("sass").exists() {
                sass::compile_sass(&theme_path, &self.output_path)?;
            }
        }

        if self.config.compile_sass {
            sass::compile_sass(&self.base_path, &self.output_path)?;
        }

        if self.config.build_search_index {
            self.build_search_index()?;
        }

        // Render aliases first to allow overwriting
        self.render_aliases()?;
        self.render_sections()?;
        self.render_orphan_pages()?;
        self.render_sitemap()?;

        let library = self.library.read().unwrap();
        if self.config.generate_feed {
            let is_multilingual = self.config.is_multilingual();
            let pages = if is_multilingual {
                library
                    .pages_values()
                    .iter()
                    .filter(|p| p.lang == self.config.default_language)
                    .cloned()
                    .collect()
            } else {
                library.pages_values()
            };
            self.render_feed(pages, None, &self.config.default_language, |c| c)?;
        }

        for lang in &self.config.languages {
            if !lang.feed {
                continue;
            }
            let pages =
                library.pages_values().iter().filter(|p| p.lang == lang.code).cloned().collect();
            self.render_feed(pages, Some(&PathBuf::from(lang.code.clone())), &lang.code, |c| c)?;
        }

        self.render_404()?;
        self.render_robots()?;
        self.render_taxonomies()?;
        // We process images at the end as we might have picked up images to process from markdown
        // or from templates
        self.process_images()?;
        // Processed images will be in static so the last step is to copy it
        self.copy_static_directories()?;

        Ok(())
    }

    pub fn build_search_index(&self) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;
        // index first
        create_file(
            &self.output_path.join(&format!("search_index.{}.js", self.config.default_language)),
            &format!(
                "window.searchIndex = {};",
                search::build_index(
                    &self.config.default_language,
                    &self.library.read().unwrap(),
                    &self.config
                )?
            ),
        )?;

        for language in &self.config.languages {
            if language.code != self.config.default_language && language.search {
                create_file(
                    &self.output_path.join(&format!("search_index.{}.js", &language.code)),
                    &format!(
                        "window.searchIndex = {};",
                        search::build_index(
                            &language.code,
                            &self.library.read().unwrap(),
                            &self.config
                        )?
                    ),
                )?;
            }
        }

        // then elasticlunr.min.js
        create_file(&self.output_path.join("elasticlunr.min.js"), search::ELASTICLUNR_JS)?;

        Ok(())
    }

    fn render_alias(&self, alias: &str, permalink: &str) -> Result<()> {
        let mut output_path = self.output_path.to_path_buf();
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

        for component in split {
            output_path.push(&component);

            if !output_path.exists() {
                create_directory(&output_path)?;
            }
        }

        create_file(
            &output_path.join(page_name),
            &render_redirect_template(&permalink, &self.tera)?,
        )
    }

    /// Renders all the aliases for each page/section: a magic HTML template that redirects to
    /// the canonical one
    pub fn render_aliases(&self) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;
        let library = self.library.read().unwrap();
        for (_, page) in library.pages() {
            for alias in &page.meta.aliases {
                self.render_alias(&alias, &page.permalink)?;
            }
        }
        for (_, section) in library.sections() {
            for alias in &section.meta.aliases {
                self.render_alias(&alias, &section.permalink)?;
            }
        }
        Ok(())
    }

    /// Renders 404.html
    pub fn render_404(&self) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;
        let mut context = Context::new();
        context.insert("config", &self.config);
        let output = render_template("404.html", &self.tera, context, &self.config.theme)?;
        create_file(&self.output_path.join("404.html"), &self.inject_livereload(output))
    }

    /// Renders robots.txt
    pub fn render_robots(&self) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;
        let mut context = Context::new();
        context.insert("config", &self.config);
        create_file(
            &self.output_path.join("robots.txt"),
            &render_template("robots.txt", &self.tera, context, &self.config.theme)?,
        )
    }

    /// Renders all taxonomies
    pub fn render_taxonomies(&self) -> Result<()> {
        for taxonomy in &self.taxonomies {
            self.render_taxonomy(taxonomy)?;
        }

        Ok(())
    }

    fn render_taxonomy(&self, taxonomy: &Taxonomy) -> Result<()> {
        if taxonomy.items.is_empty() {
            return Ok(());
        }

        ensure_directory_exists(&self.output_path)?;
        let output_path = if taxonomy.kind.lang != self.config.default_language {
            let mid_path = self.output_path.join(&taxonomy.kind.lang);
            create_directory(&mid_path)?;
            mid_path.join(&taxonomy.kind.name)
        } else {
            self.output_path.join(&taxonomy.kind.name)
        };
        let list_output =
            taxonomy.render_all_terms(&self.tera, &self.config, &self.library.read().unwrap())?;
        create_directory(&output_path)?;
        create_file(&output_path.join("index.html"), &self.inject_livereload(list_output))?;
        let library = self.library.read().unwrap();
        taxonomy
            .items
            .par_iter()
            .map(|item| {
                let path = output_path.join(&item.slug);
                if taxonomy.kind.is_paginated() {
                    self.render_paginated(
                        &path,
                        &Paginator::from_taxonomy(&taxonomy, item, &library),
                    )?;
                } else {
                    let single_output =
                        taxonomy.render_term(item, &self.tera, &self.config, &library)?;
                    create_directory(&path)?;
                    create_file(&path.join("index.html"), &self.inject_livereload(single_output))?;
                }

                if taxonomy.kind.feed {
                    self.render_feed(
                        item.pages.iter().map(|p| library.get_page_by_key(*p)).collect(),
                        Some(&PathBuf::from(format!("{}/{}", taxonomy.kind.name, item.slug))),
                        if self.config.is_multilingual() && !taxonomy.kind.lang.is_empty() {
                            &taxonomy.kind.lang
                        } else {
                            &self.config.default_language
                        },
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
            let sitemap = &render_template("sitemap.xml", &self.tera, context, &self.config.theme)?;
            create_file(&self.output_path.join("sitemap.xml"), sitemap)?;
            return Ok(());
        }

        // Create multiple sitemaps (max 30000 urls each)
        let mut sitemap_index = Vec::new();
        for (i, chunk) in
            all_sitemap_entries.iter().collect::<Vec<_>>().chunks(sitemap_limit).enumerate()
        {
            let mut context = Context::new();
            context.insert("entries", &chunk);
            let sitemap = &render_template("sitemap.xml", &self.tera, context, &self.config.theme)?;
            let file_name = format!("sitemap{}.xml", i + 1);
            create_file(&self.output_path.join(&file_name), sitemap)?;
            let mut sitemap_url: String = self.config.make_permalink(&file_name);
            sitemap_url.pop(); // Remove trailing slash
            sitemap_index.push(sitemap_url);
        }

        // Create main sitemap that reference numbered sitemaps
        let mut main_context = Context::new();
        main_context.insert("sitemaps", &sitemap_index);
        let sitemap = &render_template(
            "split_sitemap_index.xml",
            &self.tera,
            main_context,
            &self.config.theme,
        )?;
        create_file(&self.output_path.join("sitemap.xml"), sitemap)?;

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

        if let Some(ref base) = base_path {
            let mut output_path = self.output_path.clone();
            for component in base.components() {
                output_path.push(component);
                if !output_path.exists() {
                    create_directory(&output_path)?;
                }
            }
            create_file(&output_path.join(feed_filename), &feed)?;
        } else {
            create_file(&self.output_path.join(feed_filename), &feed)?;
        }
        Ok(())
    }

    /// Renders a single section
    pub fn render_section(&self, section: &Section, render_pages: bool) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;
        let mut output_path = self.output_path.clone();

        if section.lang != self.config.default_language {
            output_path.push(&section.lang);
            if !output_path.exists() {
                create_directory(&output_path)?;
            }
        }

        for component in &section.file.components {
            output_path.push(component);

            if !output_path.exists() {
                create_directory(&output_path)?;
            }
        }

        // Copy any asset we found previously into the same directory as the index.html
        for asset in &section.assets {
            let asset_path = asset.as_path();
            copy(
                &asset_path,
                &output_path.join(
                    asset_path.file_name().expect("Failed to get asset filename for section"),
                ),
            )?;
        }

        if render_pages {
            section
                .pages
                .par_iter()
                .map(|k| self.render_page(self.library.read().unwrap().get_page_by_key(*k)))
                .collect::<Result<()>>()?;
        }

        if !section.meta.render {
            return Ok(());
        }

        if let Some(ref redirect_to) = section.meta.redirect_to {
            let permalink = self.config.make_permalink(redirect_to);
            create_file(
                &output_path.join("index.html"),
                &render_redirect_template(&permalink, &self.tera)?,
            )?;
            return Ok(());
        }

        if section.meta.is_paginated() {
            self.render_paginated(
                &output_path,
                &Paginator::from_section(&section, &self.library.read().unwrap()),
            )?;
        } else {
            let output =
                section.render_html(&self.tera, &self.config, &self.library.read().unwrap())?;
            create_file(&output_path.join("index.html"), &self.inject_livereload(output))?;
        }

        Ok(())
    }

    // TODO: remove me when reload has changed
    /// Used only on reload
    pub fn render_index(&self) -> Result<()> {
        self.render_section(
            &self
                .library
                .read()
                .unwrap()
                .get_section(&self.content_path.join("_index.md"))
                .expect("Failed to get index section"),
            false,
        )
    }

    /// Renders all sections
    pub fn render_sections(&self) -> Result<()> {
        self.library
            .read()
            .unwrap()
            .sections_values()
            .into_par_iter()
            .map(|s| self.render_section(s, true))
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
    pub fn render_paginated(&self, output_path: &Path, paginator: &Paginator) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;

        let folder_path = output_path.join(&paginator.paginate_path);
        create_directory(&folder_path)?;

        paginator
            .pagers
            .par_iter()
            .map(|pager| {
                let page_path = folder_path.join(&format!("{}", pager.index));
                create_directory(&page_path)?;
                let output = paginator.render_pager(
                    pager,
                    &self.config,
                    &self.tera,
                    &self.library.read().unwrap(),
                )?;
                if pager.index > 1 {
                    create_file(&page_path.join("index.html"), &self.inject_livereload(output))?;
                } else {
                    create_file(&output_path.join("index.html"), &self.inject_livereload(output))?;
                    create_file(
                        &page_path.join("index.html"),
                        &render_redirect_template(&paginator.permalink, &self.tera)?,
                    )?;
                }
                Ok(())
            })
            .collect::<Result<()>>()
    }
}
