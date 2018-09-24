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

#[derive(Eq, Clone,)]
pub struct VecList<'t, T: 't,> {
    nodes: Vec<Node<'t, T,>,>,
    len: usize,
    ends: Option<(usize, usize,)>,
    empty: Option<usize,>,
}

impl<'t, T: 't,> VecList<'t, T,> {
    fn append(&mut self, from: usize, to: usize,) {
        debug_assert!(self.nodes[from].next.is_none(), format!("`from` has `next`: {:?}", from,),);
        debug_assert!(self.nodes[to].prev.is_none(), format!("`to` has `prev`: {:?}", to,),);

        self.nodes[from].next = Some(to);
        self.nodes[to].prev = Some(from);
    }
    fn disconnect(&mut self, node: usize,) {
        let next = self.nodes[node].next.take();
        let prev = self.nodes[node].prev.take();
        
        if let Some(next) = next { self.nodes[next].prev = prev }
        if let Some(prev) = prev { self.nodes[prev].next = next }
    }
    fn new_node(&mut self, value: T,) -> usize {
        self.len += 1;

        match self.empty {
            Some(new) => {
                self.empty = self.nodes[new].next;
                self.disconnect(new);
                self.nodes[new].value = ManuallyDrop::new(value);

                new
            },
            None => {
                let new = self.nodes.len();

                self.nodes.push(Node::new(value));

                new
            },
        }
    }
    fn shelf_node(&mut self, node: usize,) -> T {
        self.len -= 1;
        
        self.disconnect(node,);
        if let Some(empty) = self.empty { self.append(node, empty) }
        self.empty = Some(node);
        unsafe { (&*self.nodes[node].value as *const T).read() }
    }
    fn get_index(&self, index: usize,) -> usize {
        assert!(index <= self.len, "`index` is out of bounds",);

        let back_step = self.len - 1 - index;

        if index <= back_step { self.index_forward(index) }
        else { self.index_backward(back_step) }
    }
    fn index_forward(&self, steps: usize,) -> usize {
        let mut index = self.ends.expect("index_forward: 1").0;

        for _ in 0..steps { index = self.nodes[index].next.expect("index_forward: 2") }
        index
    }
    fn index_backward(&self, steps: usize,) -> usize {
        let mut index = self.ends.expect("index_backward: 1").1;

        for _ in 0..steps { index = self.nodes[index].prev.expect("index_backward: 2") }
        index
    }
}

