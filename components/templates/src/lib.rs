pub mod filters;
pub mod global_fns;

use std::collections::HashSet;
use std::path::Path;

use config::Config;
use libs::once_cell::sync::Lazy;
use libs::regex::{Regex, escape as regex_escape};
use libs::tera::{Context, Tera};

use errors::{Context as ErrorContext, Result, bail};
use utils::templates::rewrite_theme_paths;

pub static ZOLA_TERA: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("__zola_builtins/404.html", include_str!("builtins/404.html")),
        ("__zola_builtins/vault.html", include_str!("builtins/vault.html")),
        ("__zola_builtins/atom.xml", include_str!("builtins/atom.xml")),
        ("__zola_builtins/rss.xml", include_str!("builtins/rss.xml")),
        ("__zola_builtins/sitemap.xml", include_str!("builtins/sitemap.xml")),
        ("__zola_builtins/robots.txt", include_str!("builtins/robots.txt")),
        (
            "__zola_builtins/split_sitemap_index.xml",
            include_str!("builtins/split_sitemap_index.xml"),
        ),
        ("__zola_builtins/anchor-link.html", include_str!("builtins/anchor-link.html")),
        ("__zola_builtins/summary-cutoff.html", include_str!("builtins/summary-cutoff.html")),
        ("internal/alias.html", include_str!("builtins/internal/alias.html")),
    ])
    .unwrap();
    tera.register_filter("base64_encode", filters::base64_encode);
    tera.register_filter("base64_decode", filters::base64_decode);
    tera.register_filter("regex_replace", filters::RegexReplaceFilter::new());
    tera
});

/// Renders the `internal/alias.html` template that will redirect
/// via refresh to the url given
pub fn render_redirect_template(url: &str, tera: &Tera) -> Result<String> {
    let mut context = Context::new();
    context.insert("url", &url);

    tera.render("internal/alias.html", &context)
        .with_context(|| format!("Failed to render alias for '{}'", url))
}

pub fn load_tera(path: &Path, config: &Config) -> Result<Tera> {
    let tpl_glob = format!(
        "{}/{}",
        path.to_string_lossy().replace('\\', "/"),
        "templates/**/*.{*ml,md,txt,json,ics}"
    );

    // Only parsing as we might be extending templates from themes and that would error
    // as we haven't loaded them yet
    let mut tera =
        Tera::parse(&tpl_glob).context("Error parsing templates from the /templates directory")?;

    if let Some(ref theme) = config.theme {
        // Test that the templates folder exist for that theme
        let theme_path = path.join("themes").join(theme);
        if !theme_path.join("templates").exists() {
            bail!("Theme `{}` is missing a templates folder", theme);
        }

        let theme_tpl_glob = format!(
            "{}/themes/{}/templates/**/*.{{*ml,md}}",
            path.to_string_lossy().replace('\\', "/"),
            theme
        );
        let mut tera_theme =
            Tera::parse(&theme_tpl_glob).context("Error parsing templates from themes")?;
        rewrite_theme_paths(&mut tera_theme, theme);

        // TODO: add tests for theme-provided robots.txt (https://github.com/getzola/zola/pull/1722)
        if theme_path.join("templates").join("robots.txt").exists() {
            tera_theme.add_template_file(
                theme_path.join("templates").join("robots.txt"),
                Some("robots.txt"),
            )?;
        }
        tera.extend(&tera_theme)?;
    }
    tera.extend(&ZOLA_TERA)?;

    // Inject | safe filter for shortcode function calls
    // This must happen before build_inheritance_chains()
    inject_safe_for_shortcodes(&mut tera, path)?;

    tera.build_inheritance_chains()?;

    if path.join("templates").join("robots.txt").exists() {
        tera.add_template_file(path.join("templates").join("robots.txt"), Some("robots.txt"))?;
    }

    Ok(tera)
}

