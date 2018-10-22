//! [`vec-list`] is an implementation of a Doubly-Linked-List using an underlying [`Vec`]
//! to store the nodes so as to avoid cache misses during access.
//! 
//! Author --- daniel.bechaz@gmail.com  
//! Last Moddified --- 2018-09-24

#![deny(missing_docs,)]
#![feature(const_fn, const_vec_new, nll, allocator_api, specialization, trusted_len, ptr_offset_from,)]

extern crate imply_option;
extern crate testdrop;

use std::{
  ops::{RangeBounds, Bound, Drop,},
  iter::{FromIterator, Extend, TrustedLen,},
  num::NonZeroUsize,
};

mod raw_vec;
mod nodes;
mod iters;

use self::{nodes::*, raw_vec::*,};
pub use self::iters::Drain;

/// A [`VecList`] is an implementation of a Double Linked List.
/// 
/// A [`VecList`] stores its nodes in an underlying buffer so as to minimise cache
/// misses during access.
/// 
/// The rational behind this is that in most cases ~O(1) appends are good enough when
/// using a [`Vec`] as a list but in the cases where inserts and removes are in the
/// middle of a list a linked list has the advantage; hence backing a linked list with a
/// buffer gives us the best of both worlds when making modifications in the middle of
/// the list.
pub struct VecList<T,> {
  /// The underlying [`RawVec`] of [`Node`]s.
  buf: RawVec<Node<T,>,>,
  /// The number of [`Node`]s in the [`VecList`]s buf.
  node_count: usize,
  /// The indexes to the ends of the linked list and the length of the linked list.
  ends: Option<(NonZeroUsize, usize, usize,)>,
  /// The index to the head of the stack of empty [`Node`]s and the size of the stack.
  empty: Option<(NonZeroUsize, usize,)>,
}

impl<T,> VecList<T,> {
  /// Gets a reference to the [`Node`] at `ptr` in the [`VecList`]s buffer.
  #[inline]
  unsafe fn node(&self, ptr: usize,) -> *const Node<T,> {
    self.buf.ptr().add(ptr,)
  }
  /// Gets a mutable reference to the [`Node`] at `ptr` in the [`VecList`]s buffer.
  #[inline]
  unsafe fn node_mut(&mut self, ptr: usize,) -> *mut Node<T,> {
    self.buf.ptr().add(ptr,)
  }
  /// Appends and links the two [`Node`]s.
  /// 
  /// # Params
  /// 
  /// node --- The pointer to the [`Node`] to be first in the pair.  
  /// next --- The pointer to the [`Node`] to be second in the pair.
  #[inline]
  unsafe fn node_append(&mut self, node: usize, next: usize,) {
    (*self.node_mut(node,)).next = Some(next);
    (*self.node_mut(next,)).prev = Some(node);
  }
}

impl<T,> VecList<T,> {
  /// Get the index to the [`Node`] at `index` in the [`VecList`].
  /// 
  /// # Panics
  /// 
  /// * If `index >= self.len()`
  fn ptr(&self, index: usize,) -> usize {
    /// Get the pointer to the [`Node`] `steps` steps backwards from `link`.
    /// 
    /// # Params
    /// 
    /// list --- The [`VecList`] to get [`Node`]s from.  
    /// link --- The [`Node`] to step from.  
    /// steps --- The number of steps backwards to take from `link`.  
    #[inline]
    fn backwards<T,>(list: &VecList<T,>, link: usize, steps: usize,) -> usize {
      if steps == 0 { link }
      else { backwards(list, unsafe { (*list.buf.ptr().add(link)).prev() }, steps - 1,) }
    }
    /// Get the pointer to the [`Node`] `steps` steps frowards from `link`.
    /// 
    /// # Params
    /// 
    /// list --- The [`VecList`] to get [`Node`]s from.  
    /// link --- The [`Node`] to step from.  
    /// steps --- The number of steps frowards to take from `link`.  
    #[inline]
    fn forwards<T,>(list: &VecList<T,>, link: usize, steps: usize,) -> usize {
      if steps == 0 { link }
      else { forwards(list, unsafe { (*list.buf.ptr().add(link)).next() }, steps - 1,) }
    }

    //Validate index.
    assert!(index < self.len(), "`VecList::ptr` index out of range",);

    //Calculate how many steps need to be taken from the end.
    let back_index = self.len() - index - 1;

    //Take the shortest number of steps.
    match self.ends.expect("`VecList::ptr` called on an empty `VecList`",) {
      //Go from the back.
      (_, _, end,) if back_index < index => backwards(self, end, back_index,),
      //Go from the front.
      (_, start, _,) => forwards(self, start, index,),
    }
  }
  /// Allocate a new [`Node`] populated with `value`.
  fn alloc_node(&mut self, value: T,) -> usize {
    match self.empty {
      //Allocate a new [`Node`] in the buffer.
      None => {
        let node = self.node_count;

        self.node_count += 1;
        self.reserve(1,);
        unsafe { *self.node_mut(node,) = Node::new(value,); }

        node
      },
      //Allocate a [`Node`] from the empty stack.
      Some((len, empty,)) => { unsafe {
        self.empty = (*self.node_mut(empty,)).stack_pop()
          .map(|empty| (NonZeroUsize::new_unchecked(len.get() - 1,), empty,));
        
        *self.node_mut(empty,) = Node::new(value,);

        empty
      } }
    }
  }
  /// Deallocate the [`Node`] at `ptr` in the buffer.
  /// 
  /// # Params
  /// 
  /// ptr --- The index of the [`Node`] in `buf`.
  fn dealloc_node(&mut self, ptr: usize,) -> T {
    let node = unsafe { &mut *self.buf.ptr().add(ptr) };

    node.disconnect(self,);
    self.empty = match self.empty {
      None => Some((unsafe { NonZeroUsize::new_unchecked(1,) }, ptr,)),
      Some((len, empty,)) => {
        node.stack_push(empty,);
        
        Some((unsafe { NonZeroUsize::new_unchecked(len.get() + 1,) }, ptr,))
      },
    };

    unsafe { (&*node.value as *const T).read() }
  }
}

