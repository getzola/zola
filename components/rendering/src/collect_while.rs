use std::iter::{FromIterator, Iterator, Peekable};

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
struct TakeWhile<'a, I, P>
where
    I: 'a + Iterator,
 {
    iter: &'a mut Peekable<I>,
    flag: bool,
    predicate: P,
}

impl<'a, I, P> Iterator for TakeWhile<'a, I, P>
where
    I: Iterator,
    P: FnMut(&I::Item) -> bool,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        if self.flag {
            None
        } else {
            let do_it = match self.iter.peek() {
                Some(x) => {
                    if (self.predicate)(x) {
                        Some(true)
                    } else {
                        self.flag = true;
                        Some(false)
                    }
                },
                None => None,
            };

            match do_it {
                Some(true) => {
                    self.iter.next()
                },
                Some(false) | None => None,
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper) // can't know a lower bound, due to the predicate
    }
}

fn take_while<I, P>(iter: &mut Peekable<I>, predicate: P) -> TakeWhile<I, P>
where
    I: Iterator,
{
    TakeWhile {
        iter,
        flag: false,
        predicate
    }
}

pub fn collect_while<I, P, C>(iter: &mut Peekable<I>, predicate: P) -> C
where
    I: Iterator,
    P: Fn(&I::Item) -> bool,
    C: FromIterator<I::Item>,
{
    take_while(iter, predicate).collect()
}
