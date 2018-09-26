//! [`vec-list`] is an implementation of a Doubly-Linked-List using an underlying [`Vec`]
//! to store the nodes so as to avoid cache misses during access.
//! 
//! Author --- daniel.bechaz@gmail.com  
//! Last Moddified --- 2018-09-24

#![feature(exact_size_is_empty)]
#![feature(min_const_fn)]
#![feature(const_vec_new)]
#![feature(const_manually_drop_new)]
#![feature(trusted_len)]
#![feature(nll)]

extern crate imply_option;

use std::{
    cmp::{PartialEq, PartialOrd, Ord, Ordering,},
    iter::{FromIterator, Extend,},
    ops::{Index, IndexMut, RangeBounds,},
    fmt::{self, Debug,},
};

mod nodes;
mod iters;
mod views;
mod tests;

use self::nodes::*;
use self::iters::{Iter, IterMut, Drain, DrainFilter,};
pub use self::views::{View, ViewMut,};

/// An implementation of a doubly-linked-list backed by a `Vec` so that it will avoid
/// cache misses during access.
#[derive(Eq, Clone,)]
pub struct VecList<'t, T: 't,> {
    /// The [`Node`](struct.Node.html)s of the [`VecList`].
    nodes: Vec<Node<'t, T,>,>,
    /// The number of [`Node`](struct.Node.html)s in the [`VecList`].
    len: usize,
    /// The starting and ending indexes of the linked list in [`nodes`]
    ends: Option<(usize, usize,)>,
    /// A stack of allocated [`Node`](struct.Node.html)s not used in the linked list.
    empty: Option<usize,>,
}

