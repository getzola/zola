use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};

use errors::{bail, Error, Result};
use regex::Regex;
use serde_yaml;
use std::path::Path;
use toml;

mod page;
mod section;

pub use page::PageFrontMatter;
pub use section::SectionFrontMatter;

lazy_static! {
    static ref TOML_RE: Regex =
        Regex::new(r"^[[:space:]]*\+\+\+(\r?\n(?s).*?(?-s))\+\+\+\r?\n?((?s).*(?-s))$").unwrap();
    static ref YAML_RE: Regex =
        Regex::new(r"^[[:space:]]*---(\r?\n(?s).*?(?-s))---\r?\n?((?s).*(?-s))$").unwrap();
}

pub enum RawFrontMatter<'a> {
    Toml(&'a str),
    Yaml(&'a str),
}

impl RawFrontMatter<'_> {
    fn deserialize<T>(&self) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let f: T = match self {
            RawFrontMatter::Toml(s) => toml::from_str(s)?,
            RawFrontMatter::Yaml(s) => match serde_yaml::from_str(s) {
                Ok(d) => d,
                Err(e) => bail!(format!("YAML deserialize error: {:?}", e)),
            },
        };
        Ok(f)
    }
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
fn split_content<'c>(file_path: &Path, content: &'c str) -> Result<(RawFrontMatter<'c>, &'c str)> {
    let (re, is_toml) = if TOML_RE.is_match(content) {
        (&TOML_RE as &Regex, true)
    } else if YAML_RE.is_match(content) {
        (&YAML_RE as &Regex, false)
    } else {
        bail!(
            "Couldn't find front matter in `{}`. Did you forget to add `+++` or `---`?",
            file_path.to_string_lossy()
        );
    };

    // 2. extract the front matter and the content
    let caps = re.captures(content).unwrap();
    // caps[0] is the full match
    // caps[1] => front matter
    // caps[2] => content
    let front_matter = caps.get(1).unwrap().as_str();
    let content = caps.get(2).unwrap().as_str();
    if is_toml {
        Ok((RawFrontMatter::Toml(front_matter), content))
    } else {
        Ok((RawFrontMatter::Yaml(front_matter), content))
    }
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
    use test_case::test_case;

    use super::{split_page_content, split_section_content};

    #[test_case(r#"
+++
title = "Title"
description = "hey there"
date = 2002-10-12
+++
Hello
"#; "toml")]
    #[test_case(r#"
---
title: Title
description: hey there
date: 2002-10-12
---
Hello
"#; "yaml")]
    fn can_split_page_content_valid(content: &str) {
        let (front_matter, content) = split_page_content(Path::new(""), content).unwrap();
        assert_eq!(content, "Hello\n");
        assert_eq!(front_matter.title.unwrap(), "Title");
    }

    #[test_case(r#"
+++
paginate_by = 10
+++
Hello
"#; "toml")]
    #[test_case(r#"
---
paginate_by: 10
---
Hello
"#; "yaml")]
    fn can_split_section_content_valid(content: &str) {
        let (front_matter, content) = split_section_content(Path::new(""), content).unwrap();
        assert_eq!(content, "Hello\n");
        assert!(front_matter.is_paginated());
    }

    #[test_case(r#"
+++
title = "Title"
description = "hey there"
date = 2002-10-12
+++"#; "toml")]
    #[test_case(r#"
---
title: Title
description: hey there
date: 2002-10-12
---"#; "yaml")]
    fn can_split_content_with_only_frontmatter_valid(content: &str) {
        let (front_matter, content) = split_page_content(Path::new(""), content).unwrap();
        assert_eq!(content, "");
        assert_eq!(front_matter.title.unwrap(), "Title");
    }

    #[test_case(r#"
+++
title = "Title"
description = "hey there"
date = 2002-10-02T15:00:00Z
+++
+++"#, "+++"; "toml with pluses in content")]
    #[test_case(r#"
+++
title = "Title"
description = "hey there"
date = 2002-10-02T15:00:00Z
+++
---"#, "---"; "toml with minuses in content")]
    #[test_case(r#"
---
title: Title
description: hey there
date: 2002-10-02T15:00:00Z
---
+++"#, "+++"; "yaml with pluses in content")]
    #[test_case(r#"
---
title: Title
description: hey there
date: 2002-10-02T15:00:00Z
---
---"#, "---"; "yaml with minuses in content")]
    fn can_split_content_lazily(content: &str, expected: &str) {
        let (front_matter, content) = split_page_content(Path::new(""), content).unwrap();
        assert_eq!(content, expected);
        assert_eq!(front_matter.title.unwrap(), "Title");
    }

    #[test_case(r#"
+++
title = "Title"
description = "hey there"
date = 2002-10-12"#; "toml")]
    #[test_case(r#"
+++
title = "Title"
description = "hey there"
date = 2002-10-12
---"#; "toml unmatched")]
    #[test_case(r#"
---
title: Title
description: hey there
date: 2002-10-12"#; "yaml")]
    #[test_case(r#"
---
title: Title
description: hey there
date: 2002-10-12
+++"#; "yaml unmatched")]
    fn errors_if_cannot_locate_frontmatter(content: &str) {
        let res = split_page_content(Path::new(""), content);
        assert!(res.is_err());
    }
}
