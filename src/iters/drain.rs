
use ::*;
use std::{
    iter::*,
    ops::{Drop, RangeBounds, Bound,},
};

macro_rules! list {
    ($ptr:expr) => (unsafe { &mut *$ptr });
}

/// Creates a new `Drain` iterator from parts.
/// 
/// # Params
/// 
/// list --- The `VecList` to iterator over.  
/// range --- The range of indexes to iterate over.
pub fn new_drain<'t, T: 't, R,>(list: &'t mut VecList<T,>, range: R,) -> Drain<'t, T,>
    where R: RangeBounds<usize,> {
    use imply_option::*;
    
    //Get the start bound.
    let start = match range.start_bound() {
        //If the bound is included, use it.
        Bound::Included(&start) => start,
        //If the bound is excluded, advance it.
        Bound::Excluded(&start) => start + 1,
        //If it is unbounded, start from the 0th index.
        Bound::Unbounded => 0,
    };
    //Get the end bound.
    let ends = match range.end_bound() {
        //If the bound is included, use the bound.
        Bound::Included(&end) => Some((start, end,)),
        //If the bound is excluded, advance the bound.
        Bound::Excluded(&end) => Some((start, end - 1,)),
        //If it is unbounded, use the length.
        //  If the list is empty, there are no ends and no values.
        Bound::Unbounded => (list.len() > 0).then((start, list.len().saturating_sub(1),)),
    };
    //Validate the ends.
    let ends = match ends {
        None => None,
        Some((start, end,)) => {
            assert!(start <= end, "The range must be acending",);
            assert!(end < list.len(), "The end of the range must be less than the length",);

            eprintln!("{}, {}, {}", start, end, list.len(),);
            //Convert the index bounds to their actual `Node` indexes.
            Some((list.get_index(start,), list.get_index(end,),))
        },
    };
    
    Drain { list, ends, }
}

/// A `Drain` iterator which lazily removes all of the values in its range.
#[derive(PartialEq, Eq, PartialOrd, Ord,)]
pub struct Drain<'t, T: 't,> {
    //The `VecList` to iterate over.
    list: *mut VecList<'t, T,>,
    //The ends to iterate over.
    ends: Option<(usize, usize,)>,
}

impl<'t, T: 't,> Drop for Drain<'t, T,> {
    //Drop all values in the range, even if the user does not iterate over them.
    //Iterating them like this ensures that the values will be dropped.
    #[inline]
    fn drop(&mut self) { self.for_each(|_| ()) }
}

impl<'t, T: 't,> Iterator for Drain<'t, T,> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item,> {
        let list = list!(self.list);

        self.ends.map(|(node, tail,)| {
            //If the `Node` is an end pointer for the `VecList` the end pointer needs to be updated.
            if let Some((l_head, l_tail,)) = list.ends {
                //Check if the `Node` is an end pointer for the `VecList`.
                match (node == l_head, node == l_tail,) {
                    //If the `Node` is both pointers, there is only a single `Node` being removed.
                    (true, true,) => list.ends = None,
                    //If the `Node` is the head pointer, advance the head pointer.
                    (true, false,) => list.ends = list.nodes[node].next.map(|next| (next, l_tail,)),
                    //If the `Node` is the tail pointer, advance the tail pointer.
                    (false, true,) => list.ends = list.nodes[node].prev.map(|prev| (l_head, prev,)),
                    //If the `Node` is neither pointer, no change.
                    _ => (),
                }
            }

            //If the `Node` is the tail pointer while advancing forward, this is the final value.
            self.ends = if node == tail { None }
                //Advance the head pointer.
                else { list.nodes[node].next.map(|next| (next, tail,)) };
            
            //Remove the `Node`.
            list.remove_node(node,)
        })
    }
    fn size_hint(&self) -> (usize, Option<usize,>,) {
        let len = match self.ends {
            //If there are no end pointers, there are no values.
            None => 0,
            Some((mut node, tail,)) => {
                let list = list!(self.list);
                //If there are end pointers, there is at least 1 value to remove.
                let mut count = 1;

                //Count the remaining `Node`s.
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
            //If the `Node` is an end pointer for the `VecList` the end pointer needs to be updated.
            if let Some((l_head, l_tail,)) = list.ends {
                match (node == l_head, node == l_tail,) {
                    //If the `Node` is both end pointers, then the last value is being removed.
                    (true, true,) => list.ends = None,
                    //If the `Node` is the head pointer, advance the head pointer.
                    (true, false,) => list.ends = list.nodes[node].next.map(|next| (next, l_tail,)),
                    //If the `Node` is the tail pointer, advance the tail pointer.
                    (false, true,) => list.ends = list.nodes[node].prev.map(|prev| (l_head, prev,)),
                    //If the `Node` is neither end pointer, then no change needed.
                    _ => (),
                }
            }
            //If the `Node` is the head pointer while advancing backwards, this is the final value.
            self.ends = if node == head { None }
                //Else advance the tail pointer.
                else { list.nodes[node].prev.map(|prev| (head, prev,)) };
            
            //Remove the `Node`.
            list.remove_node(node,)
        })
    }
}

unsafe impl<'t, T: 't,> TrustedLen for Drain<'t, T,> {}

impl<'t, T: 't,> ExactSizeIterator for Drain<'t, T,> {}

impl<'t, T: 't,> FusedIterator for Drain<'t, T,> {}
