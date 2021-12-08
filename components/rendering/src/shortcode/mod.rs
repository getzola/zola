use std::collections::HashMap;

use errors::{Error, Result};
use utils::templates::{ShortcodeDefinition, ShortcodeFileType};

mod parser;

pub(crate) use parser::{parse_for_shortcodes, Shortcode, SHORTCODE_PLACEHOLDER};

/// Extracts the shortcodes present in the source, check if we know them and errors otherwise
pub fn extract_shortcodes(
    source: &str,
    definitions: &HashMap<String, ShortcodeDefinition>,
) -> Result<(String, Vec<Shortcode>)> {
    let (out, mut shortcodes) = parse_for_shortcodes(source)?;

    for sc in &mut shortcodes {
        if let Some(def) = definitions.get(&sc.name) {
            sc.tera_name = def.tera_name.clone();
        } else {
            return Err(Error::msg(format!("Found usage of a shortcode named `{}` but we do not know about. Make sure it's not a typo and that a field name `{}.{{html,md}} exists in the `templates/shortcodes` directory.", sc.name, sc.name)));
        }
    }

    Ok((out, shortcodes))
}

pub fn insert_md_shortcodes(
    mut content: String,
    shortcodes: Vec<Shortcode>,
    tera_context: &tera::Context,
    tera: &tera::Tera,
) -> Result<(String, Vec<Shortcode>)> {
    // (span, len transformed)
    let mut transforms = Vec::new();
    let mut html_shortcodes = Vec::with_capacity(shortcodes.len());

    for mut sc in shortcodes.into_iter() {
        for (md_sc_span, rendered_length) in &transforms {
            sc.update_range(md_sc_span, *rendered_length);
        }

        if sc.file_type() == ShortcodeFileType::Html {
            html_shortcodes.push(sc);
            continue;
        }

        let span = sc.span.clone();
        let res = sc.render(tera, tera_context)?;
        transforms.push((span.clone(), res.len()));
        content.replace_range(span, &res);
    }

    Ok((content, html_shortcodes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shortcode::SHORTCODE_PLACEHOLDER;
    use tera::to_value;

    #[test]
    fn can_insert_md_shortcodes() {
        let mut tera = templates::ZOLA_TERA.clone();

        tera.add_raw_template("shortcodes/a.md", "{{ nth }}").unwrap();
        tera.add_raw_template("shortcodes/bodied.md", "{{ body }}").unwrap();

        let tera_context = tera::Context::new();
        assert_eq!(
            insert_md_shortcodes(
                format!("{}{}", SHORTCODE_PLACEHOLDER, SHORTCODE_PLACEHOLDER),
                vec![
                    Shortcode {
                        name: "a".to_string(),
                        args: to_value(&HashMap::<u8, u8>::new()).unwrap(),
                        span: 0..SHORTCODE_PLACEHOLDER.len(),
                        body: None,
                        nth: 1,
                        tera_name: "shortcodes/a.md".to_owned(),
                    },
                    Shortcode {
                        name: "a".to_string(),
                        args: to_value(&HashMap::<u8, u8>::new()).unwrap(),
                        span: SHORTCODE_PLACEHOLDER.len()..(2 * SHORTCODE_PLACEHOLDER.len()),
                        body: None,
                        nth: 2,
                        tera_name: "shortcodes/a.md".to_owned(),
                    }
                ],
                &tera_context,
                &tera
            )
            .unwrap()
            .0,
            "12".to_string()
        );

        assert_eq!(
            insert_md_shortcodes(
                format!("Much wow {}", SHORTCODE_PLACEHOLDER),
                vec![Shortcode {
                    name: "bodied".to_string(),
                    args: to_value(&HashMap::<u8, u8>::new()).unwrap(),
                    span: 9..(9 + SHORTCODE_PLACEHOLDER.len()),
                    body: Some("Content of the body".to_owned()),
                    nth: 1,

                    tera_name: "shortcodes/bodied.md".to_owned(),
                },],
                &tera_context,
                &tera
            )
            .unwrap()
            .0,
            "Much wow Content of the body".to_string()
        );
    }
}
