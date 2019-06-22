extern crate glob;
extern crate rayon;
extern crate serde;
extern crate tera;
#[macro_use]
extern crate serde_derive;
extern crate sass_rs;

#[macro_use]
extern crate errors;
extern crate config;
extern crate front_matter;
extern crate imageproc;
extern crate library;
extern crate link_checker;
extern crate search;
extern crate templates;
extern crate utils;

#[cfg(test)]
extern crate tempfile;

mod sitemap;

use std::collections::HashMap;
use std::fs::{copy, create_dir_all, remove_dir_all};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use glob::glob;
use rayon::prelude::*;
use sass_rs::{compile_file, Options as SassOptions, OutputStyle};
use tera::{Context, Tera};

use config::{get_config, Config};
use errors::{Error, ErrorKind, Result};
use front_matter::InsertAnchor;
use library::{
    find_taxonomies, sort_actual_pages_by_date, Library, Page, Paginator, Section, Taxonomy,
};
use link_checker::check_url;
use templates::{global_fns, render_redirect_template, ZOLA_TERA};
use utils::fs::{copy_directory, create_directory, create_file, ensure_directory_exists};
use utils::net::get_available_port;
use utils::templates::{render_template, rewrite_theme_paths};

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
}

impl Site {
    /// Parse a site at the given path. Defaults to the current dir
    /// Passing in a path is only used in tests
    pub fn new<P: AsRef<Path>>(path: P, config_file: &str) -> Result<Site> {
        let path = path.as_ref();
        let mut config = get_config(path, config_file);
        config.load_extra_syntaxes(path)?;

        let tpl_glob =
            format!("{}/{}", path.to_string_lossy().replace("\\", "/"), "templates/**/*.*ml");
        // Only parsing as we might be extending templates from themes and that would error
        // as we haven't loaded them yet
        let mut tera =
            Tera::parse(&tpl_glob).map_err(|e| Error::chain("Error parsing templates", e))?;
        if let Some(theme) = config.theme.clone() {
            // Grab data from the extra section of the theme
            config.merge_with_theme(&path.join("themes").join(&theme).join("theme.toml"))?;

            // Test that the templates folder exist for that theme
            let theme_path = path.join("themes").join(&theme);
            if !theme_path.join("templates").exists() {
                bail!("Theme `{}` is missing a templates folder", theme);
            }

            let theme_tpl_glob = format!(
                "{}/{}",
                path.to_string_lossy().replace("\\", "/"),
                format!("themes/{}/templates/**/*.*ml", theme)
            );
            let mut tera_theme = Tera::parse(&theme_tpl_glob)
                .map_err(|e| Error::chain("Error parsing templates from themes", e))?;
            rewrite_theme_paths(&mut tera_theme, &theme);
            // TODO: we do that twice, make it dry?
            if theme_path.join("templates").join("robots.txt").exists() {
                tera_theme
                    .add_template_file(theme_path.join("templates").join("robots.txt"), None)?;
            }
            tera_theme.build_inheritance_chains()?;
            tera.extend(&tera_theme)?;
        }
        tera.extend(&ZOLA_TERA)?;
        // the `extend` above already does it but hey
        tera.build_inheritance_chains()?;

        // TODO: Tera doesn't use globset right now so we can load the robots.txt as part
        // of the glob above, therefore we load it manually if it exists.
        if path.join("templates").join("robots.txt").exists() {
            tera.add_template_file(path.join("templates").join("robots.txt"), Some("robots.txt"))?;
        }

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
            // We will allocate it properly later on
            library: Arc::new(RwLock::new(Library::new(0, 0, false))),
        };

