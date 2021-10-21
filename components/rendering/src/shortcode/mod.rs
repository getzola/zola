#[macro_use]
mod util;

mod arg_value;
mod inner_tag;
mod parse;
mod string_literal;

use crate::transform::Transform;
use arg_value::ToJsonConvertError;
pub use parse::{fetch_shortcodes, ShortcodeContext};

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ShortcodeFileType {
    Markdown,
    Html,
}

#[derive(Debug, Clone)]
pub struct ShortcodeDefinition {
    pub file_type: ShortcodeFileType,
    pub tera_name: String,
}

impl ShortcodeDefinition {
    pub fn new(file_type: ShortcodeFileType, tera_name: &str) -> ShortcodeDefinition {
        let tera_name = tera_name.to_string();

        ShortcodeDefinition { file_type, tera_name }
    }
}

#[derive(Debug, PartialEq)]
pub enum RenderError {
    VariableNotFound { complete_var: Vec<String>, specific_part: String },
    FloatParseError,
    TeraError,
}

impl From<ToJsonConvertError> for RenderError {
    fn from(err: ToJsonConvertError) -> Self {
        match err {
            ToJsonConvertError::VariableNotFound { complete_var, specific_part } => {
                RenderError::VariableNotFound { complete_var, specific_part }
            }
            ToJsonConvertError::FloatParseError => RenderError::FloatParseError,
        }
    }
}

impl Into<errors::Error> for RenderError {
    fn into(self) -> errors::Error {
        // TODO: Improve this conversion
        errors::Error::msg("Something went wrong whilst rendering shortcodes")
    }
}

/// Take one specific shortcode and attempt to turn it into its resulting replacement string
pub fn render_shortcode(
    context: &ShortcodeContext,
    invocation_counts: &mut HashMap<String, usize>,
    tera_context: &tera::Context,
    tera: &tera::Tera,
) -> Result<String, RenderError> {
    let body_content = context.body();

    let mut new_context = tera::Context::new();

    for (key, value) in context.args().iter() {
        new_context.insert(key, &value.to_tera(&new_context)?);
    }
    if let Some(body_content) = body_content {
        // Trimming right to avoid most shortcodes with bodies ending up with a HTML new line
        new_context.insert("body", body_content.trim_end());
    }

    // We don't have to take into account the call stack, since we know for sure that it will not
    // contain this shortcode again.
    let invocation_count = *invocation_counts.get(context.name()).unwrap_or(&1);
    new_context.insert("nth", &invocation_count);
    invocation_counts.insert(context.name().to_string(), invocation_count + 1);

    new_context.extend(tera_context.clone());

    let res = utils::templates::render_template(context.tera_name(), tera, new_context, &None)
        .map_err(|e| {
            errors::Error::chain(format!("Failed to render {} shortcode", context.tera_name()), e)
        })
        .map_err(|_| RenderError::TeraError)?;

    Ok(res)
}

/// Inserts shortcodes of file type `filter_file_type` (recursively) into a source string
pub fn insert_md_shortcodes(
    mut content: String,
    shortcode_ctxs: Vec<ShortcodeContext>,
    tera_context: &tera::Context,
    tera: &tera::Tera,
) -> Result<(String, Vec<ShortcodeContext>), RenderError> {
    let mut invocation_counts = HashMap::new();
    let mut transforms = Vec::new();

    let mut html_shortcode_ctxs = Vec::with_capacity(shortcode_ctxs.len());

    for mut ctx in shortcode_ctxs.into_iter() {
        for Transform { span_start, initial_end, after_end } in transforms.iter() {
            ctx.update_on_source_insert(*span_start, *initial_end, *after_end);
        }

        if ctx.file_type() == &ShortcodeFileType::Html {
            html_shortcode_ctxs.push(ctx);
            continue;
        }

        let ctx_span = ctx.span();
        let res = render_shortcode(&ctx, &mut invocation_counts, tera_context, tera)?;

        transforms.push(Transform::new(ctx_span, res.len()));
        content.replace_range(ctx_span.clone(), &res);
    }

    Ok((content, html_shortcode_ctxs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ShortcodeFileType::*;

    macro_rules! assert_render_md_shortcode {
        ($source:expr, ($context:expr, [$($template_name:expr => $template_output:expr),*$(,)?]) => $res:expr) => {
            let mut tera = templates::ZOLA_TERA.clone();
            tera.add_raw_templates(vec![$(($template_name, $template_output)),*]).unwrap();

            let context = tera::Context::new();
            let mut invocation_counts = HashMap::new();

            assert_eq!(
                render_shortcode(&$context, &mut invocation_counts, &context, &tera).unwrap(),
                $res.to_string()
            );
        };
    }

    #[test]
    fn render_md_shortcode() {
        assert_render_md_shortcode!(
            "abc {{SC()}}",
            (
                ShortcodeContext::new("a", vec![], 4..12, None, Markdown, "shortcodes/a.md"),
                [
                    "shortcodes/a.md" => "wow"
                ]
            ) => "wow"
        );
        assert_render_md_shortcode!(
             "abc {{SC()}}",
             (
                 ShortcodeContext::new("a", vec![], 4..12, None, Markdown, "shortcodes/a.md"),
                 ["shortcodes/a.md" => "XYZ"]
             ) => "XYZ"
        );
        assert_render_md_shortcode!(
             "abc {{SC()}}",
             (
                 ShortcodeContext::new("a", vec![], 4..12, None, Markdown, "shortcodes/a.md"),
                 ["shortcodes/a.md" => "{{ nth }}"]
             ) => "1"
        );
    }

    #[test]
    fn insrt_md_shortcodes() {
        let mut tera = templates::ZOLA_TERA.clone();

        tera.add_raw_template("shortcodes/a.md", "{{ nth }}").unwrap();
        tera.add_raw_template("shortcodes/bodied.md", "{{ body }}").unwrap();

        let tera_context = tera::Context::new();
        assert_eq!(
            insert_md_shortcodes(
                "{{SC()}}{{SC()}}".to_string(),
                vec![
                    ShortcodeContext::new("", vec![], 0..8, None, Markdown, "shortcodes/a.md"),
                    ShortcodeContext::new("", vec![], 8..16, None, Markdown, "shortcodes/a.md")
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
                "Much wow {{SC()}}".to_string(),
                vec![ShortcodeContext::new(
                    "",
                    vec![],
                    9..17,
                    Some("Content of the body"),
                    Markdown,
                    "shortcodes/bodied.md"
                )],
                &tera_context,
                &tera
            )
            .unwrap()
            .0,
            "Much wow Content of the body".to_string()
        );
    }

    #[test]
    fn does_nothing_with_no_shortcodes() {
        let mut tera = templates::ZOLA_TERA.clone();

        tera.add_raw_template("shortcodes/a.md", "{{ nth }}").unwrap();
        tera.add_raw_template("shortcodes/bodied.md", "{{ bodied }}").unwrap();

        let tera_context = tera::Context::new();
        assert_eq!(
            insert_md_shortcodes("{{SC()}}{{SC()}}".to_string(), vec![], &tera_context, &tera)
                .unwrap()
                .0,
            "{{SC()}}{{SC()}}".to_string()
        );
    }
}
