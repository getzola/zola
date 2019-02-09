pub trait InsertMany {
    type Element;
    fn insert_many(&mut self, elem_to_insert: Vec<(usize, Self::Element)>);
}

impl<T> InsertMany for Vec<T> {
    type Element = T;

    /// Efficiently insert multiple element in their specified index.
    /// The elements should sorted in ascending order by their index.
    ///
    /// This is done in O(n) time.
    fn insert_many(&mut self, elem_to_insert: Vec<(usize, T)>) {
        let mut inserted = vec![];
        let mut last_idx = 0;

        for (idx, elem) in elem_to_insert.into_iter() {
            let head_len = idx - last_idx;
            inserted.extend(self.splice(0..head_len, std::iter::empty()));
            inserted.push(elem);
            last_idx = idx;
        }
        let len = self.len();
        inserted.extend(self.drain(0..len));

        *self = inserted;
    }
}

#[cfg(test)]
mod test {
    use super::InsertMany;

    #[test]
    fn insert_many_works() {
        let mut v = vec![1, 2, 3, 4, 5];
        v.insert_many(vec![(0, 0), (2, -1), (5, 6)]);
        assert_eq!(v, &[0, 1, 2, -1, 3, 4, 5, 6]);

        let mut v2 = vec![1, 2, 3, 4, 5];
        v2.insert_many(vec![(0, 0), (2, -1)]);
        assert_eq!(v2, &[0, 1, 2, -1, 3, 4, 5]);
    }
}
