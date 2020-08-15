use super::codeblock::Range;

impl Range {
    fn parse(s: &str) -> Option<Range> {
        match s.find('-') {
            Some(dash) => {
                let from = s[..dash].parse().ok()?;
                let to = s[dash+1..].parse().ok()?;
                Some(Range {
                    from,
                    to,
                })
            },
            None => {
                let val = s.parse().ok()?;
                Some(Range {
                    from: val,
                    to: val,
                })
            },
        }
    }
}

pub struct FenceSettings<'a> {
    pub language: Option<&'a str>,
    pub line_numbers: bool,
    pub highlight_lines: Vec<Range>,
}
impl<'a> FenceSettings<'a> {
    pub fn new(fence_info: &'a str) -> Self {
        let mut me = Self {
            language: None,
            line_numbers: false,
            highlight_lines: Vec::new(),
        };

        let mut fence_iter = FenceIter::new(fence_info);
        while let Some(token) = fence_iter.next() {
            match token {
                FenceToken::Language(lang) => me.language = Some(lang),
                FenceToken::EnableLineNumbers => me.line_numbers = true,
                FenceToken::HighlightLines(lines) => me.highlight_lines.extend(lines),
            }
        }

        me
    }
}

enum FenceToken<'a> {
    Language(&'a str),
    EnableLineNumbers,
    HighlightLines(Vec<Range>),
}

struct FenceIter<'a> {
    split: std::str::Split<'a, char>,
}
impl<'a> FenceIter<'a> {
    fn new(fence_info: &'a str) -> Self {
        Self {
            split: fence_info.split(','),
        }
    }
    fn next(&mut self) -> Option<FenceToken<'a>> {
        loop {
            let tok = self.split.next()?.trim();

            let mut tok_split = tok.split('=');
            match tok_split.next().unwrap_or("") {
                "" => continue,
                "linenos" => return Some(FenceToken::EnableLineNumbers),
                "hl_lines" => {
                    let mut ranges = Vec::new();
                    for range in tok_split.next().unwrap_or("").split(' ') {
                        if let Some(range) = Range::parse(range) {
                            ranges.push(range);
                        }
                    }
                    return Some(FenceToken::HighlightLines(ranges));
                },
                lang => {
                    return Some(FenceToken::Language(lang));
                },
            }
        }
    }
}