        Ok(site)
    }

    /// The index sections are ALWAYS at those paths
    /// There are one index section for the basic language + 1 per language
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

    /// Get the number of orphan (== without section) pages in the site
    pub fn get_number_orphan_pages(&self) -> usize {
        self.library.read().unwrap().get_all_orphan_pages().len()
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
            pages_insert_anchors.insert(
                p.file.path.clone(),
                self.find_parent_section_insert_anchor(&p.file.parent.clone(), &p.lang),
            );
            self.add_page(p, false)?;
        }

        // taxonomy Tera fns are loaded in `register_early_global_fns`
        // so we do need to populate it first.
        self.populate_taxonomies()?;
        self.register_early_global_fns();
        self.populate_sections();
        self.render_markdown()?;
        self.register_tera_global_fns();

        // Needs to be done after rendering markdown as we only get the anchors at that point
        self.check_internal_links_with_anchors()?;

        if self.config.check_external_links {
            self.check_external_links()?;
        }

        Ok(())
    }

    /// Very similar to check_external_links but can't be merged as far as I can see since we always
    /// want to check the internal links but only the external in zola check :/
    pub fn check_internal_links_with_anchors(&self) -> Result<()> {
        let library = self.library.write().expect("Get lock for check_internal_links_with_anchors");
        let page_links = library
            .pages()
            .values()
            .map(|p| {
                let path = &p.file.path;
                p.internal_links_with_anchors.iter().map(move |l| (path.clone(), l))
            })
            .flatten();
        let section_links = library
            .sections()
            .values()
            .map(|p| {
                let path = &p.file.path;
                p.internal_links_with_anchors.iter().map(move |l| (path.clone(), l))
            })
            .flatten();
        let all_links = page_links.chain(section_links).collect::<Vec<_>>();
        let mut full_path = self.base_path.clone();
        full_path.push("content");

        let errors: Vec<_> = all_links
            .iter()
            .filter_map(|(page_path, (md_path, anchor))| {
                // There are a few `expect` here since the presence of the .md file will
                // already have been checked in the markdown rendering
                let mut p = full_path.clone();
                for part in md_path.split('/') {
                    p.push(part);
                }
                if md_path.contains("_index.md") {
                    let section = library
                        .get_section(&p)
                        .expect("Couldn't find section in check_internal_links_with_anchors");
                    if section.has_anchor(&anchor) {
                        None
                    } else {
                        Some((page_path, md_path, anchor))
                    }
                } else {
                    let page = library
                        .get_page(&p)
                        .expect("Couldn't find section in check_internal_links_with_anchors");
                    if page.has_anchor(&anchor) {
                        None
                    } else {
                        Some((page_path, md_path, anchor))
                    }
                }
            })
            .collect();
        if errors.is_empty() {
            return Ok(());
        }
        let msg = errors
            .into_iter()
            .map(|(page_path, md_path, anchor)| {
                format!(
                    "The anchor in the link `@/{}#{}` in {} does not exist.",
                    md_path,
                    anchor,
                    page_path.to_string_lossy(),
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        Err(Error { kind: ErrorKind::Msg(msg.into()), source: None })
    }

    pub fn check_external_links(&self) -> Result<()> {
        let library = self.library.write().expect("Get lock for check_external_links");
        let page_links = library
            .pages()
            .values()
            .map(|p| {
                let path = &p.file.path;
                p.external_links.iter().map(move |l| (path.clone(), l))
            })
            .flatten();
        let section_links = library
            .sections()
            .values()
            .map(|p| {
                let path = &p.file.path;
                p.external_links.iter().map(move |l| (path.clone(), l))
            })
            .flatten();
        let all_links = page_links.chain(section_links).collect::<Vec<_>>();

        // create thread pool with lots of threads so we can fetch
        // (almost) all pages simultaneously
        let threads = std::cmp::min(all_links.len(), 32);
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .map_err(|e| Error { kind: ErrorKind::Msg(e.to_string().into()), source: None })?;

        let errors: Vec<_> = pool.install(|| {
            all_links
                .par_iter()
                .filter_map(|(page_path, link)| {
                    let res = check_url(&link);
                    if res.is_valid() {
                        None
                    } else {
                        Some((page_path, link, res))
                    }
                })
                .collect()
        });

        if errors.is_empty() {
            return Ok(());
        }
        let msg = errors
            .into_iter()
            .map(|(page_path, link, check_res)| {
                format!(
                    "Dead link in {} to {}: {}",
                    page_path.to_string_lossy(),
                    link,
                    check_res.message()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        Err(Error { kind: ErrorKind::Msg(msg.into()), source: None })
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
                    index_section.permalink = self.config.make_permalink(l);
                    let filename = format!("_index.{}.md", l);
                    index_section.file.path = self.content_path.join(&filename);
                    index_section.file.relative = filename;
                    index_section.lang = index_section.file.find_language(&self.config)?;
                } else {
                    index_section.file.name = "_index".to_string();
                    index_section.permalink = self.config.make_permalink("");
                    index_section.file.path = self.content_path.join("_index.md");
                    index_section.file.relative = "_index.md".to_string();
                }
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

    /// Adds global fns that are to be available to shortcodes while
    /// markdown
    pub fn register_early_global_fns(&mut self) {
        self.tera.register_function(
            "get_url",
            global_fns::GetUrl::new(self.config.clone(), self.permalinks.clone()),
        );
        self.tera.register_function(
            "resize_image",
            global_fns::ResizeImage::new(self.imageproc.clone()),
        );
        self.tera.register_function(
            "get_image_metadata",
            global_fns::GetImageMeta::new(self.content_path.clone()),
        );
        self.tera.register_function("load_data", global_fns::LoadData::new(self.base_path.clone()));
        self.tera.register_function("trans", global_fns::Trans::new(self.config.clone()));
        self.tera.register_function(
            "get_taxonomy_url",
            global_fns::GetTaxonomyUrl::new(&self.taxonomies),
        );
    }

    pub fn register_tera_global_fns(&mut self) {
        self.tera.register_function(
            "get_page",
            global_fns::GetPage::new(self.base_path.clone(), self.library.clone()),
        );
        self.tera.register_function(
            "get_section",
            global_fns::GetSection::new(self.base_path.clone(), self.library.clone()),
        );
        self.tera.register_function(
            "get_taxonomy",
            global_fns::GetTaxonomy::new(self.taxonomies.clone(), self.library.clone()),
        );
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
    fn inject_livereload(&self, html: String) -> String {
        if let Some(port) = self.live_reload {
            return html.replace(
                "</body>",
                &format!(
                    r#"<script src="/livereload.js?port={}&mindelay=10"></script></body>"#,
                    port
                ),
            );
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
            )?;
        }
        // We're fine with missing static folders
        if self.static_path.exists() {
            copy_directory(&self.static_path, &self.output_path)?;
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
                self.compile_sass(&theme_path)?;
            }
        }

        if self.config.compile_sass {
            self.compile_sass(&self.base_path)?;
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
        if self.config.generate_rss {
            let pages = if self.config.is_multilingual() {
                library
                    .pages_values()
                    .iter()
                    .filter(|p| p.lang == self.config.default_language)
                    .map(|p| *p)
                    .collect()
            } else {
                library.pages_values()
            };
            self.render_rss_feed(pages, None)?;
        }

        for lang in &self.config.languages {
            if !lang.rss {
                continue;
            }
            let pages =
                library.pages_values().iter().filter(|p| p.lang == lang.code).map(|p| *p).collect();
            self.render_rss_feed(pages, Some(&PathBuf::from(lang.code.clone())))?;
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
        // index first
        create_file(
            &self.output_path.join(&format!("search_index.{}.js", self.config.default_language)),
            &format!(
                "window.searchIndex = {};",
                search::build_index(&self.config.default_language, &self.library.read().unwrap())?
            ),
        )?;

        // then elasticlunr.min.js
        create_file(&self.output_path.join("elasticlunr.min.js"), search::ELASTICLUNR_JS)?;

        Ok(())
    }

    pub fn compile_sass(&self, base_path: &Path) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;

        let sass_path = {
            let mut sass_path = PathBuf::from(base_path);
            sass_path.push("sass");
            sass_path
        };

        let mut options = SassOptions::default();
        options.output_style = OutputStyle::Compressed;
        let mut compiled_paths = self.compile_sass_glob(&sass_path, "scss", &options.clone())?;

        options.indented_syntax = true;
        compiled_paths.extend(self.compile_sass_glob(&sass_path, "sass", &options)?);

        compiled_paths.sort();
        for window in compiled_paths.windows(2) {
            if window[0].1 == window[1].1 {
                bail!(
                    "SASS path conflict: \"{}\" and \"{}\" both compile to \"{}\"",
                    window[0].0.display(),
                    window[1].0.display(),
                    window[0].1.display(),
                );
            }
        }

        Ok(())
    }

    fn compile_sass_glob(
        &self,
        sass_path: &Path,
        extension: &str,
        options: &SassOptions,
    ) -> Result<Vec<(PathBuf, PathBuf)>> {
        let glob_string = format!("{}/**/*.{}", sass_path.display(), extension);
        let files = glob(&glob_string)
            .expect("Invalid glob for sass")
            .filter_map(|e| e.ok())
            .filter(|entry| {
                !entry.as_path().file_name().unwrap().to_string_lossy().starts_with('_')
            })
            .collect::<Vec<_>>();

        let mut compiled_paths = Vec::new();
        for file in files {
            let css = compile_file(&file, options.clone())?;

            let path_inside_sass = file.strip_prefix(&sass_path).unwrap();
            let parent_inside_sass = path_inside_sass.parent();
            let css_output_path = self.output_path.join(path_inside_sass).with_extension("css");

            if parent_inside_sass.is_some() {
                create_dir_all(&css_output_path.parent().unwrap())?;
            }

            create_file(&css_output_path, &css)?;
            compiled_paths.push((path_inside_sass.to_owned(), css_output_path));
        }

        Ok(compiled_paths)
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

    /// Renders all taxonomies with at least one non-draft post
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

                if taxonomy.kind.rss {
                    self.render_rss_feed(
                        item.pages.iter().map(|p| library.get_page_by_key(*p)).collect(),
                        Some(&PathBuf::from(format!("{}/{}", taxonomy.kind.name, item.slug))),
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
        let all_sitemap_entries = {
            let mut all_sitemap_entries =
                sitemap::find_entries(&library, &self.taxonomies[..], &self.config);
            all_sitemap_entries.sort();
            all_sitemap_entries
        };
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

    /// Renders a RSS feed for the given path and at the given path
    /// If both arguments are `None`, it will render only the RSS feed for the whole
    /// site at the root folder.
    pub fn render_rss_feed(
        &self,
        all_pages: Vec<&Page>,
        base_path: Option<&PathBuf>,
    ) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;

        let mut context = Context::new();
        let mut pages = all_pages
            .into_iter()
            .filter(|p| p.meta.date.is_some() && !p.is_draft())
            .collect::<Vec<_>>();

        // Don't generate a RSS feed if none of the pages has a date
        if pages.is_empty() {
            return Ok(());
        }

        pages.par_sort_unstable_by(sort_actual_pages_by_date);

        context.insert("last_build_date", &pages[0].meta.date.clone());
        let library = self.library.read().unwrap();
        // limit to the last n elements if the limit is set; otherwise use all.
        let num_entries = self.config.rss_limit.unwrap_or_else(|| pages.len());
        let p = pages
            .iter()
            .take(num_entries)
            .map(|x| x.to_serialized_basic(&library))
            .collect::<Vec<_>>();

        context.insert("pages", &p);
        context.insert("config", &self.config);

        let rss_feed_url = if let Some(ref base) = base_path {
            self.config.make_permalink(&base.join("rss.xml").to_string_lossy().replace('\\', "/"))
        } else {
            self.config.make_permalink("rss.xml")
        };

        context.insert("feed_url", &rss_feed_url);

        let feed = &render_template("rss.xml", &self.tera, context, &self.config.theme)?;

        if let Some(ref base) = base_path {
            let mut output_path = self.output_path.clone();
            for component in base.components() {
                output_path.push(component);
                if !output_path.exists() {
                    create_directory(&output_path)?;
                }
            }
            create_file(&output_path.join("rss.xml"), feed)?;
        } else {
            create_file(&self.output_path.join("rss.xml"), feed)?;
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
