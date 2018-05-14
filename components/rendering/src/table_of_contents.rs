use tera::{Tera, Context as TeraContext};
use front_matter::InsertAnchor;


#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Header {
    #[serde(skip_serializing)]
    pub level: i32,
    pub id: String,
    pub title: String,
    pub permalink: String,
    pub children: Vec<Header>,
}

impl Header {
    pub fn from_temp_header(tmp: &TempHeader, children: Vec<Header>) -> Header {
        Header {
            level: tmp.level,
            id: tmp.id.clone(),
            title: tmp.title.clone(),
            permalink: tmp.permalink.clone(),
            children,
        }
    }
}

/// Populated while receiving events from the markdown parser
#[derive(Debug, PartialEq, Clone)]
pub struct TempHeader {
    pub level: i32,
    pub id: String,
    pub permalink: String,
    pub title: String,
}

impl TempHeader {
    pub fn new(level: i32) -> TempHeader {
        TempHeader {
            level,
            id: String::new(),
            permalink: String::new(),
            title: String::new(),
        }
    }

    pub fn push(&mut self, val: &str) {
        self.title += val;
    }

    /// Transform all the information we have about this header into the HTML string for it
    pub fn to_string(&self, tera: &Tera, insert_anchor: InsertAnchor) -> String {
        let anchor_link = if insert_anchor != InsertAnchor::None {
            let mut c = TeraContext::new();
            c.add("id", &self.id);
            tera.render("anchor-link.html", &c).unwrap()
        } else {
            String::new()
        };

        match insert_anchor {
            InsertAnchor::None => format!("<h{lvl} id=\"{id}\">{t}</h{lvl}>\n", lvl=self.level, t=self.title, id=self.id),
            InsertAnchor::Left => format!("<h{lvl} id=\"{id}\">{a}{t}</h{lvl}>\n", lvl=self.level, a=anchor_link, t=self.title, id=self.id),
            InsertAnchor::Right => format!("<h{lvl} id=\"{id}\">{t}{a}</h{lvl}>\n", lvl=self.level, a=anchor_link, t=self.title, id=self.id),
        }
    }
}

impl Default for TempHeader {
    fn default() -> Self {
        TempHeader::new(0)
    }
}


/// Recursively finds children of a header
fn find_children(parent_level: i32, start_at: usize, temp_headers: &[TempHeader]) -> (usize, Vec<Header>) {
    let mut headers = vec![];

    let mut start_at = start_at;
    // If we have children, we will need to skip some headers since they are already inserted
    let mut to_skip = 0;

    for h in &temp_headers[start_at..] {
        // stop when we encounter a title at the same level or higher
        // than the parent one. Here a lower integer is considered higher as we are talking about
        // HTML headers: h1, h2, h3, h4, h5 and h6
        if h.level <= parent_level {
            return (start_at, headers);
        }

        // Do we need to skip some headers?
        if to_skip > 0 {
            to_skip -= 1;
            continue;
        }

        let (end, children) = find_children(h.level, start_at + 1, temp_headers);
        headers.push(Header::from_temp_header(h, children));

        // we didn't find any children
        if end == start_at {
            start_at += 1;
            to_skip = 0;
        } else {
            // calculates how many we need to skip. Since the find_children start_at starts at 1,
            // we need to remove 1 to ensure correctness
            to_skip = end - start_at - 1;
            start_at = end;
        }

        // we don't want to index out of bounds
        if start_at + 1 > temp_headers.len() {
            return (start_at, headers);
        }
    }

    (start_at, headers)
}


/// Converts the flat temp headers into a nested set of headers
/// representing the hierarchy
pub fn make_table_of_contents(temp_headers: &[TempHeader]) -> Vec<Header> {
    let mut toc = vec![];
    let mut start_idx = 0;
    for (i, h) in temp_headers.iter().enumerate() {
        if i < start_idx {
            continue;
        }
        let (end_idx, children) = find_children(h.level, start_idx + 1, temp_headers);
        start_idx = end_idx;
        toc.push(Header::from_temp_header(h, children));
    }

    toc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_make_basic_toc() {
        let input = vec![
            TempHeader::new(1),
            TempHeader::new(1),
            TempHeader::new(1),
        ];
        let toc = make_table_of_contents(&input);
        assert_eq!(toc.len(), 3);
    }

    #[test]
    fn can_make_more_complex_toc() {
        let input = vec![
            TempHeader::new(1),
            TempHeader::new(2),
            TempHeader::new(2),
            TempHeader::new(3),
            TempHeader::new(2),
            TempHeader::new(1),
            TempHeader::new(2),
            TempHeader::new(3),
            TempHeader::new(3),
        ];
        let toc = make_table_of_contents(&input);
        assert_eq!(toc.len(), 2);
        assert_eq!(toc[0].children.len(), 3);
        assert_eq!(toc[1].children.len(), 1);
        assert_eq!(toc[0].children[1].children.len(), 1);
        assert_eq!(toc[1].children[0].children.len(), 2);
    }

    #[test]
    fn can_make_messy_toc() {
        let input = vec![
            TempHeader::new(3),
            TempHeader::new(2),
            TempHeader::new(2),
            TempHeader::new(3),
            TempHeader::new(2),
            TempHeader::new(1),
            TempHeader::new(4),
        ];
        let toc = make_table_of_contents(&input);
        assert_eq!(toc.len(), 5);
        assert_eq!(toc[2].children.len(), 1);
        assert_eq!(toc[4].children.len(), 1);
    }
}
