use std::str;

use katex;
use pcre2::bytes::{Regex, RegexBuilder};

pub fn render_katex(content: &str) -> String {
    let mut re_builder = RegexBuilder::new();
    let re_options = re_builder
        .multi_line(true)
        .extended(true)
        .ucp(true) // unicode
        .jit_if_available(true);
    let inline_math_re = re_options.clone()
        .build(
            r"(?<![\\\$])\$ # non-escaped opening dollar and non-double-dollar
            (
              [^\s\$] # immediately followed by a non-whitespace character
              [^\$]*
              (?<![\\\s\$]) # closing dollar is immediately preceeded by a non-whitespace,
                            # non-backslash character
            )
            \$(?![\d\$]) # closing dollar is not immediately followed by a digit or another dollar"
        ).unwrap();
    let display_math_re = re_options
        .build(
            r"(?<!\\)\$\$ # opening double-dollar not preceeded by a backslash
            (?=[^\s]|\h*\n\h*[^\$\s]) # either no whitespace, or a single newline
                                      # followed by a non-empty line
            ([^\$]*[^\s\$]) # any amount of characters not ending in whitespace
            (?:\h*\n\h*)? # a possibly empty line before closing dollars
            \$\$"
        ).unwrap();


    let inline = render_katex_aux(content, inline_math_re, false);
    render_katex_aux(&inline.to_owned(), display_math_re, true)
}

fn render_katex_aux(content: &str, rex: Regex, display: bool) -> String {
    let cont_bytes = content.as_bytes();
    let k_opts = katex::Opts::builder().display_mode(display).build().unwrap();
    let mut last: usize = 0;
    let mut with_katex = String::with_capacity(content.len());
    for ma in rex.captures_iter(cont_bytes) {
        if let Ok(mb) = ma {
            let replace = mb.get(0).unwrap();
            let tex = mb.get(1).unwrap();
            with_katex.push_str(str::from_utf8(&cont_bytes[last..replace.start()]).unwrap());
            last = replace.end();
            let s = str::from_utf8(&cont_bytes[tex.start()..tex.end()]).unwrap();
            let k_html = katex::render_with_opts(s, k_opts.clone()).unwrap();
            with_katex.push_str(&k_html);
            // println!("{:?}", k_html);
        }
    }
    with_katex.push_str(str::from_utf8(&cont_bytes[last..]).unwrap());
    with_katex
}


#[cfg(test)]
mod tests {
    use super::*;

    fn unchanged(eg: &str) {
        assert_eq!(eg, render_katex(eg));
    }

    fn changed(eg: &str) {
        assert_ne!(eg, render_katex(eg));
    }

    #[test]
    fn no_math_unchanged() {
        unchanged("This is just a sentence.");
    }

    #[test]
    fn price_not_math_unchanged() {
        unchanged("This has a number that is not math $3 000");
    }

    #[test]
    fn two_consecutive_prices_unchanged() {
        unchanged("Here are two consecutive prices with no whitspace $50$60");
        unchanged("Here are two consecutive prices with whitspace $50 $60");
        unchanged("Here are two consecutive prices with comma $50,$60");
    }

    #[test]
    fn backslash_proceeding_dollar_unchanged() {
        unchanged(r"\$F = ma$");
        unchanged(r"$F = ma\$");
    }

    #[test]
    fn double_dollar_unchanged() {
        unchanged(r"$$F = ma$");
        unchanged(r"$F = ma$$");
    }

    #[test]
    fn internal_whitespace_padding_unchanged() {
        unchanged(r"$ F = ma$");
        unchanged(r"$F = ma $");
        unchanged(r"$$ \int_0^1 x^2 = \frac{1}{2}$$");
        unchanged(r"$$\int_0^1 x^2 = \frac{1}{2} $$");
        unchanged(
r"$$

\int_0^1 x^2 = \frac{1}{2}
$$"
        );
        unchanged(
r"$$
\int_0^1 x^2 = \frac{1}{2}

$$"
        );
    }

    #[test]
    fn bad_internal_dollar_unchanged() {
        unchanged(r"$$\int_0^1 x^2 = \frac{1}${2}$$");
    }

    #[test]
    fn double_dollar_escaped_unchanged() {
        unchanged(r"\$$\int_0^1 x^2 = \frac{1}{2}$$");
        unchanged(r"$\$\int_0^1 x^2 = \frac{1}{2}$$");
        unchanged(r"$$\int_0^1 x^2 = \frac{1}${2}\$$");
        unchanged(r"$$\int_0^1 x^2 = \frac{1}{2}$\$");
    }

    #[test]
    fn random_double_dollar_unchanged() {
        unchanged(r"Hey $$ planet");
    }

    #[test]
    fn working_inline() {
        let eg = r"Consider $π = \frac{1}{2}τ$ for a moment.";
        let result = render_katex(eg);
        assert_ne!(eg, result);
        assert_eq!(eg[..9], result[..9]);
        assert_eq!(eg[eg.len()-14..], result[result.len()-14..]);
    }

    #[test]
    fn working_multiline() {
        changed(
r"$$\sum_{i = 0}^n i = \frac{1}{2}n(n+1)$$"
        );
        changed(
r"    $$ 
        \sum_{i = 0}^n i = \frac{1}{2}n(n+1) 
    $$"
        );
    }
}