impl<T,> VecList<T,> {
  /// Forwards the call to `VecList::with_capacity(0)`.
  #[inline]
  pub fn new() -> Self { Self::with_capacity(0,) }
  /// Constructs a new empty [`VecList`] with space for `capacity` nodes in the
  /// underlying buffer.
  #[inline]
  pub fn with_capacity(capacity: usize,) -> Self {
    Self { buf: RawVec::with_capacity(capacity), node_count: 0, ends: None, empty: None, }
  }
  /// Returns the capacity of the underlying buffer.
  #[inline]
  pub fn capacity(&self,) -> usize { self.buf.cap() }
  /// Reserves enough capacity for exactly `additional` more elements to be inserted into
  /// the [`VecList`].
  /// 
  /// # Panics
  /// 
  /// * If the new capacity overflows usize.  
  #[inline]
  pub fn reserve_exact(&mut self, mut additional: usize,) {
    //Remove the empty `Node`s count from additional.
    if let Some((empty, _,)) = self.empty {
      additional = additional.saturating_sub(empty.get(),)
    }

    self.buf.reserve_exact(self.node_count, additional,)
  }
  /// Reserves enough capacity for at least `additional` more elements to be inserted
  /// into the [`VecList`].
  #[inline]
  pub fn reserve(&mut self, mut additional: usize,) {
    //Remove the empty `Node`s count from additional.
    if let Some((empty, _,)) = self.empty {
      additional = additional.saturating_sub(empty.get(),)
    }

    self.buf.reserve(self.node_count, additional,)
  }
  /// Returns the number of elements in this [`VecList`].
  #[inline]
  pub fn len(&self,) -> usize {
    self.ends.map_or(0, |(len, _, _,)| len.get(),)
  }
  /// Clears all values from this [`VecList`].
  #[inline]
  pub fn clear(&mut self,) { self.drain(..); }
}

impl<T,> VecList<T,> {
  /// Pushes `value` onto the front of this [`VecList`].
  pub fn push_front(&mut self, value: T,) {
    let node = self.alloc_node(value,);

    self.ends = unsafe { match self.ends {
      None => Some((NonZeroUsize::new_unchecked(1,), node, node,)),
      Some((len, head, tail,)) => {
        self.node_append(node, head,);

        Some((NonZeroUsize::new_unchecked(len.get() + 1,), node, tail,))
      },
    } };
  }
  /// Pops a value off the front of this [`VecList`].
  #[inline]
  pub fn pop_front(&mut self,) -> Option<T> {
    self.ends.take().map(|(len, head, tail,)| {
      let head_node = unsafe { &mut *self.node_mut(head,) };

      self.ends = if head == tail { None }
        else { Some((unsafe { NonZeroUsize::new_unchecked(len.get() - 1,) }, head_node.next(), tail,)) };
      head_node.disconnect(self,);

      self.dealloc_node(head,)
    })
  }
  /// Pushes `value` onto the back of this [`VecList`].
  pub fn push_back(&mut self, value: T,) {
    let node = self.alloc_node(value,);

    self.ends = unsafe { match self.ends {
      None => Some((NonZeroUsize::new_unchecked(1,), node, node,)),
      Some((len, head, tail,)) => {
        self.node_append(tail, node,);

        Some((NonZeroUsize::new_unchecked(len.get() + 1,), head, node,))
      },
    } };
  }
  /// Pops a value off the back of this [`VecList`].
  #[inline]
  pub fn pop_back(&mut self,) -> Option<T> {
    self.ends.take().map(|(len, head, tail,)| {
      let tail_node = unsafe { &mut *self.node_mut(tail,) };

      self.ends = if head == tail { None }
        else { Some((unsafe { NonZeroUsize::new_unchecked(len.get() - 1,) }, head, tail_node.prev(),)) };
      tail_node.disconnect(self,);

      self.dealloc_node(tail,)
    })
  }
}

