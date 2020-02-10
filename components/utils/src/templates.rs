use std::collections::HashMap;

use tera::{Context, Tera};

use errors::{bail, Result};

static DEFAULT_TPL: &str = include_str!("default_tpl.html");

macro_rules! render_default_tpl {
    ($filename: expr, $url: expr) => {{
        let mut context = Context::new();
        context.insert("filename", $filename);
        context.insert("url", $url);
        Tera::one_off(DEFAULT_TPL, &context, true).map_err(std::convert::Into::into)
    }};
}

/// Renders the given template with the given context, but also ensures that, if the default file
/// is not found, it will look up for the equivalent template for the current theme if there is one.
/// Lastly, if it's a default template (index, section or page), it will just return an empty string
/// to avoid an error if there isn't a template with that name
pub fn render_template(
    name: &str,
    tera: &Tera,
    context: Context,
    theme: &Option<String>,
) -> Result<String> {
    // check if it is in the templates
    if tera.templates.contains_key(name) {
        return tera.render(name, &context).map_err(std::convert::Into::into);
    }

    // check if it is part of a theme
    if let Some(ref t) = *theme {
        let theme_template_name = format!("{}/templates/{}", t, name);
        if tera.templates.contains_key(&theme_template_name) {
            return tera.render(&theme_template_name, &context).map_err(std::convert::Into::into);
        }
    }

    // check if it is part of ZOLA_TERA defaults
    let default_name = format!("__zola_builtins/{}", name);
    if tera.templates.contains_key(&default_name) {
        return tera.render(&default_name, &context).map_err(std::convert::Into::into);
    }

    // maybe it's a default one?
    match name {
        "index.html" | "section.html" => render_default_tpl!(
            name,
            "https://www.getzola.org/documentation/templates/pages-sections/#section-variables"
        ),
        "page.html" => render_default_tpl!(
            name,
            "https://www.getzola.org/documentation/templates/pages-sections/#page-variables"
        ),
        "single.html" | "list.html" => {
            render_default_tpl!(name, "https://www.getzola.org/documentation/templates/taxonomies/")
        }
        _ => bail!("Tried to render `{}` but the template wasn't found", name),
    }
}

/// Rewrites the path from extend/macros of the theme used to ensure
/// that they will point to the right place (theme/templates/...)
/// Include is NOT supported as it would be a pain to add and using blocks
/// or macros is always better anyway for themes
/// This will also rename the shortcodes to NOT have the themes in the path
/// so themes shortcodes can be used.
pub fn rewrite_theme_paths(tera_theme: &mut Tera, site_templates: Vec<&str>, theme: &str) {
    let mut shortcodes_to_move = vec![];
    let mut templates = HashMap::new();
    let old_templates = ::std::mem::replace(&mut tera_theme.templates, HashMap::new());

    // We want to match the paths in the templates to the new names
    for (key, mut tpl) in old_templates {
        tpl.name = format!("{}/templates/{}", theme, tpl.name);
        // First the parent if there is one
        // If a template with the same name is also in site, assumes it overrides the theme one
        // and do not change anything
        if let Some(ref p) = tpl.parent.clone() {
            if !site_templates.contains(&p.as_ref()) {
                tpl.parent = Some(format!("{}/templates/{}", theme, p));
            }
        }

        // Next the macros import
        let mut updated = vec![];
        for &(ref filename, ref namespace) in &tpl.imported_macro_files {
            updated.push((format!("{}/templates/{}", theme, filename), namespace.to_string()));
        }
        tpl.imported_macro_files = updated;

        if tpl.name.starts_with(&format!("{}/templates/shortcodes", theme)) {
            let new_name = tpl.name.replace(&format!("{}/templates/", theme), "");
            shortcodes_to_move.push((key, new_name.clone()));
            tpl.name = new_name;
        }

        templates.insert(tpl.name.clone(), tpl);
    }

    tera_theme.templates = templates;

    // and then replace shortcodes in the Tera instance using the new names
    for (old_name, new_name) in shortcodes_to_move {
        let tpl = tera_theme.templates.remove(&old_name).unwrap();
        tera_theme.templates.insert(new_name, tpl);
    }
}

#[cfg(test)]
mod tests {
    use super::rewrite_theme_paths;
    use tera::Tera;

    #[test]
    fn can_rewrite_all_paths_of_theme() {
        let mut tera = Tera::parse("test-templates/*.html").unwrap();
        rewrite_theme_paths(&mut tera, vec!["base.html"], "hyde");
        // special case to make the test work: we also rename the files to
        // match the imports
        for (key, val) in &tera.templates.clone() {
            tera.templates.insert(format!("hyde/templates/{}", key), val.clone());
        }
        // Adding our fake base
        tera.add_raw_template("base.html", "Hello").unwrap();
        tera.build_inheritance_chains().unwrap();

        assert_eq!(
            tera.templates["hyde/templates/index.html"].parent,
            Some("base.html".to_string())
        );
        assert_eq!(
            tera.templates["hyde/templates/child.html"].parent,
            Some("hyde/templates/index.html".to_string())
        );
    }
}
