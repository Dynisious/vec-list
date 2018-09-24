
use ::*;
use std::{
    iter::*,
    ops::{Drop, RangeBounds, Bound,},
};

macro_rules! list {
    ($ptr:expr) => (unsafe { &mut *$ptr });
}

pub fn new_drain<'t, T: 't, R,>(list: &'t mut VecList<T,>, range: R,) -> Drain<'t, T,>
    where R: RangeBounds<usize,> {
    use imply_option::*;

    let start = match range.start_bound() {
        Bound::Included(&start) => start,
        Bound::Excluded(&start) => start + 1,
        Bound::Unbounded => 0,
    };
    let ends = match range.end_bound() {
        Bound::Included(&end) => Some((start, end,)),
        Bound::Excluded(&end) => Some((start, end - 1,)),
        Bound::Unbounded => (list.len() > 0).then((start, list.len().saturating_sub(1),)),
    };
    let ends = match ends {
        None => None,
        Some((start, end,)) => {
            assert!(start <= end, "The range must be acending",);
            assert!(end < list.len(), "The end of the range must be less than the length",);

            Some((list.get_index(start,), list.get_index(end,),))
        },
    };
    
    Drain { list, ends, }
}

#[derive(PartialEq, Eq, PartialOrd, Ord,)]
pub struct Drain<'t, T: 't,> {
    list: *mut VecList<'t, T,>,
    ends: Option<(usize, usize,)>,
}

impl<'t, T: 't,> Drop for Drain<'t, T,> {
    #[inline]
    fn drop(&mut self) { self.for_each(|_| ()) }
}

impl<'t, T: 't,> Iterator for Drain<'t, T,> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item,> {
        let list = list!(self.list);

        self.ends.map(|(node, tail,)| {
            if let Some((l_head, l_tail,)) = list.ends {
                match (node == l_head, node == l_tail,) {
                    (true, true,) => list.ends = None,
                    (true, false,) => list.ends = list.nodes[node].next.map(|next| (next, l_tail,)),
                    (false, true,) => list.ends = list.nodes[node].prev.map(|prev| (l_head, prev,)),
                    _ => (),
                }
            }
            self.ends = if node == tail { None }
                else { list.nodes[node].next.map(|next| (next, tail,)) };
            
            list.shelf_node(node,)
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

impl<'t, T: 't,> DoubleEndedIterator for Drain<'t, T,> {
    fn next_back(&mut self) -> Option<Self::Item,> {
        let list = list!(self.list);

        self.ends.map(|(head, node,)| {
            if let Some((l_head, l_tail,)) = list.ends {
                match (node == l_head, node == l_tail,) {
                    (true, true,) => list.ends = None,
                    (true, false,) => list.ends = list.nodes[node].next.map(|next| (next, l_tail,)),
                    (false, true,) => list.ends = list.nodes[node].prev.map(|prev| (l_head, prev,)),
                    _ => (),
                }
            }
            self.ends = if node == head { None }
                else { list.nodes[node].prev.map(|prev| (head, prev,)) };
            
            list.shelf_node(node,)
        })
    }
}

unsafe impl<'t, T: 't,> TrustedLen for Drain<'t, T,> {}

impl<'t, T: 't,> ExactSizeIterator for Drain<'t, T,> {}

impl<'t, T: 't,> FusedIterator for Drain<'t, T,> {}
