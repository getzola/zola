#[derive(Copy, Clone, Debug)]
pub struct Range {
    pub from: usize,
    pub to: usize,
}

impl Range {
    fn parse(s: &str) -> Option<Range> {
        match s.find('-') {
            Some(dash) => {
                let mut from = s[..dash].parse().ok()?;
                let mut to = s[dash + 1..].parse().ok()?;
                if to < from {
                    std::mem::swap(&mut from, &mut to);
                }
                Some(Range { from, to })
            }
            None => {
                let val = s.parse().ok()?;
                Some(Range { from: val, to: val })
            }
        }
    }
}

#[derive(Debug)]
pub struct FenceSettings<'a> {
    pub language: Option<&'a str>,
    pub line_numbers: bool,
    pub highlight_lines: Vec<Range>,
}
impl<'a> FenceSettings<'a> {
    pub fn new(fence_info: &'a str) -> Self {
        let mut me = Self { language: None, line_numbers: false, highlight_lines: Vec::new() };

        for token in FenceIter::new(fence_info) {
            match token {
                FenceToken::Language(lang) => me.language = Some(lang),
                FenceToken::EnableLineNumbers => me.line_numbers = true,
                FenceToken::HighlightLines(lines) => me.highlight_lines.extend(lines),
            }
        }

        me
    }
}

#[derive(Debug)]
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
        Self { split: fence_info.split(',') }
    }
}

impl<'a> Iterator for FenceIter<'a> {
    type Item = FenceToken<'a>;

    fn next(&mut self) -> Option<FenceToken<'a>> {
        loop {
            let tok = self.split.next()?.trim();

            let mut tok_split = tok.split('=');
            match tok_split.next().unwrap_or("").trim() {
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
                }
                lang => {
                    return Some(FenceToken::Language(lang));
                }
            }
        }
    }
}
