use std::path::Path;

use errors::{Context, Result, bail};
use regex::Regex;
use std::sync::LazyLock;

use crate::front_matter::page::PageFrontMatter;
use crate::front_matter::section::SectionFrontMatter;

static TOML_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^[[:space:]]*\+\+\+[[:space:]]*(\r?\n(?s).*?(?-s))\+\+\+[[:space:]]*(?:$|(?:\r?\n((?s).*(?-s))$))",
    )
    .unwrap()
});

static YAML_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[[:space:]]*---[[:space:]]*(\r?\n(?s).*?(?-s))---[[:space:]]*(?:$|(?:\r?\n((?s).*(?-s))$))")
        .unwrap()
});

pub enum RawFrontMatter<'a> {
    Toml(&'a str),
    Yaml(&'a str),
}

impl RawFrontMatter<'_> {
    pub(crate) fn deserialize<T>(&self) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let f: T = match self {
            RawFrontMatter::Toml(s) => toml::from_str(s)?,
            RawFrontMatter::Yaml(s) => match serde_yaml::from_str(s) {
                Ok(d) => d,
                Err(e) => bail!("YAML deserialize error: {:?}", e),
            },
        };
        Ok(f)
    }

    pub fn to_owned_raw(&self) -> OwnedRawFrontMatter {
        match self {
            RawFrontMatter::Toml(s) => OwnedRawFrontMatter::Toml(s.to_string()),
            RawFrontMatter::Yaml(s) => OwnedRawFrontMatter::Yaml(s.to_string()),
        }
    }
}

/// An owned copy of a page's raw front matter, kept on multilingual sites.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnedRawFrontMatter {
    Toml(String),
    Yaml(String),
}

impl OwnedRawFrontMatter {
    pub fn as_raw(&self) -> RawFrontMatter<'_> {
        match self {
            OwnedRawFrontMatter::Toml(s) => RawFrontMatter::Toml(s.as_str()),
            OwnedRawFrontMatter::Yaml(s) => RawFrontMatter::Yaml(s.as_str()),
        }
    }
}

/// Split a file between the front matter and its content
/// Will return an error if the front matter wasn't found
fn split_content<'c>(file_path: &Path, content: &'c str) -> Result<(RawFrontMatter<'c>, &'c str)> {
    let (caps, is_toml) = if let Some(caps) = TOML_RE.captures(content) {
        (caps, true)
    } else if let Some(caps) = YAML_RE.captures(content) {
        (caps, false)
    } else {
        bail!(
            "Couldn't find front matter in `{}`. Did you forget to add `+++` or `---`?",
            file_path.to_string_lossy()
        );
    };

    // 2. extract the front matter and the content
    // caps[0] is the full match
    // caps[1] => front matter
    // caps[2] => content
    let front_matter = caps.get(1).unwrap().as_str();
    let content = caps.get(2).map_or("", |m| m.as_str());

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
    let meta = SectionFrontMatter::parse(&front_matter).with_context(|| {
        format!("Error when parsing front matter of section `{}`", file_path.to_string_lossy())
    })?;

    Ok((meta, content))
}

/// Split a file between the front matter and its content
/// Returns the raw front matter, a parsed `PageFrontMatter`, and the rest of the content
pub fn split_page_content_with_raw<'c>(
    file_path: &Path,
    content: &'c str,
) -> Result<(RawFrontMatter<'c>, PageFrontMatter, &'c str)> {
    let (front_matter, content) = split_content(file_path, content)?;
    let meta = PageFrontMatter::parse(&front_matter).with_context(|| {
        format!("Error when parsing front matter of page `{}`", file_path.to_string_lossy())
    })?;
    Ok((front_matter, meta, content))
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use test_case::test_case;

    use super::{RawFrontMatter, split_page_content_with_raw, split_section_content};

    #[test]
    fn split_page_content_with_raw_keeps_raw() {
        let content = "+++\ntitle = \"Title\"\n+++\nHello\n";
        let (raw, meta, body) = split_page_content_with_raw(Path::new(""), content).unwrap();
        assert_eq!(body, "Hello\n");
        assert_eq!(meta.title.unwrap(), "Title");
        match raw {
            RawFrontMatter::Toml(s) => assert!(s.contains("title = \"Title\"")),
            RawFrontMatter::Yaml(_) => panic!("expected TOML"),
        }
    }

    #[test]
    fn owned_raw_roundtrip() {
        let raw = RawFrontMatter::Toml("title = \"x\"");
        let owned = raw.to_owned_raw();
        let owned2 = owned.as_raw().to_owned_raw();
        assert_eq!(owned, owned2);
    }

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
    #[test_case(r#"
+++  
title = "Title"
description = "hey there"
date = 2002-10-12
+++
Hello
"#; "toml with trailing whitespace")]
    #[test_case(r#"
---  
title: Title
description: hey there
date: 2002-10-12
---
Hello
"#; "yaml with trailing whitespace")]
    fn can_split_page_content_valid(content: &str) {
        let (_, front_matter, content) =
            split_page_content_with_raw(Path::new(""), content).unwrap();
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
+++
"#; "toml")]
    #[test_case(r#"
---
title: Title
description: hey there
date: 2002-10-12
---
"#; "yaml")]
    #[test_case(r#"
+++
title = "Title"
description = "hey there"
date = 2002-10-12
+++"#; "toml no newline")]
    #[test_case(r#"
---
title: Title
description: hey there
date: 2002-10-12
---"#; "yaml no newline")]
    fn can_split_content_with_only_frontmatter_valid(content: &str) {
        let (_, front_matter, content) =
            split_page_content_with_raw(Path::new(""), content).unwrap();
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
        let (_, front_matter, content) =
            split_page_content_with_raw(Path::new(""), content).unwrap();
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
+++
title = "Title"
description = "hey there"
date = 2002-10-12
++++"#; "toml too many pluses")]
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
    #[test_case(r#"
---
title: Title
description: hey there
date: 2002-10-12
----"#; "yaml too many dashes")]
    fn errors_if_cannot_locate_frontmatter(content: &str) {
        let res = split_page_content_with_raw(Path::new(""), content);
        assert!(res.is_err());
    }
}
