
use node::Node;
use std::{
    marker::PhantomData,
    iter::{Iterator,},
};

pub struct Iter<'t, T> {
    head: *const Node<T>,
    tail: *const Node<T>,
    len: usize,
    marker: PhantomData<&'t ()>,
}

impl<'t, T: 't> Iterator for Iter<'t, T> {
    type Item = &'t T;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

pub struct IterMut<'t, T> {
    head: *mut Node<T>,
    tail: *mut Node<T>,
    len: usize,
    marker: PhantomData<&'t ()>,
}