/// Inject `| safe` filter for shortcode function calls in templates
/// This runs after templates are loaded but before build_inheritance_chains()
fn inject_safe_for_shortcodes(tera: &mut Tera, base_path: &Path) -> Result<()> {
    // Step 1: Scan templates to find shortcode names
    let mut shortcode_names = HashSet::new();
    for (_, template) in tera.templates.iter() {
        if template.name.starts_with("shortcodes/") {
            let name = template.name.strip_prefix("shortcodes/").unwrap();
            // Remove .md or .html extension
            let name_without_ext = if let Some(idx) = name.rfind('.') {
                &name[..idx]
            } else {
                name
            };
            shortcode_names.insert(name_without_ext.to_string());
        } else if template.name.starts_with("__zola_builtins/shortcodes/") {
            let name = template.name.strip_prefix("__zola_builtins/shortcodes/").unwrap();
            let name_without_ext = if let Some(idx) = name.rfind('.') {
                &name[..idx]
            } else {
                name
            };
            shortcode_names.insert(name_without_ext.to_string());
        }
    }

    if shortcode_names.is_empty() {
        return Ok(());
    }

    // Step 2: Build regex patterns for each shortcode
    let shortcode_vec: Vec<String> = shortcode_names.into_iter().collect();

    // Step 3: Get list of non-shortcode templates that need modification
    let templates_to_modify: Vec<(String, String)> = tera
        .templates
        .iter()
        .filter(|(_, template)| {
            // Don't modify shortcode templates themselves
            !template.name.starts_with("shortcodes/")
                && !template.name.starts_with("__zola_builtins/")
                && !template.name.starts_with("internal/")
        })
        .filter_map(|(key, template)| {
            // Try to reconstruct the file path
            let file_path = if template.name.contains("theme") {
                // This is from a theme, skip it for now (can't easily reconstruct path)
                None
            } else {
                // Regular template from base_path/templates/
                let template_path = base_path.join("templates").join(&template.name);
                if template_path.exists() {
                    Some(template_path)
                } else {
                    None
                }
            };

            file_path.map(|path| (key.clone(), path.to_string_lossy().to_string()))
        })
        .collect();

    // Step 4: Read, modify, and re-add each template
    for (template_key, file_path) in templates_to_modify {
        let content = std::fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read template file: {}", file_path))?;

        let modified_content = inject_safe_filter(&content, &shortcode_vec);

        // Only re-add if content actually changed
        if modified_content != content {
            tera.add_raw_template(&template_key, &modified_content)
                .with_context(|| format!("Failed to re-add modified template: {}", template_key))?;
        }
    }

    Ok(())
}

/// Automatically inject `| safe` filter for shortcode function calls
fn inject_safe_filter(template_source: &str, shortcode_names: &[String]) -> String {
    if shortcode_names.is_empty() {
        return template_source.to_string();
    }

    let mut result = template_source.to_string();

    for shortcode_name in shortcode_names {
        // Pattern matches {{ shortcode_name(...) }} without | safe already present
        //  \{\{ matches {{
        //  \s* matches optional whitespace
        //  ({name}) captures the shortcode name
        //  \s*\( matches optional whitespace and opening paren
        //  (?:[^}]|\}(?!\}))*? matches content, handling single } but stopping at }}
        //  \) matches closing paren
        //  (?!\s*\|[^}]*\bsafe\b) negative lookahead: not followed by | ... safe
        //  (\s*\}\}) captures closing }} with optional whitespace
        // Pattern: match {{ name(...) }} and capture name, args, and closing braces separately
        // In raw string r"...", \{ matches literal {, so \{\{ matches {{
        let opening = r"\{\{";  // matches {{
        let closing = r"\}\}";  // matches }}
        let pattern = format!(
            r"{opening}\s*({name})\s*(\([^)]*\))(\s*{closing})",
            opening = opening,
            closing = closing,
            name = regex_escape(shortcode_name)
        );

        if let Ok(re) = Regex::new(&pattern) {
            // Replace: "{{ $1$2 | safe$3" where $1=name, $2=(args), $3=}}
            let new_result = re.replace_all(&result, "{{ $1$2 | safe$3").to_string();
            if new_result != result {
                result = new_result;
            }
        }
    }

    result
}
