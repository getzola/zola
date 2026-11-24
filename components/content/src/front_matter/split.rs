use std::path::Path;

use crate::FrontMatter;
use crate::file_info::Kind;
use errors::{Context, Result, bail};
use regex::Regex;
use std::sync::LazyLock;

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
            RawFrontMatter::Yaml(s) => serde_yaml::from_str(s)?,
        };
        Ok(f)
    }
}

/// Split a file between the front matter and its content.
/// Returns a parsed `FrontMatter` and the rest of the content.
/// Will return an error if the front matter wasn't found
pub fn split_content<'c>(
    file_path: &Path,
    content: &'c str,
    kind: Kind,
) -> Result<(FrontMatter, &'c str)> {
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

    // extract the front matter and the content
    // caps[0] is the full match
    // caps[1] => front matter
    // caps[2] => content
    let front_matter = caps.get(1).unwrap().as_str();
    let content = caps.get(2).map_or("", |m| m.as_str());

    let raw = if is_toml {
        RawFrontMatter::Toml(front_matter)
    } else {
        RawFrontMatter::Yaml(front_matter)
    };
    let what = match kind {
        Kind::Page => "page",
        Kind::Section => "section",
    };
    let meta = FrontMatter::parse(&raw, kind)
        .with_context(|| format!("Error when parsing front matter of {what}"))?;

    Ok((meta, content))
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use test_case::test_case;

    use super::split_content;
    use crate::file_info::Kind;

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
        let (front_matter, content) = split_content(Path::new(""), content, Kind::Page).unwrap();
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
        let (front_matter, content) = split_content(Path::new(""), content, Kind::Section).unwrap();
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
        let (front_matter, content) = split_content(Path::new(""), content, Kind::Page).unwrap();
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
        let (front_matter, content) = split_content(Path::new(""), content, Kind::Page).unwrap();
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
        let res = split_content(Path::new(""), content, Kind::Page);
        assert!(res.is_err());
    }
}
