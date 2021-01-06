use super::CodeBlockPass;

pub enum LineNumbers<T, N> {
    #[allow(unused)]
    RowPerLine {
        code_pass: T,
        num_pass: N,
        openned: bool
    },
    #[allow(unused)]
    TwoCell {
        code_pass: T,
        num_pass: N,
        code_cell: String,
        openned: bool
    },
    NoLineNumbers {
        code_pass: T
    },
    #[allow(unused)]
    CSSCounter {
        code_pass: T
    }
}
use LineNumbers::{ RowPerLine, TwoCell, NoLineNumbers, CSSCounter };
impl<T: CodeBlockPass, N: CodeBlockPass> CodeBlockPass for LineNumbers<T, N> {
    fn close_to_root(&mut self, output: &mut String) {
        match self {
            RowPerLine { openned, .. } => {
                // No need to close the code or num passes because they are closed on every line
                output.push_str("</table>");
                *openned = false;
            },
            TwoCell { code_cell, openned, code_pass, num_pass } => {
                num_pass.close_to_root(output);
                output.push_str("</td><td>");
                output.push_str(code_cell.as_str());
                code_pass.close_to_root(output);
                output.push_str("</td></tr></table>");
                *openned = false;
            },
            NoLineNumbers { code_pass } => {
                code_pass.close_to_root(output);
            },
            CSSCounter { .. } => {
                // CSS Counter closes it's spans on every line so there's nothing to do here.
            }
        }
    }
    fn handle_line(&mut self, output: &mut String, line_num: usize, input: &str) {
        match self {
            RowPerLine { code_pass, num_pass, openned} => {
                if !*openned {
                    output.push_str("<table>");
                    *openned = true;
                }
                output.push_str("<tr><td>");
                num_pass.handle_line(output, line_num, line_num.to_string().as_str());
                num_pass.close_to_root(output);
                output.push_str("</td><td>");
                code_pass.handle_line(output, line_num, input);
                code_pass.close_to_root(output);
                output.push_str("</td></tr>");
            },
            TwoCell { code_pass, num_pass, code_cell, openned } => {
                if !*openned {
                    output.push_str("<table><tr><td>");
                    *openned = true;
                }
                let line = line_num.to_string() + "\n";
                num_pass.handle_line(output, line_num, line.as_str());
                code_pass.handle_line(code_cell, line_num, input);
            },
            CSSCounter { code_pass } => {
                output.push_str("<span class=\"code-line\">");
                code_pass.handle_line(output, line_num, input);
                code_pass.close_to_root(output);
                output.push_str("</span>");
            },
            NoLineNumbers { code_pass } => {
                code_pass.handle_line(output, line_num, input);
            }
        }
    }
    fn pre_styles(&self, line_num: usize) -> Option<String> {
        match self {
            RowPerLine { code_pass, .. } |
            TwoCell { code_pass, .. } |
            NoLineNumbers { code_pass } => {
                code_pass.pre_styles(line_num)
            },
            CSSCounter { code_pass } => {
                let style = format!("counter-reset:line-numbers {};", line_num);
                code_pass.pre_styles(line_num)
                    .map(|other_styles| other_styles + style.as_str())
                    .or(Some(style))
            }
        }
    }
    // Pass Overrides up to parent passes
    fn pre_class(&self) -> Option<String> {
        match self {
            RowPerLine { code_pass, .. } |
            TwoCell { code_pass, .. } |
            CSSCounter { code_pass } |
            NoLineNumbers { code_pass } => {
                code_pass.pre_class()
            }
        }
    }
    fn mark_styles(&self) -> Option<String> {
        match self {
            RowPerLine { code_pass, .. } |
            TwoCell { code_pass, .. } |
            CSSCounter { code_pass } |
            NoLineNumbers { code_pass } => {
                code_pass.mark_styles()
            }
        }
    }
}

#[cfg(test)]
mod linenos_tests {
    use super::*;
    use super::super::*;

    #[test]
    fn simple() {
        let mut block = CodeBlock {
            remainder: String::new(),
            current_line: 1,
            passes: RowPerLine {
                code_pass: (),
                num_pass: (),
                openned: false
            }
        };
        let text = "\
foo
bar
bar
baz
";
        assert_eq!(
            block.begin(None) + &block.code(text) + &block.close(),
            "\
<pre><code>\
<table>\
<tr><td>1</td><td>foo\n</td></tr>\
<tr><td>2</td><td>bar\n</td></tr>\
<tr><td>3</td><td>bar\n</td></tr>\
<tr><td>4</td><td>baz\n</td></tr>\
</table>\
</code></pre>"
        );
    }
/*
    #[test]
    fn non_one_start() {
        let text = "\
foo
bar
bar
baz
";
        assert_eq!(
            highlight(&Config::default(), "linenos, linenostart=3", text),
            output_inline_plaintext("3\n4\n5\n6\n", "foo\nbar\nbar\nbaz\n")
        );
    }

    #[test]
    fn highlighted_line() {
        let text = "\
foo
bar
bar
baz
";
        assert_eq!(
            highlight(&Config::default(), "linenos, linenostart=3, hl_lines=5", text),
            "<pre style=\"background-color:#2b303b;color:#c0c5ce;\">\
                <code>\
                    <table>\
                        <tr>\
                            <td>\
                                3\n4\n\
                                <mark style=\"background-color:#65737e30;\">5\n</mark>\
                                6\n\
                            </td>\
                            <td>\
                                foo\nbar\n\
                                <mark style=\"background-color:#65737e30;\">\
                                    bar\n\
                                </mark>\
                                baz\n\
                            </td>\
                        </tr>\
                    </table>\
                </code>\
            </pre>"
        );
    }
    */
}