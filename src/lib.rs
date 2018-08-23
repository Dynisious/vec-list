//! [`vec-list`] is an implementation of a Doubly-Linked-List using an underlying [`VecList`]
//! to store the nodes so as to avoid cache misses during access.
//! 
//! Author --- daniel.bechaz@gmail.com  
//! Last Moddified --- 2018-08-21

#![cfg_attr(feature = "try_reserve", feature(try_reserve))]

#[cfg(feature = "try_reserve")]
use std::collections::CollectionAllocErr;
use std::{
    iter::Extend,
    ops::{RangeBounds, Bound,},
};

mod node;
mod drain;
mod iter;

use self::node::*;
use self::drain::{drain_new,};
pub use self::drain::{Drain,};
pub use self::iter::{Iter, IterMut,};

/// `VecList` is a Doubly-Linked-List which stores it's nodes internally in an underlying `Vec`.
/// 
/// This data structure is useful for when you want fast insertions to the front and
/// middle of the list.
pub struct VecList<T> {
    head: Option<*mut Node<T>>,
    tail: Option<*mut Node<T>>,
    len: usize,
    /// Note! cache `Node`s are only singly linked.
    cache: Option<*mut Node<T>>,
    nodes: Vec<Node<T>>,
}

impl<T> VecList<T> {
    /// Returns the number of cached `Nodes` which are ready to be used.
    fn cached(&self) -> usize {
        self.cache
        //Count the `Nodes` in the cache.
        .map(|cache| unsafe { (*cache).len() })
        //Default to 0 `Nodes`.
        .unwrap_or(0)
    }
    /// Returns the node representing `index`.
    /// 
    /// # Params
    /// 
    /// index --- The index of the `Node` to get.
    fn get_node(&self, index: usize) -> *const Node<T> {
        assert!(!self.is_empty() && index < self.len(),
            "Invalid index; index:{}, len:{}", index, self.len(),
        );

        let reverse = self.len() - index - 1;
        let mut node;

        unsafe {
            if index < reverse {
                node = self.head.expect("`VecList` was indicated not empty but no head was found");

                for _ in 0..index {
                    node = (*node).next.expect("`index` was in bounds but no `next` was found")
                }
            } else {
                node = self.tail.expect("`VecList` was indicated not empty but no tail was found");

                for _ in 0..reverse {
                    node = (*node).prev.expect("`index` was in bounds but no `prev` was found")
                }
            }
        }

        node
    }
    /// Returns a reference too a new `Node` in the `VecList`.
    fn new_node(&mut self, value: T) -> &mut Node<T> {
        use std::mem::ManuallyDrop;

        //Check if there are any cached `Nodes`.
        match self.cache.take() {
            //There is a cached `Node`.
            Some(new) => unsafe {
                //Populate the value.
                (*new).value = ManuallyDrop::new(value);
                //Update the cache.
                self.cache = (*new).pop_next();

                //Return a reference to the `Node`.
                &mut *new
            },
            //There is no cached `Node`; make a new one.
            None => {
                //Remember the index the new `Node` will occupy.
                let new = self.nodes.len();

                //Make the new `Node`.
                self.nodes.push(Node {
                    prev: None,
                    next: None,
                    value: ManuallyDrop::new(value),
                });
                //Return a reference too the `Node`.
                //Safe because the new `Node` was placed at `new` when we created it.
                unsafe { self.nodes.get_unchecked_mut(new) }
            },
        }
    }
    /// Adds the passed node too the cache.
    fn cache_node(&mut self, node: &mut Node<T>) {
        //Append the cache too the `Node`.
        unsafe { node.append(self.cache) }
        //Update the head of the cache.
        self.cache = Some(node);
    }
}

