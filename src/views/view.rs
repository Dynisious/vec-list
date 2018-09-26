
use ::*;
use std::{
    cmp::{PartialEq, PartialOrd, Ord, Ordering,},
    ops::Deref,
    convert::AsRef,
    borrow::Borrow,
};

macro_rules! list {
    ($ptr:expr) => (unsafe { &*$ptr });
}

/// Creates a new `View` value from parts.
/// 
/// # Params
/// 
/// list --- The `VecList` to iterate over.  
/// view --- The index in the `VecList` to view.
#[inline]
pub fn new_view<'i, 't: 'i, T: 't,>(list: &'i VecList<'t, T,>, view: usize,) -> View<'i, T,> {
    View { list, view, }
}

#[derive(Eq, Debug,)]
pub struct View<'t, T: 't,> {
    /// The `VecList` to iterate over.
    list: *const VecList<'t, T,>,
    /// The postion to view at.
    view: usize,
}

impl<'t, T: 't,> View<'t, T,> {
    /// Creates a `View` at the next position in the `VecList`.
    pub fn next(&self) -> Option<Self,> {
        let list = list!(self.list);

        list.nodes[self.view].next
            .map(|next| new_view(list, next,))
    }
    /// Creates a `View` at the previous position in the `VecList`.
    pub fn prev(&self) -> Option<Self,> {
        let list = list!(self.list);

        list.nodes[self.view].prev
            .map(|prev| new_view(list, prev,))
    }
    /// Convert this `View` into a reference to the value.
    #[inline]
    pub fn as_ref(self) -> &'t T {
        &list!(self.list).nodes[self.view].value
    }
}

impl<'t, T: 't,> Deref for View<'t, T,> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &list!(self.list).nodes[self.view].value
    }
}

impl<'t, T: 't,> AsRef<T> for View<'t, T,> {
    #[inline]
    fn as_ref(&self) -> &T { &list!(self.list).nodes[self.view].value }
}

impl<'t, T: 't,> Borrow<T> for View<'t, T,> {
    #[inline]
    fn borrow(&self) -> &T { &list!(self.list).nodes[self.view].value }
}

impl<'t, T: 't + PartialEq,> PartialEq for View<'t, T,> {
    #[inline]
    fn eq(&self, rhs: &Self) -> bool { T::eq(self, rhs,) }
}

impl<'i, 't, T: 't + PartialEq,> PartialEq<&'i T,> for View<'t, T,> {
    #[inline]
    fn eq(&self, rhs: &&T) -> bool { T::eq(self, rhs,) }
}

impl<'t, T: 't + PartialOrd,> PartialOrd for View<'t, T,> {
    #[inline]
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering,> { T::partial_cmp(self, rhs,) }
}

impl<'i, 't, T: 't + PartialOrd,> PartialOrd<&'i T,> for View<'t, T,> {
    #[inline]
    fn partial_cmp(&self, rhs: &&T) -> Option<Ordering,> { T::partial_cmp(self, rhs,) }
}

impl<'t, T: 't + Ord,> Ord for View<'t, T,> {
    #[inline]
    fn cmp(&self, rhs: &Self) -> Ordering { T::cmp(self, rhs,) }
}
