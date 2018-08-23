
use node::Node;
use std::iter::{Iterator, ExactSizeIterator, DoubleEndedIterator,};

pub fn drain_new<'t, T: 't>(head: *const Node<T>, tail: *const Node<T>,
    len: usize, marker: &'t ::VecList<T>,) -> Drain<'t, T> {
    Drain { head, tail, len, marker, }
}

pub struct Drain<'t, T: 't> {
    head: *const Node<T>,
    tail: *const Node<T>,
    len: usize,
    marker: &'t ::VecList<T>,
}

impl<'t, T> Iterator for Drain<'t, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { (self.len, Some(self.len)) }
}

impl<'t, T> DoubleEndedIterator for Drain<'t, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

impl<'t, T> ExactSizeIterator for Drain<'t, T> {}

impl<'t, T> Drop for Drain<'t, T> {
    fn drop(&mut self) { self.for_each(|_| ()) }
}

#[cfg(feature = "drain_filter",)]
pub struct DrainFilter<'t, T> {
    marker: &'t mut ::VecList<T>,
}

#[cfg(feature = "drain_filter",)]
impl<'t, T> Iterator for DrainFilter<'t, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}