impl<T> VecList<T> {
    /// Creates an empty `VecList`.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    ///
    /// let list: VecList<u32> = VecList::new();
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
            cache: None,
            nodes: Vec::new(),
        }
    }
    /// Creates an empty `VecList` with enough space in the underlying [`Vec`] for `capacity` elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    ///
    /// let list: VecList<u32> = VecList::with_capacity(10);
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
            cache: None,
            nodes: Vec::with_capacity(capacity),
        }
    }
    /// Returns the number of elements the `VecList` can hold without reallocating.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    /// 
    /// let list: VecList<i32> = VecList::with_capacity(10);
    /// 
    /// assert_eq!(list.capacity(), 10);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize { self.nodes.capacity() }
    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the underlying [`Vec<T>`].
    /// 
    /// Refer to [`Vec::reserve`] for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    /// 
    /// let mut list = VecList::from(vec![1]);
    /// 
    /// list.reserve(10);
    /// assert!(list.capacity() >= 11);
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional: usize) { self.nodes.reserve(additional) }
    /// Reserves the minimum capacity for exactly `additional` more elements to
    /// be inserted in the underlying [`Vec<T>`].
    /// 
    /// Refer to [`Vec::reserve_exact`] for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    /// 
    /// let mut list = VecList::from(vec![1]);
    /// 
    /// list.reserve_exact(10);
    /// assert!(list.capacity() >= 11);
    /// ```
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) { self.nodes.reserve_exact(additional) }
    /// Tries to reserve capacity for at least `additional` more elements to be inserted
    /// in the underlying [`Vec<T>`].
    /// 
    /// Refer to [`Vec::try_reserve`] for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    /// #![feature(try_reserve)]
    /// use std::collections::CollectionAllocErr;
    ///
    /// fn process_data(data: &[u32]) -> Result<VecList<u32>, CollectionAllocErr> {
    ///     let mut output = VecList::new();
    ///
    ///     // Pre-reserve the memory, exiting if we can't
    ///     output.try_reserve(data.len())?;
    ///
    ///     // Now we know this can't OOM in the middle of our complex work
    ///     output.extend(data.iter().map(|&val| {
    ///         val * 2 + 5 // very complicated
    ///     }));
    ///
    ///     Ok(output)
    /// }
    /// # process_data(&[1, 2, 3]).expect("why is the test harness OOMing on 12 bytes?");
    /// ```
    #[cfg(feature = "try_reserve")]
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), CollectionAllocErr> {
        self.nodes.try_reserve(additional)
    }
    /// Tries to reserves the minimum capacity for exactly `additional` more elements to
    /// be inserted in the underlying [`Vec<T>`].
    /// 
    /// Refer to [`Vec::try_reserve_exact`] for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    /// #![feature(try_reserve)]
    /// use std::collections::CollectionAllocErr;
    ///
    /// fn process_data(data: &[u32]) -> Result<VecList<u32>, CollectionAllocErr> {
    ///     let mut output = VecList::new();
    ///
    ///     // Pre-reserve the memory, exiting if we can't
    ///     output.try_reserve(data.len())?;
    ///
    ///     // Now we know this can't OOM in the middle of our complex work
    ///     output.extend(data.iter().map(|&val| {
    ///         val * 2 + 5 // very complicated
    ///     }));
    ///
    ///     Ok(output)
    /// }
    /// # process_data(&[1, 2, 3]).expect("why is the test harness OOMing on 12 bytes?");
    /// ```
    #[cfg(feature = "try_reserve",)]
    #[inline]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), CollectionAllocErr> {
        self.nodes.try_reserve_exact(additional)
    }
    /// Shrinks the capacity of the underlying [`Vec`] as much as possible.
    ///
    /// Refer to [`Vec::shrink_to_fit`] for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    /// 
    /// let mut list = VecList::with_capacity(10);
    /// 
    /// list.extend([1, 2, 3].iter().cloned());
    /// assert_eq!(list.capacity(), 10);
    /// 
    /// list.shrink_to_fit();
    /// assert!(list.capacity() >= 3);
    /// ```
    #[inline]
    pub fn shrink_to_fit(&mut self) { self.nodes.shrink_to_fit() }
    /// Shrinks the capacity of the underlying [`Vec`] with a lower bound.
    ///
    /// Refer to [`Vec::shrink_to`] for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    /// 
    /// #![feature(shrink_to)]
    /// let mut list = VecList::with_capacity(10);
    /// 
    /// list.extend([1, 2, 3].iter().cloned());
    /// assert_eq!(list.capacity(), 10);
    /// 
    /// list.shrink_to(4);
    /// assert!(list.capacity() >= 4);
    /// 
    /// list.shrink_to(0);
    /// assert!(list.capacity() >= 3);
    /// ```
    #[cfg(feature = "shrink_to",)]
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.nodes.shrink_to(min_capacity);
    }
    /// Moves all elements from `other` to the end of the list.
    ///
    /// This will require a reallocation if there are not enough spaces to store all the
    /// nodes in the underlying [`Vec`].
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    ///
    /// let mut list1 = VecList::new();
    /// list1.push_back('a');
    ///
    /// let mut list2 = VecList::new();
    /// list2.push_back('b');
    /// list2.push_back('c');
    ///
    /// list1.append(&mut list2);
    ///
    /// let mut iter = list1.iter();
    /// assert_eq!(iter.next(), Some(&'a'));
    /// assert_eq!(iter.next(), Some(&'b'));
    /// assert_eq!(iter.next(), Some(&'c'));
    /// assert!(iter.next().is_none());
    ///
    /// assert!(list2.is_empty());
    /// ```
    pub fn append(&mut self, other: &mut Self) {
        //Calculate the number of spaces for the new values.
        let reserve = other.len() - self.cached();

        //Reserve the space.
        self.reserve(reserve);

        //Append the elements.
        for item in other.drain(..) {
            self.push_back(item)
        }
    }
    /// Creates a draining iterator that removes the specified range in the `VecList` and
    /// yields the removed items.
    /// 
    /// Note!!! The element range is removed even if the iterator is only partially
    /// consumed or not consumed at all.
    /// 
    /// # Panics
    /// 
    /// Panics if the starting point is greater than the end point or if the end point is
    /// greater than the length of the `VecList`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut list = VecList::from(vec![1, 2, 3]);
    /// let drained: Vec<_> = list.drain(1..).collect();
    /// assert_eq!(list, &[1]);
    /// assert_eq!(drained, &[2, 3]);
    /// 
    /// // A full range clears the `VecList`.
    /// list.drain(..);
    /// assert_eq!(list, &[]);
    /// ```
    pub fn drain<R>(&mut self, range: R) -> Drain<T>
        where R: RangeBounds<usize> {
        use std::ptr;

        let head = match range.start_bound() {
            Bound::Included(&index) => index,
            Bound::Excluded(&index) => index + 1,
            Bound::Unbounded => 0,
        };
        let tail = match range.end_bound() {
            Bound::Included(&index) => index,
            Bound::Excluded(&index) =>
                if index == 0 { panic!("End point excludes all values") }
                else { index - 1 },
            Bound::Unbounded => Some(self.len()).filter(|i| *i == 0).unwrap_or(0),
        };

        assert!(head <= tail, "Start point was greater than the end point");
        assert!(head == 0 || head < self.len(), "Start point goes off the end of the list");        
        assert!(tail == 0 || tail < self.len(), "End point goes off the end of the list");

        if self.is_empty() { return drain_new(ptr::null(), ptr::null(), 0, self) }

        drain_new(self.get_node(head), self.get_node(tail), tail - head + 1, self)
    }
    /// Provides a forward iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    ///
    /// let mut list: VecList<u32> = VecList::new();
    ///
    /// list.push_back(0);
    /// list.push_back(1);
    /// list.push_back(2);
    ///
    /// let mut iter = list.iter();
    /// assert_eq!(iter.next(), Some(&0));
    /// assert_eq!(iter.next(), Some(&1));
    /// assert_eq!(iter.next(), Some(&2));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<T> {
        unimplemented!()
    }
    /// Provides a forward iterator with mutable references.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    ///
    /// let mut list: VecList<u32> = VecList::new();
    ///
    /// list.push_back(0);
    /// list.push_back(1);
    /// list.push_back(2);
    ///
    /// for element in list.iter_mut() {
    ///     *element += 10;
    /// }
    ///
    /// let mut iter = list.iter();
    /// assert_eq!(iter.next(), Some(&10));
    /// assert_eq!(iter.next(), Some(&11));
    /// assert_eq!(iter.next(), Some(&12));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<T> {
        unimplemented!()
    }
    /// Returns `true` if the `VecList` is empty.
    ///
    /// This operation should compute in O(1) time.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    ///
    /// let mut dl = VecList::new();
    /// assert!(dl.is_empty());
    ///
    /// dl.push_front("foo");
    /// assert!(!dl.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool { self.head.is_none() }
    /// Returns the length of the `VecList`.
    ///
    /// This operation should compute in O(1) time.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    ///
    /// let mut dl = VecList::new();
    ///
    /// dl.push_front(2);
    /// assert_eq!(dl.len(), 1);
    ///
    /// dl.push_front(1);
    /// assert_eq!(dl.len(), 2);
    ///
    /// dl.push_back(3);
    /// assert_eq!(dl.len(), 3);
    /// ```
    #[inline]
    pub fn len(&self) -> usize { self.len }
    /// Removes all elements from the `VecList`.
    ///
    /// This operation should compute in O(n) time.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    ///
    /// let mut dl = VecList::new();
    ///
    /// dl.push_front(2);
    /// dl.push_front(1);
    /// assert_eq!(dl.len(), 2);
    /// assert_eq!(dl.front(), Some(&1));
    ///
    /// dl.clear();
    /// assert_eq!(dl.len(), 0);
    /// assert_eq!(dl.front(), None);
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        //Forget the cache.
        self.cache = None;
        //Remove all existing values.
        self.drain(..);
        //Safe because all of the values have been dropped.
        unsafe { self.nodes.set_len(0) }
        //There are no more elements.
        self.len = 0;
    }
    /// Returns `true` if the `VecList` contains an element equal to the
    /// given value.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    ///
    /// let mut list: VecList<u32> = VecList::new();
    ///
    /// list.push_back(0);
    /// list.push_back(1);
    /// list.push_back(2);
    ///
    /// assert_eq!(list.contains(&0), true);
    /// assert_eq!(list.contains(&10), false);
    /// ```
    pub fn contains(&self, x: &T) -> bool
        where T: PartialEq<T> {
        self.iter().any(|e| e == x)
    }
    /// Provides a reference to the front element, or `None` if the list is
    /// empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    ///
    /// let mut dl = VecList::new();
    /// assert_eq!(dl.front(), None);
    ///
    /// dl.push_front(1);
    /// assert_eq!(dl.front(), Some(&1));
    /// ```
    #[inline]
    pub fn front(&self) -> Option<&T> {
        self.head.map(|head| unsafe { &*(*head).value })
    }
    /// Provides a mutable reference to the front element, or `None` if the list
    /// is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    ///
    /// let mut dl = VecList::new();
    /// assert_eq!(dl.front(), None);
    ///
    /// dl.push_front(1);
    /// assert_eq!(dl.front(), Some(&1));
    ///
    /// match dl.front_mut() {
    ///     None => {},
    ///     Some(x) => *x = 5,
    /// }
    /// assert_eq!(dl.front(), Some(&5));
    /// ```
    #[inline]
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.head.map(|head| unsafe { &mut *(*head).value })
    }
    /// Provides a reference to the back element, or `None` if the list is
    /// empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    ///
    /// let mut dl = VecList::new();
    /// assert_eq!(dl.back(), None);
    ///
    /// dl.push_back(1);
    /// assert_eq!(dl.back(), Some(&1));
    /// ```
    #[inline]
    pub fn back(&self) -> Option<&T> {
        self.tail.map(|tail| unsafe { &*(*tail).value })
    }
    /// Provides a mutable reference to the back element, or `None` if the list
    /// is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    ///
    /// let mut dl = VecList::new();
    /// assert_eq!(dl.back(), None);
    ///
    /// dl.push_back(1);
    /// assert_eq!(dl.back(), Some(&1));
    ///
    /// match dl.back_mut() {
    ///     None => {},
    ///     Some(x) => *x = 5,
    /// }
    /// assert_eq!(dl.back(), Some(&5));
    /// ```
    #[inline]
    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.tail.map(|tail| unsafe { &mut *(*tail).value })
    }
    /// Adds an element first in the list.
    ///
    /// This operation should compute in amortized O(1) time.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    ///
    /// let mut dl = VecList::new();
    ///
    /// dl.push_front(2);
    /// assert_eq!(dl.front().unwrap(), &2);
    ///
    /// dl.push_front(1);
    /// assert_eq!(dl.front().unwrap(), &1);
    /// ```
    pub fn push_front(&mut self, elt: T) {
        use std::mem;

        //Get a new `Node`.
        let mut node: Option<*mut _> = Some(self.new_node(elt));

        //Update the head.
        mem::swap(&mut node, &mut self.head);

        match node {
            //A head existed; append it too the new `Node`.
            Some(node) => unsafe { (*node).append_to(self.head) },
            //No head existed; the head is now also the tail.
            None => self.tail = self.head,
        }

        //Increment the length.
        self.len += 1;
    }
    /// Removes the first element and returns it, or `None` if the list is
    /// empty.
    ///
    /// This operation should compute in amortized O(1) time.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    ///
    /// let mut d = VecList::new();
    /// assert_eq!(d.pop_front(), None);
    ///
    /// d.push_front(1);
    /// d.push_front(3);
    /// assert_eq!(d.pop_front(), Some(3));
    /// assert_eq!(d.pop_front(), Some(1));
    /// assert_eq!(d.pop_front(), None);
    /// ```
    pub fn pop_front(&mut self) -> Option<T> {
        use std::mem::{self, ManuallyDrop,};
        
        //Get the head.
        let head = self.head.take();

        //Update the head value.
        if let Some(head_p) = head {
            //The head existed.
            unsafe {
                //Get the next `Node`.
                self.head = (*head_p).pop_next();
                //Put the head `Node` at the head of the cached `Nodes`.
                (*head_p).append(self.cache);
            }
            //Update the cache.
            self.cache = head;

            //Decrement the length.
            self.len -= 1;
            //If there are no more elements clear the tail.
            if self.len() == 0 { self.tail = None }
        }

        //Return the value.
        //  Safe because:
        //  * `head` was the head of the `ListVec` which must have a value
        //  * `ManuallyDrop` is the same size as `T`.
        head.map(|head| unsafe {
            mem::transmute_copy::<ManuallyDrop<T>, T>(&(*head).value)
        })
    }
    /// Appends an element to the back of a list
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    ///
    /// let mut d = VecList::new();
    /// d.push_back(1);
    /// d.push_back(3);
    /// assert_eq!(3, *d.back().unwrap());
    /// ```
    pub fn push_back(&mut self, elt: T) {
        use std::mem;

        //Get a new `Node`.
        let mut node: Option<*mut _> = Some(self.new_node(elt));

        //Update the tail.
        mem::swap(&mut node, &mut self.tail);

        match node {
            //A tail existed; append the new `Node` too it.
            Some(node) => unsafe { (*node).append(self.tail) },
            //No tail existed; the tail is now also the head.
            None => self.head = self.tail,
        }

        //Increment the length.
        self.len += 1;
    }
    /// Removes the last element from a list and returns it, or `None` if
    /// it is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    ///
    /// let mut d = VecList::new();
    /// assert_eq!(d.pop_back(), None);
    /// d.push_back(1);
    /// d.push_back(3);
    /// assert_eq!(d.pop_back(), Some(3));
    /// ```
    pub fn pop_back(&mut self) -> Option<T> {
        use std::mem::{self, ManuallyDrop,};
        
        //Get the tail.
        let tail = self.tail.take();

        //Update the tail value.
        if let Some(tail_p) = tail {
            //The tail existed.
            unsafe {
                //Get the previous `Node`.
                self.tail = (*tail_p).pop_prev();
                //Put the tail `Node` at the head of the cached `Nodes`.
                (*tail_p).append(self.cache);
            }
            //Update the cache.
            self.cache = tail;

            //Decrement the length.
            self.len -= 1;
            //If there are no more elements clear the head.
            if self.len() == 0 { self.head = None }
        }

        //Return the value.
        //  Safe because:
        //  * `tail` was the tail of the `ListVec` which must have a value
        //  * `ManuallyDrop` is the same size as `T`.
        tail.map(|tail| unsafe {
            mem::transmute_copy::<ManuallyDrop<T>, T>(&(*tail).value)
        })
    }
    /// Splits the list into two at the given index. Returns everything after the given index,
    /// including the index.
    ///
    /// This operation should compute in O(n) time.
    ///
    /// # Panics
    ///
    /// Panics if `at > len`.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_list::VecList;
    ///
    /// let mut d = VecList::new();
    ///
    /// d.push_front(1);
    /// d.push_front(2);
    /// d.push_front(3);
    ///
    /// let mut splitted = d.split_off(2);
    ///
    /// assert_eq!(splitted.pop_front(), Some(1));
    /// assert_eq!(splitted.pop_front(), None);
    /// ```
    pub fn split_off(&mut self, at: usize) -> VecList<T> {
        //Create a new `VecList` with enough spaces.
        let mut list = Self::with_capacity(self.len() - at);

        //Get the elements.
        list.extend(self.drain(at..));
        list
    }
    /// Creates an iterator which uses a closure to determine if an element should be removed.
    ///
    /// If the closure returns true, then the element is removed and yielded.
    /// If the closure returns false, the element will remain in the list and will not be yielded
    /// by the iterator.
    ///
    /// Note that `drain_filter` lets you mutate every element in the filter closure, regardless of
    /// whether you choose to keep or remove it.
    ///
    /// # Examples
    ///
    /// Splitting a list into evens and odds, reusing the original list:
    ///
    /// ```
    /// #![feature(drain_filter)]
    /// use vec_list::VecList;
    ///
    /// let mut numbers: VecList<u32> = VecList::new();
    /// numbers.extend(&[1, 2, 3, 4, 5, 6, 8, 9, 11, 13, 14, 15]);
    ///
    /// let evens = numbers.drain_filter(|x| *x % 2 == 0).collect::<VecList<_>>();
    /// let odds = numbers;
    ///
    /// assert_eq!(evens.into_iter().collect::<Vec<_>>(), vec![2, 4, 6, 8, 14]);
    /// assert_eq!(odds.into_iter().collect::<Vec<_>>(), vec![1, 3, 5, 9, 11, 13, 15]);
    /// ```
    #[cfg(feature = "drain_filter",)]
    pub fn drain_filter<F>(&mut self, filter: F) -> DrainFilter<T, F>
        where F: FnMut(&mut T) -> bool {
        unimplemented!()
    }
}

impl<T, I: Into<T>> Extend<I> for VecList<T> {
    fn extend<Iter>(&mut self, iter: Iter)
        where Iter: IntoIterator<Item = I> {
        for item in iter.into_iter().map(I::into) {
            self.push_back(item)
        }
    }
}

impl<T> Drop for VecList<T> {
    #[inline]
    fn drop(&mut self) {
        //Clean any values.
        self.clear()
    }
}