impl<'t, T: 't,> VecList<'t, T,> {
    /// Append the `to` to `from`.
    /// 
    /// # Params
    /// 
    /// from --- The index of the [`Node`](struct.Node.html) to append to.  
    /// to --- The index of the [`Node`](struct.Node.html) to append.
    /// 
    /// # Panics
    /// 
    /// * If `from` has a `next` pointer.  
    /// * If `to` has a `prev` pointer.
    fn append(&mut self, from: usize, to: usize,) {
        debug_assert!(self.nodes[from].next.is_none(), format!("`from` has `next`: {:?}", from,),);
        debug_assert!(self.nodes[to].prev.is_none(), format!("`to` has `prev`: {:?}", to,),);

        //Append the Nodes.
        self.nodes[from].next = Some(to);
        self.nodes[to].prev = Some(from);
    }
    /// Disconnect the passed [`Node`](struct.Node.html).
    /// 
    /// # Params
    /// 
    /// node --- The index of the [`Node`](struct.Node.html) to disconnect.
    fn disconnect(&mut self, node: usize,) {
        //Disconnect `node`.
        let next = self.nodes[node].next.take();
        let prev = self.nodes[node].prev.take();
        
        //Relink `node`s neighbours.
        if let Some(next) = next { self.nodes[next].prev = prev }
        if let Some(prev) = prev { self.nodes[prev].next = next }
    }
    /// Creates a new [`Node`](struct.Node.html) and returns its index.
    /// 
    /// # Params
    /// 
    /// value --- The value to populate the [`Node`](struct.Node.html) with.
    fn new_node(&mut self, value: T,) -> usize {
        //Increase the length.
        self.len += 1;

        match self.empty {
            //There is a preallocated empty `Node`.
            Some(new) => {
                //Pop the empty `Node`.
                self.empty = self.nodes[new].next;
                //Disconnect the popped `Node`.
                self.disconnect(new);
                //Populate the `Node`.
                self.nodes[new].value = ManuallyDrop::new(value);

                new
            },
            //There is no preallocated empty `Node`.
            None => {
                //Get the index of the new `Node`.
                let new = self.nodes.len();

                //Push the new `Node`.
                self.nodes.push(Node::new(value));

                new
            },
        }
    }
    /// Empty the passed [`Node`](struct.Node.html).
    /// 
    /// The [`Node`](struct.Node.html) will be placed on the `empty` stack.  
    /// The value in the [`Node`](struct.Node.html) is returned.
    /// 
    /// # Params
    /// 
    /// node --- The [`Node`](struct.Node.html) to remove.
    fn remove_node(&mut self, node: usize,) -> T {
        //Decrement the `Node` count.
        self.len -= 1;
        
        //Disconnect the `Node`.
        self.disconnect(node,);
        //Link the `Node` to the empty stack.
        if let Some(empty) = self.empty { self.append(node, empty) }
        //Push the `Node` onto the empty stack.
        self.empty = Some(node);
        //Return the value.
        unsafe { self.nodes[node].move_value() }
    }
    /// Returns the index of the [`Node`](struct.Node.html) representing `index` in the
    /// linked list.
    /// 
    /// # Params
    /// 
    /// index --- The `index` in the linked-list to find.
    /// 
    /// # Panics
    /// 
    /// * If `index` is out of bounds.
    fn get_index(&self, index: usize,) -> usize {
        assert!(index <= self.len, "`index` is out of bounds",);

        //Calculate the nunber of steps backwards to take.
        let back_step = self.len - 1 - index;

        //Take the smallest number of steps to the `Node`.
        if index <= back_step { self.index_forward(index) }
        else { self.index_backward(back_step) }
    }
    /// Returns the index of the [`Node`](struct.Node.html) some steps forward in the
    /// linked list.
    /// 
    /// # Params
    /// 
    /// steps --- The number of steps to take forward in the linked list.
    /// 
    /// # Panics
    /// 
    /// * If `steps` is out of bounds.
    fn index_forward(&self, steps: usize,) -> usize {
        /// Step to the specified index.
        /// 
        /// # Params
        /// 
        /// list --- The [`VecList`] to index in.  
        /// steps --- The number of steps to take.  
        /// at --- The current position.
        fn index<T>(list: &VecList<T,>, steps: usize, at: usize,) -> usize {
            //Check if there are more steps to do.
            if steps == 0 { return at }

            //Take the next step.
            index(list, steps - 1, list.nodes[at].next.expect("index_forward: 2"))
        }
        
        eprintln!("temp: {}, {}, {:?}", self.len(), steps, self.ends,);
        //Step forward.
        index(self, steps, self.ends.expect("index_forward: 1").0)
    }
    /// Returns the index of the [`Node`](struct.Node.html) some steps backwards in the
    /// linked list.
    /// 
    /// # Params
    /// 
    /// steps --- The number of steps to take backward in the linked list.
    /// 
    /// # Panics
    /// 
    /// * If `steps` is out of bounds.
    fn index_backward(&self, steps: usize,) -> usize {
        /// Step to the specified index.
        /// 
        /// # Params
        /// 
        /// list --- The [`VecList`] to index in.  
        /// steps --- The number of steps to take.  
        /// at --- The current position.
        fn index<T>(list: &VecList<T,>, steps: usize, at: usize,) -> usize {
            //Check if there are more steps to do.
            if steps == 0 { return at }

            //Take the next step.
            index(list, steps - 1, list.nodes[at].prev.expect("index_backward: 2"))
        }
        
        //Step backward.
        index(self, steps, self.ends.expect("index_backward: 1").1)
    }
}

