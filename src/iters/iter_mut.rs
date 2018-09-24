
use ::*;
use std::iter::*;

macro_rules! list {
    ($ptr:expr) => (unsafe { &mut *$ptr });
}

#[inline]
pub fn new_iter_mut<'t, T: 't,>(list: &'t mut VecList<T,>, ends: Option<(usize, usize,)>,) -> IterMut<'t, T,> {
    IterMut { list, ends, }
}

#[derive(PartialEq, Eq, PartialOrd, Ord,)]
pub struct IterMut<'t, T: 't,> {
    list: *mut VecList<'t, T,>,
    ends: Option<(usize, usize,)>,
}

impl<'t, T: 't,> Iterator for IterMut<'t, T,> {
    type Item = &'t mut T;

    fn next(&mut self) -> Option<Self::Item,> {
        use imply_option::*;

        self.ends.map(|(node, tail,)| {
            self.ends = list!(self.list).nodes[node].next
                .and_then(|next| (node != tail).then((next, tail,)));
            
            &mut *list!(self.list).nodes[node].value
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

impl<'t, T: 't,> DoubleEndedIterator for IterMut<'t, T,> {
    fn next_back(&mut self) -> Option<Self::Item,> {
        use imply_option::*;

        self.ends.map(|(head, node,)| {
            self.ends = list!(self.list).nodes[node].prev
                .and_then(|prev| (node != head).then((head, prev,)));
            
            &mut *list!(self.list).nodes[node].value
        })
    }
}

unsafe impl<'t, T: 't,> TrustedLen for IterMut<'t, T,> {}

impl<'t, T: 't,> ExactSizeIterator for IterMut<'t, T,> {}

impl<'t, T: 't,> FusedIterator for IterMut<'t, T,> {}
