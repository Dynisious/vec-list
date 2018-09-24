
pub use std::{
    mem::ManuallyDrop,
    marker::PhantomData,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug,)]
pub struct Node<'t, T: 't,> {
    pub value: ManuallyDrop<T,>,
    pub prev: Option<usize,>,
    pub next: Option<usize,>,
    _marker: PhantomData<&'t ()>,
}

impl<'t, T: 't,> Node<'t, T,> {
    #[inline]
    pub fn new(value: T,) -> Self {
        Self {
            value: ManuallyDrop::new(value),
            prev: None, next: None,
            _marker: PhantomData::default(),
        }
    }
    #[inline]
    pub unsafe fn value(&self,) -> T { (&*self.value as *const T).read() }
}