impl<'t, T: 't,> VecList<'t, T,> {
    /// Returns a new empty [`VecList`].
    #[inline]
    pub const fn new() -> Self {
        Self { nodes: Vec::new(), len: 0, ends: None, empty: None, }
    }
    /// Returns a new empty [`VecList`] with room for `capacity` [`Node`](struct.Node.html)s.
    /// 
    /// # Params
    /// 
    /// capacity --- The number of spaces to allocate.
    #[inline]
    pub fn with_capacity(capacity: usize,) -> Self {
        Self { nodes: Vec::with_capacity(capacity), len: 0, ends: None, empty: None, }
    }
    /// Reserves space for at least `additional` number of [`Node`](struct.Node.html)s.
    /// 
    /// # Params
    /// 
    /// additional --- The number of additional spaces to allocate for.
    pub fn reserve(&mut self, mut additional: usize,) {
        //The stack of empty `Node`s.
        let mut empty = self.empty;

        loop {
            match empty {
                //No more empty `Node`s.
                None => break,
                //Remove the empty `Node` from the `additional` count.
                Some(node,) => {
                    //If this is the last additional space, no additional spaces are needed.
                    if additional == 1 { return }
                    additional -= 1;
                    empty = self.nodes[node].next;
                },
            }
        }

        //Reserve the necessary nodes.
        self.nodes.reserve(additional,)
    }
    /// Reserves space for exactly `additional` number of [`Node`](struct.Node.html)s.
    /// 
    /// # Params
    /// 
    /// additional --- The number of additional spaces to allocate.
    pub fn reserve_exact(&mut self, mut additional: usize,) {
        //The stack of empty `Node`s.
        let mut empty = self.empty;

        loop {
            match empty {
                //No more empty `Node`s.
                None => break,
                //Remove the empty `Node` from the `additional` count.
                Some(node,) => {
                    //If this is the last additional space, no additional spaces are needed.
                    if additional == 1 { return }
                    additional -= 1;
                    empty = self.nodes[node].next;
                },
            }
        }

        //Reserve the necessary nodes.
        self.nodes.reserve_exact(additional,)
    }
    /// Shrinks the [`VecList`] to only store the existing values.
    #[inline]
    pub fn shrink_to_fit(&mut self,) {
        /// Forgets empty `Nodes` until a node which will be retained is found.
        /// 
        /// # Params
        /// 
        /// list --- The [`VecList`] to clean.
        #[inline]
        fn clean_empty<'t, T: 't,>(list: &mut VecList<'t, T,>) {
            //Get the head of the empty stack.
            if let Some(empty) = list.empty {
                //If this node is going to be retained, stop.
                if empty >= list.len() {
                    //Pop the head of the empty `Node`.
                    list.empty = list.nodes[empty].next;
                    //Keep cleaning.
                    clean_empty(list);
                }
            }
        }
        
        //Get the ends of the [`VecList`].
        if let Some((mut head, tail,)) = self.ends {
            //Check if the head needs to be cleaned.
            if head >= self.len() {
                //Get the next empty `Node`.
                clean_empty(self);
                
                //Specify the new `head` `Node`.
                let new_head = self.empty.expect("shrink_to_fit: 1");

                //Pop the `Node` of the empty stack.
                self.empty = self.nodes[new_head].next;
                //Populate the new `head` `Node` to be the previous `head` `Node`.
                self.nodes[new_head] = unsafe { (&mut self.nodes[head] as *mut Node<T,>).read() };
                //Update the `prev` pointer for the next `Node` to point at the new `head` `Node`.
                self.nodes[new_head].next.map(
                    |next| self.nodes[next].prev = Some(new_head)
                );

                //Update the `head` pointer.
                head = new_head;
                //If there is only a single `Node`, update the tail pointer and return.
                if tail == head {
                    return self.ends = Some((new_head, new_head,));
                //Else update the head pointer and keep cleaning.
                } else {
                    self.ends = Some((new_head, tail,));
                }
            }

            let mut cur_node = head;
            //Loop while the empty stack contains `Node`s to populate.
            loop {
                //Get the next empty `Node`.
                clean_empty(self);
                //Check if there is an empty `Node` to populate.
                match self.empty {
                    None => break,
                    //There is an empty `Node` to populate.
                    Some(new_node) => {
                        //Get an invalid `Node`.
                        while cur_node < self.len() {
                            cur_node = self.nodes[cur_node].next.expect("shrink_to_fit: 2");
                        }

                        //Populate the new `Node` to be the same as the old `Node`.
                        self.nodes[new_node] = unsafe { (&mut self.nodes[cur_node] as *mut Node<T,>).read() };
                        //---Update the new `Node`s neighbours.---
                        self.nodes[new_node].next.map(
                            |next| self.nodes[next].prev = Some(new_node)
                        );
                        self.nodes[new_node].prev.map(
                            |prev| self.nodes[prev].next = Some(new_node)
                        );
                    },
                }
            }
        }

        //Remove all empty `Node`s.
        self.nodes.truncate(self.len);
        //Deallocate all unneeded space.
        self.nodes.shrink_to_fit()
    }
    /// Clear the [`VecList`].
    #[inline]
    pub fn clear(&mut self) { self.drain(..); }
    /// Returns the number of values in this [`VecList`].
    #[inline]
    pub const fn len(&self) -> usize { self.len }
    /// `true` if this [`VecList`] is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool { self.len() == 0 }
    /// Returns the number of spaces in this [`VecList`].
    #[inline]
    pub fn capacity(&self) -> usize { self.nodes.capacity() }
    /// Returns the first value.
    #[inline]
    pub fn front(&self) -> Option<&T,> {
        self.ends.map(|(head, _,)| &*self.nodes[head].value)
    }
    /// Returns the first value.
    #[inline]
    pub fn front_mut(&mut self) -> Option<&mut T,> {
        self.ends.map(move |(head, _,)| &mut *self.nodes[head].value)
    }
    /// Returns the last value.
    #[inline]
    pub fn back(&self) -> Option<&T,> {
        self.ends.map(|(_, tail,)| &*self.nodes[tail].value)
    }
    /// Returns the last value.
    #[inline]
    pub fn back_mut(&mut self) -> Option<&mut T,> {
        self.ends.map(move |(_, tail,)| &mut *self.nodes[tail].value)
    }
    /// Pushes a value onto the front of the [`VecList`].
    /// 
    /// # Params
    /// 
    /// The value to push on.
    pub fn push_front(&mut self, value: T,) {
        let new = self.new_node(value,);

        self.ends = match self.ends {
            None => Some((new, new,)),
            Some((head, tail,)) => {
                self.append(new, head,);
                
                Some((new, tail,))
            }
        };
    }
    /// Pops the first value off the front of the [`VecList`].
    pub fn pop_front(&mut self,) -> Option<T,> {
        match self.ends {
            None => None,
            Some((head, tail,)) => {
                self.ends = self.nodes[head].next
                    .map(|next| (next, tail,));
                self.remove_node(head,);

                Some(unsafe { self.nodes[head].move_value() },)
            }
        }
    }
    /// Pushes a value onto the back of the [`VecList`].
    /// 
    /// # Params
    /// 
    /// The value to push on.
    pub fn push_back(&mut self, value: T,) {
        let new = self.new_node(value,);

        self.ends = match self.ends {
            None => Some((new, new,)),
            Some((head, tail,)) => {
                self.append(tail, new,);
                
                Some((head, new,))
            }
        };
    }
    /// Pops the first value off the front of the [`VecList`].
    pub fn pop_back(&mut self,) -> Option<T,> {
        match self.ends {
            None => None,
            Some((head, tail,)) => {
                self.ends = self.nodes[head].prev
                    .map(|prev| (head, prev,));
                self.remove_node(tail,);

                Some(unsafe { self.nodes[tail].move_value() },)
            }
        }
    }
    /// Removes all values that don't pass the `pred` filter.
    /// 
    /// # Params
    /// 
    /// pred --- The filter function values need to pass to be retained.
    pub fn retain(&mut self, mut pred: impl FnMut(&T) -> bool,) {
        use imply_option::*;

        if let Some((mut index, _,)) = self.ends {
            loop {
                let next = self.nodes[index].next;
                
                if !pred(&self.nodes[index].value) {
                    self.ends = self.ends.and_then(
                        |(head, tail,)| (head != index).then(head).or(next)
                            .and_then(|head| (tail != index).then((head, tail,))
                                .or(self.nodes[index].prev.map(|tail| (head, tail,)))
                            )
                    );
                    
                    self.remove_node(index);
                }

                match next {
                    Some(next) => index = next,
                    None => break,
                }
            }
        }
    }
    /// Splits the [`VecList`] into two lists at the passed index.
    /// 
    /// # Params
    /// 
    /// at --- The index to split the [`VecList`] at.
    /// 
    /// # Panics
    /// 
    /// * If `at` is greater than the length of the [`VecList`].
    pub fn split_at(&mut self, at: usize,) -> VecList<T,> {
        assert!(at <= self.len(), "`at` was greater than the length of the `VecList`",);

        //The list to return.
        let mut list = VecList::with_capacity(self.len - 1 - at);
        
        //Push all the end values onto the new [`VecList`].
        list.extend(self.drain(at..));

        list
    }
    /// Returns an iterator over all values in the [`VecList`].
    #[inline]
    pub fn iter(&self) -> Iter<T> {
        iters::new_iter(self, self.ends)
    }
    /// Returns a mutable iterator over all values in the [`VecList`].
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<T> {
        iters::new_iter_mut(self, self.ends)
    }
    /// Returns a view into the [`VecList`] at the passed index.
    /// 
    /// # Params
    /// 
    /// index --- The index to start the [`View`](struct.View.html) at.
    /// 
    /// # Panics
    /// 
    /// * If `index` is greater than or equal to the length of the [`VecList`].
    #[inline]
    pub fn view(&self, index: usize) -> View<T> {
        views::new_view(self, self.get_index(index),)
    }
    /// Returns a mutable view into the [`VecList`] at the passed index.
    /// 
    /// # Params
    /// 
    /// index --- The index to start the [`ViewMut`](struct.ViewMut.html) at.
    /// 
    /// # Panics
    /// 
    /// * If `index` is greater than or equal to the length of the [`VecList`].
    #[inline]
    pub fn view_mut(&mut self, index: usize) -> ViewMut<T> {
        views::new_view_mut(self, self.get_index(index),)
    }
    /// Removes the values of `range` from the [`VecList`] and yields them as an iterator.
    /// 
    /// NOTE!!! Even if the iterator is not used, all values contained in the range will be removed.
    /// 
    /// # Params
    /// 
    /// range --- The range of values to remove.
    /// 
    /// # Panics
    /// 
    /// * If `range` uses fixed bounds which are not contained inside the [`VecList`].
    #[inline]
    pub fn drain<R>(&mut self, range: R,) -> Drain<T,>
        where R: RangeBounds<usize,> {
        iters::new_drain(self, range,)
    }
    /// Applies `filter` to each value in [`VecList`] and yields the values which return
    /// `false`.
    /// 
    /// NOTE!!! The filter is given mutable access to all values, including the ones not returned.
    /// 
    /// # Params
    /// 
    /// filter --- The filter to apply to each value.
    #[inline]
    pub fn drain_filter<F>(&mut self, filter: F,) -> DrainFilter<T, F,>
        where F: FnMut(&mut T,) -> bool, {
        iters::new_drain_filter(self, filter)
    }
}

