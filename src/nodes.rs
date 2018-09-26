
pub use std::{
    mem::ManuallyDrop,
    marker::PhantomData,
};

/// A `Node` is a single `Node` in a doubly linked list.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug,)]
pub struct Node<'t, T: 't,> {
    /// The value stored in this `Node`.
    pub value: ManuallyDrop<T,>,
    /// The pointer to the previous `Node`.
    pub prev: Option<usize,>,
    /// The pointer to the next `Node`.
    pub next: Option<usize,>,
    /// A marker to consume the lifetime parameter.
    _marker: PhantomData<&'t ()>,
}

impl<'t, T: 't,> Node<'t, T,> {
    /// Create a new `Node` populated with the value.
    #[inline]
    pub fn new(value: T,) -> Self {
        Self {
            value: ManuallyDrop::new(value),
            prev: None, next: None,
            _marker: PhantomData::default(),
        }
    }
    /// A way to unsafely move the value out of the `Node`.
    #[inline]
    pub unsafe fn move_value(&self,) -> T { (&*self.value as *const T).read() }
}
