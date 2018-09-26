
use ::*;
use std::iter::*;

macro_rules! list {
    ($ptr:expr) => (unsafe { &mut *$ptr });
}

/// Creates a new `IterMut` from parts.
/// 
/// # Params
/// 
/// list --- The `VecList` to iterate over.  
/// ends --- The ends of the `VecList` to iterate over.
#[inline]
pub fn new_iter_mut<'t, T: 't,>(list: &'t mut VecList<T,>, ends: Option<(usize, usize,)>,) -> IterMut<'t, T,> {
    IterMut { list, ends, }
}

/// A mutable iterator over a `VecList`.
#[derive(PartialEq, Eq, PartialOrd, Ord,)]
pub struct IterMut<'t, T: 't,> {
    /// The `VecList` to iterate over.
    list: *mut VecList<'t, T,>,
    /// The ends to iterate over.
    ends: Option<(usize, usize,)>,
}

impl<'t, T: 't,> Iterator for IterMut<'t, T,> {
    type Item = &'t mut T;

    fn next(&mut self) -> Option<Self::Item,> {
        use imply_option::*;

        self.ends.map(|(node, tail,)| {
            //Advance the ends.
            self.ends = list!(self.list).nodes[node].next
                //If the `Node` is the tail pointer while iterating forward, this is the last value.
                .and_then(|next| (node != tail).then((next, tail,)));
            
            &mut *list!(self.list).nodes[node].value
        })
    }
    fn size_hint(&self) -> (usize, Option<usize,>,) {
        //Calculate the number of `Node`s.
        let len = match self.ends {
            //If there are no ends, there are no values.
            None => 0,
            Some((mut node, tail,)) => {
                let list = list!(self.list);
                //If there are ends there is at least 1 value.
                let mut count = 1;

                //Count the remaining values.
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
            //Advance the ends.
            self.ends = list!(self.list).nodes[node].prev
                //If the `Node` is the head pointer while iterating backward, this is the last value.
                .and_then(|prev| (node != head).then((head, prev,)));
            
            &mut *list!(self.list).nodes[node].value
        })
    }
}

unsafe impl<'t, T: 't,> TrustedLen for IterMut<'t, T,> {}

impl<'t, T: 't,> ExactSizeIterator for IterMut<'t, T,> {}

impl<'t, T: 't,> FusedIterator for IterMut<'t, T,> {}