impl<'t, T: 't,> VecList<T,> {
  /// Removes the elements in `range` from the [`VecList`] and returns them as an
  /// iterator.
  /// 
  /// # Params
  /// 
  /// range --- The range of indexes to remove.
  /// 
  /// # Panics
  /// 
  /// * If `range.end >= self.len()`.
  pub fn drain<R,>(&'t mut self, range: R,) -> Drain<'t, T,>
    where R: RangeBounds<usize>, {
    use imply_option::ImplyOption;

    //Get the starting index.
    let start = match range.start_bound() {
      Bound::Excluded(&start,) => start + 1,
      Bound::Included(&start,) => start,
      Bound::Unbounded => 0,
    };
    //Get the ending index.
    let end = match range.end_bound() {
      Bound::Excluded(&end,) => Some(end.saturating_sub(1)),
      Bound::Included(&end,) => Some(end),
      Bound::Unbounded => self.len().checked_sub(1),
    };
    //Validate the ends.
    let ends = match end {
      Some(end) => if end < self.len() { (start <= end).then_do(|| (self.ptr(start), self.ptr(end),)) }
        else { panic!("The end of the range must be less than the length of the `VecList`") },
      None => None,
    };

    iters::drain(self, ends,)
  }
}

impl<T,> FromIterator<T> for VecList<T,> {
  #[inline]
  fn from_iter<I,>(iter: I,) -> Self
    where I: IntoIterator<Item = T>, {
    let mut list = VecList::new();

    list.extend(iter,); list
  }
}

impl<T,> Extend<T> for VecList<T,> {
  #[inline]
  fn extend<I,>(&mut self, iter: I,)
    where I: IntoIterator<Item = T>, {
    self.spec_extend(iter.into_iter(),)
  }
}

impl<'t, T: 't + Clone,> FromIterator<&'t T> for VecList<T,> {
  #[inline]
  fn from_iter<I,>(iter: I,) -> Self
    where I: IntoIterator<Item = &'t T>, {
    let mut list = VecList::new();

    list.extend(iter,); list
  }
}

impl<'t, T: 't + Clone,> Extend<&'t T> for VecList<T,> {
  #[inline]
  fn extend<I,>(&mut self, iter: I,)
    where I: IntoIterator<Item = &'t T>, {
    self.spec_extend(iter.into_iter().cloned(),)
  }
}

trait SpecExtend<A, I,> {
  fn spec_extend(&mut self, iter: I,)
    where I: IntoIterator<Item = A>;
}

impl<T, A, I,> SpecExtend<A, I,> for VecList<T,>
  where A: Into<T>, I: Iterator<Item = A>, {
  #[inline]
  default fn spec_extend(&mut self, iter: I,) {
    for a in iter { self.push_back(a.into(),) }
  }
}

impl<T, A, I,> SpecExtend<A, I,> for VecList<T,>
  where A: Into<T>, I: TrustedLen<Item = A>, {
  fn spec_extend(&mut self, iter: I,) {
    //Reserve additional space for the values.
    if let Some(additional) = iter.size_hint().1 { self.reserve(additional,) }

    for a in iter { self.push_back(a.into(),) }
  }
}

impl<T,> Default for VecList<T,> {
  #[inline]
  fn default() -> Self { Self::new() }
}

impl<T,> Drop for VecList<T,> {
  #[inline]
  fn drop(&mut self,) { self.clear() }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_vec_list() {
    let list = VecList::<i32>::new();

    assert_eq!(list.capacity(), 0, "`VecList::new()` incorrect capacity",);
    assert_eq!(list.len(), 0, "`VecList::new()` created with incorrect length",);

    let mut list = VecList::<i32>::with_capacity(2,);

    assert_eq!(list.capacity(), 2, "`VecList::with_capacity(2,)` created with incorrect capacity",);
    assert_eq!(list.len(), 0, "`VecList::with_capacity(2,)` created with incorrect length",);

    list.reserve(1,);
    assert_eq!(list.capacity(), 2, "`VecList::reserve(1,)` incorrect capacity",);
    list.reserve(2,);
    assert_eq!(list.capacity(), 2, "`VecList::reserve(2,)` incorrect capacity",);

    list.reserve(3,);
    assert_eq!(list.capacity(), 4, "`VecList::reserve(3,)` incorrect capacity",);

    list.reserve_exact(6,);
    assert_eq!(list.capacity(), 10, "`VecList::reserve_exact(6,)` incorrect capacity",);

    for i in 1..3 { list.push_back(i,); }
    list.push_front(0,);
    assert_eq!(list.len(), 3, "`VecList::push_(front/back)` did not increment the length",);

    assert_eq!(list.pop_back(), Some(2), "`VecList::pop_back` returned incorrect result",);
    assert_eq!(list.len(), 2, "`VecList::pop_front` did not decrement the length.",);

    assert_eq!(list.pop_front(), Some(0), "`VecList::pop_front` returned incorrect result",);
    assert_eq!(list.len(), 1, "`VecList::pop_back` did not decrement the length.",);

    list.clear();
    assert_eq!(list.capacity(), 10, "`VecList::clear` changed the capacity",);
    assert_eq!(list.len(), 0, "`VecList::clear` did not empty the list",);
  }
}