impl<'t, T: 't + PartialEq,> VecList<'t, T,> {
    /// Returns `true` if the `x` is found in the [`VecList`].
    /// 
    /// # Params
    /// 
    /// x --- The value to search for.
    pub fn contains(&mut self, x: &T,) -> bool {
        self.iter().find(|&y| x == y).is_some()
    }
}

impl<'t, T: 't,> Index<usize,> for VecList<'t, T,> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize,) -> &Self::Output {
        &self.nodes[self.get_index(index)].value
    }
}

impl<'t, T: 't,> IndexMut<usize,> for VecList<'t, T,> {
    #[inline]
    fn index_mut(&mut self, index: usize,) -> &mut Self::Output {
        let index = self.get_index(index);

        &mut self.nodes[index].value
    }
}

impl<'t, T: 't, A: Into<T,>,> Extend<A,> for VecList<'t, T,> {
    fn extend<I,>(&mut self, iter: I)
        where I: IntoIterator<Item = A>, {
        iter.into_iter().for_each(|item| self.push_back(item.into()))
    }
}

impl<'t, T: 't, A: Into<T>,> FromIterator<A,> for VecList<'t, T,> {
    fn from_iter<I,>(iter: I) -> Self
        where I: IntoIterator<Item = A>, {
        let mut list = VecList::new();

        list.extend(iter); list
    }
}

