use serde_derive::Serialize;

/// Populated while receiving events from the markdown parser
#[derive(Debug, Default, PartialEq, Clone, Serialize)]
pub struct Heading {
    pub level: u32,
    pub id: String,
    pub permalink: String,
    pub title: String,
    pub children: Vec<Heading>,
}

impl Heading {
    pub fn new(level: u32) -> Heading {
        Heading { level, ..Self::default() }
    }
}

// Takes a potential (mutable) parent and a heading to try and insert into
// Returns true when it performed the insertion, false otherwise
fn insert_into_parent(potential_parent: Option<&mut Heading>, heading: &Heading) -> bool {
    match potential_parent {
        None => {
            // No potential parent to insert into so it needs to be insert higher
            false
        }
        Some(parent) => {
            if heading.level <= parent.level {
                // Heading is same level or higher so we don't insert here
                return false;
            }
            if heading.level + 1 == parent.level {
                // We have a direct child of the parent
                parent.children.push(heading.clone());
                return true;
            }
            // We need to go deeper
            if !insert_into_parent(parent.children.iter_mut().last(), heading) {
                // No, we need to insert it here
                parent.children.push(heading.clone());
            }
            true
        }
    }
}

/// Converts the flat temp headings into a nested set of headings
/// representing the hierarchy
pub fn make_table_of_contents(headings: Vec<Heading>) -> Vec<Heading> {
    let mut toc = vec![];
    for heading in headings {
        // First heading or we try to insert the current heading in a previous one
        if toc.is_empty() || !insert_into_parent(toc.iter_mut().last(), &heading) {
            toc.push(heading);
        }
    }

    toc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_make_basic_toc() {
        let input = vec![Heading::new(1), Heading::new(1), Heading::new(1)];
        let toc = make_table_of_contents(input);
        assert_eq!(toc.len(), 3);
    }

    #[test]
    fn can_make_more_complex_toc() {
        let input = vec![
            Heading::new(1),
            Heading::new(2),
            Heading::new(2),
            Heading::new(3),
            Heading::new(2),
            Heading::new(1),
            Heading::new(2),
            Heading::new(3),
            Heading::new(3),
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
            Heading::new(1),
            Heading::new(2),
            Heading::new(3),
            Heading::new(4),
            Heading::new(5),
            Heading::new(4),
        ];
        let toc = make_table_of_contents(input);
        assert_eq!(toc.len(), 1);
        assert_eq!(toc[0].level, 1);
        assert_eq!(toc[0].children.len(), 1);
        assert_eq!(toc[0].children[0].children.len(), 1);
        assert_eq!(toc[0].children[0].children[0].children.len(), 2);
        assert_eq!(toc[0].children[0].children[0].children[0].children.len(), 1);
    }

    #[test]
    fn can_make_deep_messy_toc() {
        let input = vec![
            Heading::new(2), // toc[0]
            Heading::new(3),
            Heading::new(4),
            Heading::new(5),
            Heading::new(4),
            Heading::new(2), // toc[1]
            Heading::new(1), // toc[2]
            Heading::new(2),
            Heading::new(3),
            Heading::new(4),
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
            Heading::new(3),
            Heading::new(2),
            Heading::new(2),
            Heading::new(3),
            Heading::new(2),
            Heading::new(1),
            Heading::new(4),
        ];
        let toc = make_table_of_contents(input);
        println!("{:#?}", toc);
        assert_eq!(toc.len(), 5);
        assert_eq!(toc[2].children.len(), 1);
        assert_eq!(toc[4].children.len(), 1);
    }
}
