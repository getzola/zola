/// Populated while receiving events from the markdown parser
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Header {
    #[serde(skip_serializing)]
    pub level: i32,
    pub id: String,
    pub permalink: String,
    pub title: String,
    pub children: Vec<Header>,
}

impl Header {
    pub fn new(level: i32) -> Header {
        Header {
            level,
            id: String::new(),
            permalink: String::new(),
            title: String::new(),
            children: Vec::new(),
        }
    }
}

impl Default for Header {
    fn default() -> Self {
        Header::new(0)
    }
}

/// Converts the flat temp headers into a nested set of headers
/// representing the hierarchy
pub fn make_table_of_contents(headers: Vec<Header>) -> Vec<Header> {
    let mut toc = vec![];
    'parent: for header in headers {
        if toc.is_empty() {
            toc.push(header);
            continue;
        }

        // See if we have to insert as a child of a previous header
        for h in toc.iter_mut().rev() {
            // Look in its children first
            for child in h.children.iter_mut().rev() {
                if header.level > child.level {
                    child.children.push(header);
                    continue 'parent;
                }
            }
            if header.level > h.level {
                h.children.push(header);
                continue 'parent;
            }
        }

        // Nop, just insert it
        toc.push(header)
    }

    toc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_make_basic_toc() {
        let input = vec![Header::new(1), Header::new(1), Header::new(1)];
        let toc = make_table_of_contents(input);
        assert_eq!(toc.len(), 3);
    }

    #[test]
    fn can_make_more_complex_toc() {
        let input = vec![
            Header::new(1),
            Header::new(2),
            Header::new(2),
            Header::new(3),
            Header::new(2),
            Header::new(1),
            Header::new(2),
            Header::new(3),
            Header::new(3),
        ];
        let toc = make_table_of_contents(input);
        assert_eq!(toc.len(), 2);
        assert_eq!(toc[0].children.len(), 3);
        assert_eq!(toc[1].children.len(), 1);
        assert_eq!(toc[0].children[1].children.len(), 1);
        assert_eq!(toc[1].children[0].children.len(), 2);
    }

    #[test]
    fn can_make_messy_toc() {
        let input = vec![
            Header::new(3),
            Header::new(2),
            Header::new(2),
            Header::new(3),
            Header::new(2),
            Header::new(1),
            Header::new(4),
        ];
        let toc = make_table_of_contents(input);
        println!("{:#?}", toc);
        assert_eq!(toc.len(), 5);
        assert_eq!(toc[2].children.len(), 1);
        assert_eq!(toc[4].children.len(), 1);
    }
}
