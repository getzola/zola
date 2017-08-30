use tera::{Tera, Context};

use errors::Result;

/// Renders the given template with the given context, but also ensures that, if the default file
/// is not found, it will look up for the equivalent template for the current theme if there is one.
/// Lastly, if it's a default template (index, section or page), it will just return an empty string
/// to avoid an error if there isn't a template with that name
pub fn render_template(name: &str, tera: &Tera, context: &Context, theme: Option<String>) -> Result<String> {
    if tera.templates.contains_key(name) {
        return tera
            .render(name, context)
            .map_err(|e| e.into());
    }

    if let Some(ref t) = theme {
        return tera
            .render(&format!("{}/templates/{}", t, name), context)
            .map_err(|e| e.into());
    }

    if name == "index.html" || name == "section.html" || name == "page.html" {
        return Ok(String::new());
    }

    bail!("Tried to render `{}` but the template wasn't found", name)
}


/// Rewrites the path from extend/macros of the theme used to ensure
/// that they will point to the right place (theme/templates/...)
/// Include is NOT supported as it would be a pain to add and using blocks
/// or macros is always better anyway for themes
pub fn rewrite_theme_paths(tera: &mut Tera, theme: &str) {
    // We want to match the paths in the templates to the new names
    for tpl in tera.templates.values_mut() {
        // First the parent if there is none
        if let Some(ref p) = tpl.parent.clone() {
            tpl.parent = Some(format!("{}/templates/{}", theme, p));
        }

        // Next the macros import
        let mut updated = vec![];
        for &(ref filename, ref namespace) in &tpl.imported_macro_files {
            updated.push((format!("{}/templates/{}", theme, filename), namespace.to_string()));
        }
        tpl.imported_macro_files = updated;
    }
}

#[cfg(test)]
mod tests {
    use tera::Tera;
    use super::{rewrite_theme_paths};

    #[test]
    fn can_rewrite_all_paths_of_theme() {
        let mut tera = Tera::parse("templates/*.html").unwrap();
        rewrite_theme_paths(&mut tera, "hyde");
        // special case to make the test work: we also rename the files to
        // match the imports
        for (key, val) in tera.templates.clone() {
            tera.templates.insert(format!("hyde/templates/{}", key), val.clone());
        }
        tera.build_inheritance_chains().unwrap();
    }
}
