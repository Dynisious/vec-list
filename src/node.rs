
use std::{
    mem::ManuallyDrop,
    iter::Iterator,
};

pub struct Node<T> {
    pub prev: Option<*mut Node<T>>,
    pub next: Option<*mut Node<T>>,
    pub value: ManuallyDrop<T>,
}

impl<T> Node<T> {
    /// Appends `next` as the next `Node` after this one.
    /// 
    /// # Unsafe
    /// 
    /// Whether `next`s `prev` value of `self`s `next` value already point to a `Node` is
    /// never checked risking memory leaks.
    pub unsafe fn append(&mut self, next: Option<*mut Self>) {
        debug_assert!(self.next.is_none(), "Memory Leak");

        //Update the prev value of `next`.
        if let Some(next) = next {
            debug_assert!((*next).prev.is_none(), "Memory Leak");

            //Prev exists.
            //Update the prev value.
            (*next).prev = Some(self);
        }

        //Append `next`.
        self.next = next;
    }
    /// Appends `self` as the next `Node` after this `prev`.
    /// 
    /// # Unsafe
    /// 
    /// Whether `prev`s `next` value of `self`s `prev` value already point to a `Node` is
    /// never checked risking memory leaks.
    pub unsafe fn append_to(&mut self, prev: Option<*mut Self>) {
        debug_assert!(self.prev.is_none(), "Memory Leak");

        //Update the prev value of `prev`.
        if let Some(prev) = prev {
            debug_assert!((*prev).next.is_none(), "Memory Leak");

            //Prev exists.
            //Update the prev value.
            (*prev).next = Some(self);
        }

        //Append `prev`.
        self.prev = prev;
    }
    #[inline]
    pub fn pop_next(&mut self) -> Option<*mut Self> { self.next.take() }
    #[inline]
    pub fn pop_prev(&mut self) -> Option<*mut Self> { self.prev.take() }
    fn rec_len(&self, mut acc: usize) -> usize {
        acc += 1;

        match self.next {
            None => acc,
            Some(next) => unsafe { (*next).rec_len(acc) },
        }
    }
    /// Returns the number of `Nodes` in this chain (forwards only).
    #[inline]
    pub fn len(&self) -> usize { self.rec_len(1) }
}
