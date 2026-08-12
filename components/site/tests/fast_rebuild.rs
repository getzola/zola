//! The `zola serve --fast` rebuild path.
//!
//! These go through `add_and_render_page` / `add_and_render_section`, which is
//! what `serve --fast` calls when a content file changes. They exist because
//! that path silently did nothing: the file was re-parsed and the render job
//! ran, but the renderer reads its values out of the `RenderCache`, which was
//! never refreshed — so the template was handed the copy serialized before the
//! edit and the server kept returning the old HTML, reporting "Done in 0ms".
//!
//! The site is built in a temp directory rather than from the shared fixtures
//! at the repo root, because these tests have to edit content.

use std::path::{Path, PathBuf};

use fs_err as fs;
use site::Site;
use tempfile::{TempDir, tempdir};

const CONFIG: &str = r#"
base_url = "https://example.com"
"#;

const SECTION_TEMPLATE: &str = r#"<h1>{{ section.title }}</h1>
<ul>{% for page in section.pages %}<li>{{ page.title }}</li>{% endfor %}</ul>
"#;

const PAGE_TEMPLATE: &str = "<h1>{{ page.title }}</h1><div>{{ page.content | safe }}</div>";

/// A two-page section, built once, ready to be edited.
fn scaffold() -> (Site, TempDir, PathBuf) {
    let dir = tempdir().expect("create temp dir");
    let root = dir.path();

    fs::create_dir_all(root.join("content/blog")).unwrap();
    fs::create_dir_all(root.join("templates")).unwrap();
    fs::write(root.join("config.toml"), CONFIG).unwrap();
    fs::write(root.join("templates/section.html"), SECTION_TEMPLATE).unwrap();
    fs::write(root.join("templates/page.html"), PAGE_TEMPLATE).unwrap();
    fs::write(root.join("templates/index.html"), SECTION_TEMPLATE).unwrap();
    fs::write(root.join("content/_index.md"), "+++\n+++\n").unwrap();
    fs::write(root.join("content/blog/_index.md"), "+++\ntitle = \"Blog\"\n+++\n").unwrap();
    fs::write(
        root.join("content/blog/first.md"),
        "+++\ntitle = \"Original title\"\n+++\nOriginal body.\n",
    )
    .unwrap();
    fs::write(root.join("content/blog/second.md"), "+++\ntitle = \"Second\"\n+++\nBody.\n")
        .unwrap();

    let config_file = root.join("config.toml");
    let mut site = Site::new(root, &config_file).unwrap();
    site.load().unwrap();
    let public = root.join("public");
    site.set_output_path(&public);
    site.build().expect("initial build");

    (site, dir, public)
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

#[test]
fn fast_rebuild_renders_the_edited_page() {
    let (mut site, dir, public) = scaffold();
    let page_path = dir.path().join("content/blog/first.md");
    let output = public.join("blog/first/index.html");

    assert!(read(&output).contains("Original title"));

    fs::write(&page_path, "+++\ntitle = \"Edited title\"\n+++\nEdited body.\n").unwrap();
    site.add_and_render_page(&page_path).expect("fast rebuild of a page");

    let html = read(&output);
    assert!(html.contains("Edited title"), "front matter change not rendered: {html}");
    assert!(html.contains("Edited body"), "body change not rendered: {html}");
}

#[test]
fn fast_rebuild_renders_the_edited_section() {
    let (mut site, dir, public) = scaffold();
    let section_path = dir.path().join("content/blog/_index.md");
    let output = public.join("blog/index.html");

    assert!(read(&output).contains("Blog"));

    fs::write(&section_path, "+++\ntitle = \"Edited section\"\n+++\n").unwrap();
    site.add_and_render_section(&section_path).expect("fast rebuild of a section");

    let html = read(&output);
    assert!(html.contains("Edited section"), "section title not rendered: {html}");
    // The section's page list must survive a section-only rebuild.
    assert!(html.contains("Original title") && html.contains("Second"), "pages lost: {html}");
}

#[test]
fn fast_rebuild_of_a_page_updates_its_section_listing() {
    // The listing is the part `--fast` is documented to leave behind. Asserting
    // the current behaviour keeps that a decision rather than a surprise: if it
    // ever starts working, this test says so instead of quietly passing.
    let (mut site, dir, public) = scaffold();
    let page_path = dir.path().join("content/blog/first.md");
    let listing = public.join("blog/index.html");

    fs::write(&page_path, "+++\ntitle = \"Renamed\"\n+++\nBody.\n").unwrap();
    site.add_and_render_page(&page_path).expect("fast rebuild of a page");

    let html = read(&listing);
    assert!(
        html.contains("Original title") && !html.contains("Renamed"),
        "the section listing is no longer stale after a page rebuild — \
         --fast now updates containers, so update this test and the docs: {html}"
    );
}
