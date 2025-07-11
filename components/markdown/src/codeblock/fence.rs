use std::ops::RangeInclusive;

fn parse_range(s: &str) -> Option<RangeInclusive<usize>> {
    match s.find('-') {
        Some(dash) => {
            let mut from = s[..dash].parse().ok()?;
            let mut to = s[dash + 1..].parse().ok()?;
            if to < from {
                std::mem::swap(&mut from, &mut to);
            }
            Some(from..=to)
        }
        None => {
            let val = s.parse().ok()?;
            Some(val..=val)
        }
    }
}

#[derive(Debug)]
pub struct FenceSettings<'a> {
    pub language: Option<&'a str>,
    pub line_numbers: bool,
    pub line_number_start: usize,
    pub highlight_lines: Vec<RangeInclusive<usize>>,
    pub hide_lines: Vec<RangeInclusive<usize>>,
    pub name: Option<&'a str>,
    pub include: Option<&'a str>,
    pub enable_copy: bool,
}

impl<'a> FenceSettings<'a> {
    pub fn new(fence_info: &'a str) -> Self {
        let mut me = Self {
            language: None,
            line_numbers: false,
            line_number_start: 1,
            highlight_lines: Vec::new(),
            hide_lines: Vec::new(),
            name: None,
            include: None,
            enable_copy: false,
        };

        for token in FenceIter::new(fence_info) {
            match token {
                FenceToken::Language(lang) => me.language = Some(lang),
                FenceToken::EnableLineNumbers => me.line_numbers = true,
                FenceToken::InitialLineNumber(l) => me.line_number_start = l,
                FenceToken::HighlightLines(lines) => me.highlight_lines.extend(lines),
                FenceToken::HideLines(lines) => me.hide_lines.extend(lines),
                FenceToken::Name(n) => me.name = Some(n),
                FenceToken::Include(file) => me.include = Some(file),
                FenceToken::EnableCopy => me.enable_copy = true,
            }
        }

        me
    }
}

#[derive(Debug)]
enum FenceToken<'a> {
    Language(&'a str),
    EnableLineNumbers,
    InitialLineNumber(usize),
    HighlightLines(Vec<RangeInclusive<usize>>),
    HideLines(Vec<RangeInclusive<usize>>),
    Name(&'a str),
    Include(&'a str),
    EnableCopy,
}

struct FenceIter<'a> {
    split: std::str::Split<'a, char>,
}

impl<'a> FenceIter<'a> {
    fn new(fence_info: &'a str) -> Self {
        Self { split: fence_info.split(',') }
    }

    fn parse_ranges(token: Option<&str>) -> Vec<RangeInclusive<usize>> {
        let mut ranges = Vec::new();
        for range in token.unwrap_or("").split(' ') {
            if let Some(range) = parse_range(range) {
                ranges.push(range);
            }
        }
        ranges
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
                "linenostart" => {
                    if let Some(l) = tok_split.next().and_then(|s| s.parse().ok()) {
                        return Some(FenceToken::InitialLineNumber(l));
                    }
                }
                "linenos" => return Some(FenceToken::EnableLineNumbers),
                "hl_lines" => {
                    let ranges = Self::parse_ranges(tok_split.next());
                    return Some(FenceToken::HighlightLines(ranges));
                }
                "hide_lines" => {
                    let ranges = Self::parse_ranges(tok_split.next());
                    return Some(FenceToken::HideLines(ranges));
                }
                "name" => {
                    if let Some(n) = tok_split.next() {
                        return Some(FenceToken::Name(n));
                    }
                }
                "include" => {
                    if let Some(file) = tok_split.next() {
                        return Some(FenceToken::Include(file));
                    }
                }
                "copy" => return Some(FenceToken::EnableCopy),
                lang => {
                    if tok_split.next().is_some() {
                        eprintln!("Warning: Unknown annotation {}", lang);
                        continue;
                    }
                    return Some(FenceToken::Language(lang));
                }
            }
        }
    }
}