impl<'t, T: 't,> VecList<'t, T,> {
    #[inline]
    pub const fn new() -> Self {
        Self { nodes: Vec::new(), len: 0, ends: None, empty: None, }
    }
    #[inline]
    pub fn with_capacity(capacity: usize,) -> Self {
        Self { nodes: Vec::with_capacity(capacity), len: 0, ends: None, empty: None, }
    }
    #[inline]
    pub fn reserve(&mut self, mut additional: usize,) {
        let mut empty = self.empty;

        loop {
            match empty {
                None => break,
                Some(node,) => {
                    additional -= 1;
                    empty = self.nodes[node].next;
                },
            }
        }

        self.nodes.reserve(additional,)
    }
    #[inline]
    pub fn reserve_exact(&mut self, mut additional: usize,) {
        let mut empty = self.empty;

        loop {
            match empty {
                None => break,
                Some(node,) => {
                    additional -= 1;
                    empty = self.nodes[node].next;
                },
            }
        }

        self.nodes.reserve_exact(additional,)
    }
    #[inline]
    pub fn shrink_to_fit(&mut self,) {
        fn clean_empty<'t, T: 't,>(list: &mut VecList<'t, T,>) {
            if let Some(empty) = list.empty {
                if empty >= list.len() {
                    list.empty = list.nodes[empty].next;
                    clean_empty(list);
                }
            }
        }

        if let Some((mut head, mut tail,)) = self.ends {
            if head >= self.len() {
                clean_empty(self);
                let new_head = self.empty.expect("shrink_to_fit: 1");
                self.empty = self.nodes[new_head].next;
                self.nodes[new_head] = unsafe { (&mut self.nodes[head] as *mut Node<T,>).read() };
                self.nodes[new_head].next.map(
                    |next| self.nodes[next].prev = Some(new_head)
                );

                if tail == head { tail = new_head }
                head = new_head;
            }

            if tail >= self.len() {
                clean_empty(self);
                let new_tail = self.empty.expect("shrink_to_fit: 2");
                self.empty = self.nodes[new_tail].next;
                self.nodes[new_tail] = unsafe { (&mut self.nodes[tail] as *mut Node<T,>).read() };
                self.nodes[new_tail].next.map(
                    |next| self.nodes[next].prev = Some(new_tail)
                );

                tail = new_tail;
            }

            self.ends = Some((head, tail,));
            let mut cur_node = head;
            loop {
                clean_empty(self);
                if let Some(new_node) = self.empty {
                    if new_node < self.len() { break }

                    self.empty = self.nodes[new_node].next;
                    while cur_node < self.len() {
                        cur_node = self.nodes[cur_node].next.expect("shrink_to_fit: 4");
                    }

                    self.nodes[new_node] = unsafe { (&mut self.nodes[cur_node] as *mut Node<T,>).read() };
                    self.nodes[new_node].next.map(
                        |next| self.nodes[next].prev = Some(new_node)
                    );
                    self.nodes[new_node].prev.map(
                        |prev| self.nodes[prev].next = Some(new_node)
                    );
                }
            }
        }

        self.nodes.truncate(self.len);
        self.nodes.shrink_to_fit()
    }
    #[inline]
    pub fn clear(&mut self) { self.drain(..); }
    #[inline]
    pub const fn len(&self) -> usize { self.len }
    #[inline]
    pub const fn is_empty(&self) -> bool { self.len() == 0 }
    #[inline]
    pub fn capacity(&self) -> usize { self.nodes.capacity() }
    #[inline]
    pub fn front(&self) -> Option<&T,> {
        self.ends.map(|(head, _,)| &*self.nodes[head].value)
    }
    #[inline]
    pub fn front_mut(&mut self) -> Option<&mut T,> {
        self.ends.map(move |(head, _,)| &mut *self.nodes[head].value)
    }
    #[inline]
    pub fn back(&self) -> Option<&T,> {
        self.ends.map(|(_, tail,)| &*self.nodes[tail].value)
    }
    #[inline]
    pub fn back_mut(&mut self) -> Option<&mut T,> {
        self.ends.map(move |(_, tail,)| &mut *self.nodes[tail].value)
    }
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
    pub fn pop_front(&mut self,) -> Option<T,> {
        match self.ends {
            None => None,
            Some((head, tail,)) => {
                self.ends = self.nodes[head].next
                    .map(|next| (next, tail,));
                self.shelf_node(head,);

                Some(unsafe { self.nodes[head].value() },)
            }
        }
    }
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
    pub fn pop_back(&mut self,) -> Option<T,> {
        match self.ends {
            None => None,
            Some((head, tail,)) => {
                self.ends = self.nodes[head].prev
                    .map(|prev| (head, prev,));
                self.shelf_node(tail,);

                Some(unsafe { self.nodes[tail].value() },)
            }
        }
    }
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
                    
                    self.shelf_node(index);
                }

                match next {
                    Some(next) => index = next,
                    None => break,
                }
            }
        }
    }
    pub fn split_at(&mut self, at: usize,) -> VecList<T,> {
        assert!(at <= self.len(), "`at` was greater than the length of the `VecList`",);

        if at == self.len() { VecList::new() }
        else { VecList::from_iter(self.drain(at..)) }
    }
    #[inline]
    pub fn iter(&self) -> Iter<T> {
        iters::new_iter(self, self.ends)
    }
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<T> {
        iters::new_iter_mut(self, self.ends)
    }
    #[inline]
    pub fn view(&self, index: usize) -> View<T> {
        views::new_view(self, &self.nodes[self.get_index(index)],)
    }
    #[inline]
    pub fn view_mut(&mut self, index: usize) -> ViewMut<T> {
        views::new_view_mut(self, self.get_index(index),)
    }
    #[inline]
    pub fn drain<R>(&mut self, range: R,) -> Drain<T,>
        where R: RangeBounds<usize,> {
        iters::new_drain(self, range,)
    }
}

impl<'t, T: 't + std::fmt::Display,> VecList<'t, T,> {
    #[inline]
    pub fn drain_filter<F>(&mut self, filter: F,) -> DrainFilter<T, F,>
        where F: FnMut(&mut T,) -> bool, {
        iters::new_drain_filter(self, filter)
    }
}

impl<'t, T: 't + PartialEq,> VecList<'t, T,> {
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
