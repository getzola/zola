mod common;

use common::ShortCode;
use rendering::Heading;

#[derive(PartialEq, Debug)]
struct HelperHeader {
    title: String,
    children: Vec<HelperHeader>,
}

impl PartialEq<Heading> for HelperHeader {
    fn eq(&self, other: &Heading) -> bool {
        self.title == other.title && self.children == other.children
    }
}

macro_rules! hh {
    ($title:literal, [$($children:expr),*]) => {{
        HelperHeader {
            title: $title.to_string(),
            children: vec![$($children),*],
        }
    }}
}

macro_rules! test_toc {
    ($in_str:literal, $toc:expr, [$($shortcodes:ident),*]) => {
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

        let permalinks = std::collections::HashMap::new();
        let mut context = rendering::RenderContext::new(
            &tera,
            &config,
            &config.default_language,
            "",
            &permalinks,
            front_matter::InsertAnchor::None,
        );
        let shortcode_def = utils::templates::get_shortcodes(&tera);
        context.set_shortcode_definitions(&shortcode_def);

        let rendered = rendering::render_content($in_str, &context);
        assert!(rendered.is_ok());

        let rendered = rendered.unwrap();
        let toc = rendered.toc.clone();

        assert!($toc == toc);
    }
}

#[test]
fn basic_toc() {
    test_toc!("Hello World!", <Vec<HelperHeader>>::new(), []);
    test_toc!("# ABC\n## DEF", vec![hh!("ABC", [hh!("DEF", [])])], []);
}

#[test]
fn all_layers() {
    test_toc!(
        "# A\n## B\n### C\n#### D\n##### E\n###### F\n",
        vec![hh!("A", [hh!("B", [hh!("C", [hh!("D", [hh!("E", [hh!("F", [])])])])])])],
        []
    );
}

#[test]
fn multiple_on_layer() {
    test_toc!(
        "# A\n## B\n## C\n### D\n## E\n### F\n",
        vec![hh!("A", [hh!("B", []), hh!("C", [hh!("D", [])]), hh!("E", [hh!("F", [])])])],
        []
    );
}

// const MD_SIMPLE1: ShortCode = ShortCode::new("simple", "Hello World!", true);
// const MD_SIMPLE2: ShortCode = ShortCode::new("simple2", "Wow, much cool!", true);
//
// #[test]
// fn with_shortcode_titles() {
//     test_toc!(
//         "# {{ simple() }}\n## {{ simple2() }}\n### ABC\n#### {{ simple() }}\n",
//         vec![hh!(
//             "Hello World!",
//             [hh!("Wow, much cool!", [hh!("ABC", [hh!("Hello World!", [])])])]
//         )],
//         [MD_SIMPLE1, MD_SIMPLE2]
//     );
// }
//
// const MD_MULTILINE: ShortCode = ShortCode::new("multiline", "<div>\n    Wow!\n</div>", false);
//
// #[test]
// fn with_multiline_shortcodes() {
//     test_toc!(
//         "# {{ multiline() }}\n{{ multiline() }}\n## {{ multiline()() }}\n",
//         vec![hh!("Wow!", [hh!("Wow!", [])])],
//         [MD_MULTILINE]
//     );
// }
