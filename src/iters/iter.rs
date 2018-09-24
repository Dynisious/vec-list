
use ::*;
use std::iter::*;

macro_rules! list {
    ($ptr:expr) => (unsafe { &*$ptr });
}

#[inline]
pub const fn new_iter<'t, T: 't,>(list: &'t VecList<T,>, ends: Option<(usize, usize,)>,) -> Iter<'t, T,> {
    Iter { list, ends, }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone,)]
pub struct Iter<'t, T: 't,> {
    list: *const VecList<'t, T,>,
    ends: Option<(usize, usize,)>,
}

impl<'t, T: 't,> Iterator for Iter<'t, T,> {
    type Item = &'t T;

    fn next(&mut self) -> Option<Self::Item,> {
        use imply_option::*;

        let list = list!(self.list);

        self.ends.map(|(node, tail,)| {
            self.ends = list.nodes[node].next
                .and_then(|next| (node != tail).then((next, tail,)));
            
            &*list.nodes[node].value
        })
    }
    fn size_hint(&self) -> (usize, Option<usize,>,) {
        let len = match self.ends {
            None => 0,
            Some((mut node, tail,)) => {
                let list = list!(self.list);
                let mut count = 1;

                while node != tail {
                    count += 1;
                    node = list.nodes[node].next
                        .expect(&node_err!());
                }

                count
            },
        };
        
        (len, Some(len),)
    }
}

impl<'t, T: 't,> DoubleEndedIterator for Iter<'t, T,> {
    fn next_back(&mut self) -> Option<Self::Item,> {
        use imply_option::*;
        
        let list = list!(self.list);
        
        self.ends.map(|(head, node,)| {
            self.ends = list.nodes[node].prev
                .and_then(|prev| (node != head).then((head, prev,)));
            
            &*list.nodes[node].value
        })
    }
}

unsafe impl<'t, T: 't,> TrustedLen for Iter<'t, T,> {}

impl<'t, T: 't,> ExactSizeIterator for Iter<'t, T,> {}

impl<'t, T: 't,> FusedIterator for Iter<'t, T,> {}
