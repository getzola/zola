#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate regex;
extern crate serde;
extern crate tera;
extern crate toml;

#[macro_use]
extern crate errors;
extern crate utils;

use errors::{Error, Result};
use regex::Regex;
use std::path::Path;

mod page;
mod section;

pub use page::PageFrontMatter;
pub use section::SectionFrontMatter;

lazy_static! {
    static ref PAGE_RE: Regex =
        Regex::new(r"^[[:space:]]*\+\+\+\r?\n((?s).*?(?-s))\+\+\+\r?\n?((?s).*(?-s))$").unwrap();
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortBy {
    /// Most recent to oldest
    Date,
    /// Lower weight comes first
    Weight,
    /// No sorting
    None,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InsertAnchor {
    Left,
    Right,
    None,
}

/// Split a file between the front matter and its content
/// Will return an error if the front matter wasn't found
fn split_content(file_path: &Path, content: &str) -> Result<(String, String)> {
    if !PAGE_RE.is_match(content) {
        bail!(
            "Couldn't find front matter in `{}`. Did you forget to add `+++`?",
            file_path.to_string_lossy()
        );
    }

    // 2. extract the front matter and the content
    let caps = PAGE_RE.captures(content).unwrap();
    // caps[0] is the full match
    // caps[1] => front matter
    // caps[2] => content
    Ok((caps[1].to_string(), caps[2].to_string()))
}

/// Split a file between the front matter and its content.
/// Returns a parsed `SectionFrontMatter` and the rest of the content
pub fn split_section_content(
    file_path: &Path,
    content: &str,
) -> Result<(SectionFrontMatter, String)> {
    let (front_matter, content) = split_content(file_path, content)?;
    let meta = SectionFrontMatter::parse(&front_matter).map_err(|e| {
        Error::chain(
            format!("Error when parsing front matter of section `{}`", file_path.to_string_lossy()),
            e,
        )
    })?;
    Ok((meta, content))
}

/// Split a file between the front matter and its content
/// Returns a parsed `PageFrontMatter` and the rest of the content
pub fn split_page_content(file_path: &Path, content: &str) -> Result<(PageFrontMatter, String)> {
    let (front_matter, content) = split_content(file_path, content)?;
    let meta = PageFrontMatter::parse(&front_matter).map_err(|e| {
        Error::chain(
            format!("Error when parsing front matter of page `{}`", file_path.to_string_lossy()),
            e,
        )
    })?;
    Ok((meta, content))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{split_page_content, split_section_content};

    #[test]
    fn can_split_page_content_valid() {
        let content = r#"
+++
title = "Title"
description = "hey there"
date = 2002-10-12
+++
Hello
"#;
        let (front_matter, content) = split_page_content(Path::new(""), content).unwrap();
        assert_eq!(content, "Hello\n");
        assert_eq!(front_matter.title.unwrap(), "Title");
    }

    #[test]
    fn can_split_section_content_valid() {
        let content = r#"
+++
paginate_by = 10
+++
Hello
"#;
        let (front_matter, content) = split_section_content(Path::new(""), content).unwrap();
        assert_eq!(content, "Hello\n");
        assert!(front_matter.is_paginated());
    }

    #[test]
    fn can_split_content_with_only_frontmatter_valid() {
        let content = r#"
+++
title = "Title"
description = "hey there"
date = 2002-10-12
+++"#;
        let (front_matter, content) = split_page_content(Path::new(""), content).unwrap();
        assert_eq!(content, "");
        assert_eq!(front_matter.title.unwrap(), "Title");
    }

    #[test]
    fn can_split_content_lazily() {
        let content = r#"
+++
title = "Title"
description = "hey there"
date = 2002-10-02T15:00:00Z
+++
+++"#;
        let (front_matter, content) = split_page_content(Path::new(""), content).unwrap();
        assert_eq!(content, "+++");
        assert_eq!(front_matter.title.unwrap(), "Title");
    }

    #[test]
    fn errors_if_cannot_locate_frontmatter() {
        let content = r#"
+++
title = "Title"
description = "hey there"
date = 2002-10-12"#;
        let res = split_page_content(Path::new(""), content);
        assert!(res.is_err());
    }
}
