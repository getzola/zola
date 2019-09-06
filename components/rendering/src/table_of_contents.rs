/// Populated while receiving events from the markdown parser
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Header {
    #[serde(skip_serializing)]
    pub level: u32,
    pub id: String,
    pub permalink: String,
    pub title: String,
    pub children: Vec<Header>,
}

impl Header {
    pub fn new(level: u32) -> Header {
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

// Takes a potential (mutable) parent and a header to try and insert into
// Returns true when it performed the insertion, false otherwise
fn insert_into_parent(potential_parent: Option<&mut Header>, header: &Header) -> bool {
    match potential_parent {
        None => {
            // No potential parent to insert into so it needs to be insert higher
            false
        }
        Some(parent) => {
            if header.level <= parent.level {
                // Heading is same level or higher so we don't insert here
                return false;
            }
            if header.level + 1 == parent.level {
                // We have a direct child of the parent
                parent.children.push(header.clone());
                return true;
            }
            // We need to go deeper
            if !insert_into_parent(parent.children.iter_mut().last(), header) {
                // No, we need to insert it here
                parent.children.push(header.clone());
            }
            true
        }
    }
}

/// Converts the flat temp headers into a nested set of headers
/// representing the hierarchy
pub fn make_table_of_contents(headers: Vec<Header>) -> Vec<Header> {
    let mut toc = vec![];
    for header in headers {
        // First header or we try to insert the current header in a previous one
        if toc.is_empty() || !insert_into_parent(toc.iter_mut().last(), &header) {
            toc.push(header);
        }
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
    fn can_make_deep_toc() {
        let input = vec![
            Header::new(1),
            Header::new(2),
            Header::new(3),
            Header::new(4),
            Header::new(5),
            Header::new(4),
        ];
        let toc = make_table_of_contents(input);
        assert_eq!(toc.len(), 1);
        assert_eq!(toc[0].children.len(), 1);
        assert_eq!(toc[0].children[0].children.len(), 1);
        assert_eq!(toc[0].children[0].children[0].children.len(), 2);
        assert_eq!(toc[0].children[0].children[0].children[0].children.len(), 1);
    }

    #[test]
    fn can_make_deep_messy_toc() {
        let input = vec![
            Header::new(2), // toc[0]
            Header::new(3),
            Header::new(4),
            Header::new(5),
            Header::new(4),
            Header::new(2), // toc[1]
            Header::new(1), // toc[2]
            Header::new(2),
            Header::new(3),
            Header::new(4),
        ];
        let toc = make_table_of_contents(input);
        assert_eq!(toc.len(), 3);
        assert_eq!(toc[0].children.len(), 1);
        assert_eq!(toc[0].children[0].children.len(), 2);
        assert_eq!(toc[0].children[0].children[0].children.len(), 1);
        assert_eq!(toc[1].children.len(), 0);
        assert_eq!(toc[2].children.len(), 1);
        assert_eq!(toc[2].children[0].children.len(), 1);
        assert_eq!(toc[2].children[0].children[0].children.len(), 1);
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
