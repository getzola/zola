use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};

use errors::{bail, Error, Result};
use regex::Regex;
use std::path::Path;
use std::process::Command;
use chrono::DateTime;
use chrono::offset::Utc;

mod page;
mod section;

pub use page::PageFrontMatter;
pub use section::SectionFrontMatter;

lazy_static! {
    static ref PAGE_RE: Regex =
        Regex::new(r"^[[:space:]]*(\+\+\+|<!--)(\r?\n(?s).*?(?-s))(\+\+\+|-->)\r?\n?((?s).*(?-s))$").unwrap();
    
    static ref TITLE_RE: Regex =
        Regex::new(r"^[[:space:]]*#[ ]*(.*)[ ]*\r?\n?((?s).*(?-s))$").unwrap();
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
fn split_content<'c>(file_path: &Path, content: &'c str) -> Result<(&'c str, &'c str)> {
    if !PAGE_RE.is_match(content) {
        return Ok(("", content));
    }

    // 2. extract the front matter and the content
    let caps = PAGE_RE.captures(content).unwrap();
    // caps[0] is the full match
    // caps[2] => front matter
    // caps[4] => content
    Ok((caps.get(2).unwrap().as_str(), caps.get(4).unwrap().as_str()))
}

/// Split a file between the front matter and its content.
/// Returns a parsed `SectionFrontMatter` and the rest of the content
pub fn split_section_content<'c>(
    file_path: &Path,
    content: &'c str,
) -> Result<(SectionFrontMatter, &'c str)> {
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
pub fn split_page_content<'c>(
    file_path: &Path,
    content: &'c str,
) -> Result<(PageFrontMatter, &'c str)> {
    let (front_matter, mut content) = split_content(file_path, content)?;
    let mut meta = PageFrontMatter::parse(&front_matter).map_err(|e| {
        Error::chain(
            format!("Error when parsing front matter of page `{}`", file_path.to_string_lossy()),
            e,
        )
    })?;

    // Try to get a title by other means
    if meta.title.is_none() {
        if let Some(mat) = TITLE_RE.captures(content) {
            meta.title = Some(mat[1].to_string());
            // Trim title from contents
            content = mat.get(2).unwrap().as_str();
        }
    }
    if meta.title.is_none() {
        if let Some(file_stem) = file_path.file_stem() {
            meta.title = Some(file_stem.to_string_lossy().to_string());
        }
    }

    // Try to get the dates by other means
    if meta.date.is_none() {
        // See when the file was first added to git
        let dir = file_path.parent().unwrap_or(file_path);
        let file = file_path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let output = Command::new("git")
            .args(&["log", "--diff-filter=A", "--follow", "--format=%aI", "--", &file])
            .current_dir(&dir)
            .output()?;
        if output.status.success() {
            let output = String::from_utf8_lossy(&output.stdout);
            if let Some(last_line) = output.lines().last() {
                if !last_line.is_empty() {
                    meta.date = Some(last_line.to_string());
                    meta.date_to_datetime();
                }
            }
        }
    }
    if meta.date.is_none() {
        // Take file creation date
        if let Ok(file_info) = file_path.metadata() {
            if let Ok(created) = file_info.created() {
                let created: DateTime<Utc> = created.into();
                meta.date = Some(created.to_rfc3339());
                meta.date_to_datetime();
            }
        }
    }
    if meta.updated.is_none() {
        // See when the file was last modified in a commit
        let dir = file_path.parent().unwrap_or(file_path);
        let file = file_path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let output = Command::new("git")
            .args(&["log", "--diff-filter=M", "--format=%aI", "--", &file])
            .current_dir(&dir)
            .output()?;
        if output.status.success() {
            let output = String::from_utf8_lossy(&output.stdout);
            if let Some(first_line) = output.lines().next() {
                if !first_line.is_empty() {
                    meta.updated = Some(first_line.to_string());
                }
            }
        }
    }
    if meta.updated.is_none() {
        // Take file creation date
        if let Ok(file_info) = file_path.metadata() {
            if let Ok(modified) = file_info.modified() {
                let modified: DateTime<Utc> = modified.into();
                meta.updated = Some(modified.to_rfc3339());
            }
        }
    }

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
