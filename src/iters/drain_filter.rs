
use ::*;
use std::{
    iter::*,
    ops::Drop,
};

macro_rules! list {
    ($ptr:expr) => (unsafe { &mut *$ptr });
}

pub fn new_drain_filter<'t, T: 't, F,>(list: &'t mut VecList<T,>, filter: F,) -> DrainFilter<'t, T, F,>
    where F: FnMut(&mut T,) -> bool, {
    DrainFilter { list, ends: list.ends, filter, }
}

#[derive(PartialEq, Eq, PartialOrd, Ord,)]
pub struct DrainFilter<'t, T: 't, F,>
    where F: FnMut(&mut T,) -> bool, {
    list: *mut VecList<'t, T,>,
    ends: Option<(usize, usize,)>,
    filter: F,
}

impl<'t, T: 't, F,> Drop for DrainFilter<'t, T, F,>
    where F: FnMut(&mut T,) -> bool, {
    #[inline]
    fn drop(&mut self) { self.for_each(|_| ()) }
}

impl<'t, T: 't, F,> Iterator for DrainFilter<'t, T, F,>
    where F: FnMut(&mut T,) -> bool, {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item,> {
        let list = list!(self.list);

        match self.ends {
            None => None,
            Some((node, tail,)) => {
                let passed = (self.filter)(&mut list.nodes[node].value);

                if passed {
                    if let Some((l_head, l_tail,)) = list.ends {
                        match (node == l_head, node == l_tail,) {
                            (true, true,) => list.ends = None,
                            (true, false,) => list.ends = list.nodes[node].next.map(|next| (next, l_tail,)),
                            (false, true,) => list.ends = list.nodes[node].prev.map(|prev| (l_head, prev,)),
                            _ => (),
                        }
                    }
                }
                self.ends = if node == tail { None }
                    else { list.nodes[node].next.map(|next| (next, tail,)) };
                
                if passed { Some(list.shelf_node(node,)) }
                else { return self.next() }
            },
        }
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

impl<'t, T: 't, F,> DoubleEndedIterator for DrainFilter<'t, T, F,>
    where F: FnMut(&mut T,) -> bool, {
    fn next_back(&mut self) -> Option<Self::Item,> {
        let list = list!(self.list);

        match self.ends {
            None => None,
            Some((head, node,)) => {
                let passed = (self.filter)(&mut list.nodes[node].value);

                if passed {
                    if let Some((l_head, l_tail,)) = list.ends {
                        match (node == l_head, node == l_tail,) {
                            (true, true,) => list.ends = None,
                            (true, false,) => list.ends = list.nodes[node].next.map(|next| (next, l_tail,)),
                            (false, true,) => list.ends = list.nodes[node].prev.map(|prev| (l_head, prev,)),
                            _ => (),
                        }
                    }
                }
                self.ends = if node == head { None }
                    else { list.nodes[node].prev.map(|prev| (head, prev,)) };
                
                if passed { Some(list.shelf_node(node,)) }
                else { return self.next_back() }
            },
        }
    }
}

impl<'t, T: 't, F,> FusedIterator for DrainFilter<'t, T, F,> 
    where F: FnMut(&mut T,) -> bool, {}
