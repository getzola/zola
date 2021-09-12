mod common;

use common::ShortCode;

macro_rules! test_links {
    (
        $in_str:literal,
        [$($id:literal => $perma_link:literal),*],
        [$($abs_path:literal),*],
        [$($rel_path:literal:$opt_anchor:expr),*],
        [$($shortcodes:ident),*]
    ) => {
        let config = config::Config::default_for_test();

        #[allow(unused_mut)]
        let mut tera = tera::Tera::default();

        // Add all shortcodes
        $(
            tera.add_raw_template(
                &format!("shortcodes/{}", $shortcodes.filename()),
                $shortcodes.output
            ).expect("Failed to add raw template");
        )*

        #[allow(unused_mut)]
        let mut permalinks = std::collections::HashMap::new();

        $(
            permalinks.insert($id.to_string(), $perma_link.to_string());
        )*

        let context = rendering::RenderContext::new(
            &tera,
            &config,
            &config.default_language,
            "",
            &permalinks,
            front_matter::InsertAnchor::None,
        );

        let rendered = rendering::render_content($in_str, &context);
        assert!(rendered.is_ok(), "Rendering failed");

        let rendered = rendered.unwrap();

        let asserted_int_links = vec![
            $(
                ($rel_path.to_string(), $opt_anchor.map(|x| x.to_string()))
            ),*
        ];
        let asserted_ext_links: Vec<&str> = vec![$($abs_path),*];

        assert_eq!(rendered.internal_links, asserted_int_links, "Internal links unequal");
        assert_eq!(rendered.external_links, asserted_ext_links, "External links unequal");
    }
}

#[test]
fn basic_internal() {
    test_links!(
        "Hello World!",
        [],
        [],
        [],
        []
    );
}

#[test]
fn absolute_links() {
    test_links!(
        "[abc](https://google.com/)",
        [],
        ["https://google.com/"],
        [],
        []
    );
}


#[test]
fn relative_links() {
    test_links!(
        "[abc](@/def/123.md)",
        ["def/123.md" => "https://xyz.com/def/123"],
        [],
        ["def/123.md":<Option<&str>>::None],
        []
    );
}

#[test]
#[should_panic]
fn relative_links_no_perma() {
    test_links!(
        "[abc](@/def/123.md)",
        [],
        [],
        ["def/123.md":<Option<&str>>::None],
        []
    );
}
