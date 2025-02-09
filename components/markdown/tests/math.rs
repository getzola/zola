use config::Config;

mod common;

fn default_config_math_typst() -> Config {
    let mut config = Config::default_for_test();
    config.markdown.math = config::MathRendering::Typst;
    config
}

#[test]
fn can_render_typst_inline_math() {
    let res = common::render_with_config(
        r#"This is an inline math $a^2 + b^2 = c^2$ and some more text"#,
        default_config_math_typst(),
    )
    .unwrap();

    println!("{}", res.body);

    assert!(res.body.contains(r#"<img"#));
    assert!(res.body.contains(r#"class="typst-inline typst-doc""#));
    assert!(res.body.contains(r#"src="data:image/svg+xml"#));
}

#[test]
fn can_render_typst_block_math() {
    let res = common::render_with_config(
        r#"
This is a block math

$$
a^2 + b^2 = c^2
$$

and some more text"#,
        default_config_math_typst(),
    )
    .unwrap();

    println!("{}", res.body);

    assert!(res.body.contains(r#"<img"#));
    assert!(res.body.contains(r#"class="typst-display typst-doc""#));
    assert!(res.body.contains(r#"src="data:image/svg+xml"#));
}