impl<'t, T: 't + PartialEq,> PartialEq for VecList<'t, T,> {
    fn eq(&self, rhs: &Self) -> bool {
        self.iter().zip(rhs.iter())
        .all(|(lhs, rhs,)| lhs == rhs)
    }
}

impl<'t, T: 't + PartialOrd,> PartialOrd for VecList<'t, T,> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering,> {
        for (lhs, rhs,) in self.iter().zip(rhs.iter()) {
            match lhs.partial_cmp(rhs,) {
                Some(Ordering::Equal) => (),
                cmp => return cmp,
            }
        }

        Some(Ordering::Equal)
    }
}

impl<'t, T: 't + Ord,> Ord for VecList<'t, T,> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        for (lhs, rhs,) in self.iter().zip(rhs.iter()) {
            match lhs.cmp(rhs,) {
                Ordering::Equal => (),
                cmp => return cmp,
            }
        }

        Ordering::Equal
    }
}

impl<'t, T: 't + Debug,> Debug for VecList<'t, T,> {
    fn fmt(&self, fmt: &mut fmt::Formatter,) -> fmt::Result {
        fmt.debug_list().entries(self.iter()).finish()
    }
}

impl<'t, T: 't,> Drop for VecList<'t, T,> {
    #[inline]
    fn drop(&mut self,) { self.clear() }
}
