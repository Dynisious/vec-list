
use ::*;
use std::{
    iter::*,
};

macro_rules! list {
    ($ptr:expr) => (unsafe { &mut *$ptr });
}

/// Creates a new `DrainFilter` from parts.
/// 
/// # Params
/// 
/// list --- The `VecList` to iterate over.  
/// filter --- The filter function to use.
pub fn new_drain_filter<'t, T: 't, F,>(list: &'t mut VecList<T,>, filter: F,) -> DrainFilter<'t, T, F,>
    where F: FnMut(&mut T,) -> bool, {
    DrainFilter { list, ends: list.ends, filter, }
}

/// An iterator over a `VecList` which applies a filter to each value and returns all
/// values which pass the filter.
#[derive(PartialEq, Eq, PartialOrd, Ord,)]
pub struct DrainFilter<'t, T: 't, F,>
    where F: FnMut(&mut T,) -> bool, {
    /// The `VecList` to iterate over.
    list: *mut VecList<'t, T,>,
    /// The ends of the iterator range.
    ends: Option<(usize, usize,)>,
    /// The filter function to apply.
    filter: F,
}

impl<'t, T: 't, F,> Iterator for DrainFilter<'t, T, F,>
    where F: FnMut(&mut T,) -> bool, {
    type Item = T;
    
    fn next(&mut self) -> Option<Self::Item,> {
        let list = list!(self.list);

        match self.ends {
            //If there are no ends, there's a value.
            None => None,
            //If there are ends, there's a value.
            Some((node, tail,)) => {
                //Apply the filter to the value.
                let passed = (self.filter)(&mut list.nodes[node].value);

                //Check if the value passed the filter.
                if passed {
                    //If the value passed the filter, it being removed could change the
                    //  end pointers of the `VecList`.

                    if let Some((l_head, l_tail,)) = list.ends {
                        //Check if the `Node` being removed is one of the end pointers.
                        match (node == l_head, node == l_tail,) {
                            //If the `Node` is both end pointers then it is the only
                            //  `Node` and is being removed hence the `VecList` is empty.
                            (true, true,) => list.ends = None,
                            //If the `Node` is the head pointer, then advance the head pointer.
                            (true, false,) => list.ends = list.nodes[node].next.map(|next| (next, l_tail,)),
                            //If the `Node` is the tail pointer, then advance the tail pointer.
                            (false, true,) => list.ends = list.nodes[node].prev.map(|prev| (l_head, prev,)),
                            //If it is not either pointer, no change needed.
                            _ => (),
                        }
                    }
                }
                //If the `Node` is the tail pointer while advancing forward then this is
                //  the last `Node`.
                self.ends = if node == tail { None }
                    //Advance the head pointer.
                    else { list.nodes[node].next.map(|next| (next, tail,)) };
                
                //If the value passed the filter, remove the `Node`.
                if passed { Some(list.remove_node(node,)) }
                //IF the value did not pass the filter, continue iterating.
                else { return self.next() }
            },
        }
    }
    fn size_hint(&self) -> (usize, Option<usize,>,) {
        //Calculate the maximum length of the `DrainFilter`.
        let len = match self.ends {
            //If there are no ends, there are no values.
            None => 0,
            //If there are ends, then at most all of the values will be removed.
            Some((mut node, tail,)) => {
                let list = list!(self.list);
                //There must be at least 1 value.
                let mut count = 1;

                //Count all the other `Node`s.
                while node != tail {
                    count += 1;
                    node = list.nodes[node].next
                        .expect(&node_err!());
                }

                count
            },
        };
        
        (0, Some(len),)
    }
}

impl<'t, T: 't, F,> DoubleEndedIterator for DrainFilter<'t, T, F,>
    where F: FnMut(&mut T,) -> bool, {
    fn next_back(&mut self) -> Option<Self::Item,> {
        let list = list!(self.list);

        match self.ends {
            //If there are no ends, there's a value.
            None => None,
            //If there are ends, there's a value.
            Some((head, node,)) => {
                //Check if the value passes the value.
                let passed = (self.filter)(&mut list.nodes[node].value);

                //If the value is going to be removed it may affect the `VecList` end pointers.
                if passed {
                    if let Some((l_head, l_tail,)) = list.ends {
                        //Check if the node being removed is one of the `VecList` end pointers.
                        match (node == l_head, node == l_tail,) {
                            //If it is both end pointers then the only `Node` is being removed.
                            (true, true,) => list.ends = None,
                            //If it is the head pointer, advance the head pointer.
                            (true, false,) => list.ends = list.nodes[node].next.map(|next| (next, l_tail,)),
                            //If it is the tail pointer, advance the tail pointer.
                            (false, true,) => list.ends = list.nodes[node].prev.map(|prev| (l_head, prev,)),
                            //If it is neither pointer, no change needed.
                            _ => (),
                        }
                    }
                }
                //If the `Node` is the head pointer then this is the last `Node`.
                //  Clear the ends.
                self.ends = if node == head { None }
                    //Advance the tail pointer.
                    else { list.nodes[node].prev.map(|prev| (head, prev,)) };
                
                //If the value passed the filter, remove the `Node`.
                if passed { Some(list.remove_node(node,)) }
                //Else get the next value.
                else { return self.next_back() }
            },
        }
    }
}

impl<'t, T: 't, F,> FusedIterator for DrainFilter<'t, T, F,> 
    where F: FnMut(&mut T,) -> bool, {}
