
use ::*;
use std::{
    cmp::{PartialEq, PartialOrd, Ord, Ordering,},
    ops::Deref, convert::AsRef, borrow::Borrow,
};

macro_rules! make_ref {
    ($ptr:expr) => (unsafe { &*$ptr });
}

#[inline]
pub const fn new_view<'i, 't: 'i, T: 't,>(list: &'i VecList<'t, T,>, view: &'i Node<'t, T,>,) -> View<'i, T,> {
    View { list, view, }
}

#[derive(Eq, Clone, Debug,)]
pub struct View<'t, T: 't,> {
    list: *const VecList<'t, T,>,
    view: *const Node<'t, T,>,
}

impl<'t, T: 't,> View<'t, T,> {
    pub fn next(&self) -> Option<Self,> {
        make_ref!(self.view).next.map(
            |next| new_view(make_ref!(self.list), &make_ref!(self.list).nodes[next],)
        )
    }
    pub fn prev(&self) -> Option<Self,> {
        make_ref!(self.view).prev.map(
            |prev| new_view(make_ref!(self.list), &make_ref!(self.list).nodes[prev],)
        )
    }
    #[inline]
    pub fn as_ref(self) -> &'t T { &make_ref!(self.view).value }
}

impl<'t, T: 't,> Deref for View<'t, T,> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target { &make_ref!(self.view).value }
}

impl<'t, T: 't,> AsRef<T> for View<'t, T,> {
    #[inline]
    fn as_ref(&self) -> &T { &make_ref!(self.view).value }
}

impl<'t, T: 't,> Borrow<T> for View<'t, T,> {
    #[inline]
    fn borrow(&self) -> &T { &make_ref!(self.view).value }
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
