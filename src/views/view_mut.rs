
use ::*;
use std::{
    cmp::{PartialEq, PartialOrd, Ord, Ordering,},
    ops::{Deref, DerefMut,},
    convert::{AsRef, AsMut,},
    borrow::{Borrow, BorrowMut,},
};

macro_rules! list {
    ($ptr:expr) => (unsafe { &mut *$ptr });
}

#[inline]
pub fn new_view_mut<'i, 't: 'i, T: 't,>(list: &'i mut VecList<'t, T,>, view: usize,) -> ViewMut<'i, T,> {
    ViewMut { list, view, }
}

#[derive(Eq, Debug,)]
pub struct ViewMut<'t, T: 't,> {
    list: *mut VecList<'t, T,>,
    view: usize,
}

impl<'t, T: 't,> ViewMut<'t, T,> {
    pub fn next(&mut self) -> Result<&mut Self, &mut Self> {
        match list!(self.list).nodes[self.view].next {
            Some(view) => { self.view = view; Ok(self) },
            _ => Err(self),
        }
    }
    pub fn prev(&mut self) -> Result<&mut Self, &mut Self> {
        match list!(self.list).nodes[self.view].prev {
            Some(view) => { self.view = view; Ok(self) },
            _ => Err(self),
        }
    }
    pub fn push_before(&mut self, value: T,) {
        use std::mem;

        let list = list!(self.list);
        let new_node = list.new_node(value,);

        list.nodes[new_node].prev = Some(self.view);
        list.nodes[new_node].next = mem::replace(&mut list.nodes[self.view].next, Some(new_node));
        if self.view == list.ends.expect("push_before: 1").1 {
            list.ends.as_mut().expect("push_before: 2").1 = new_node;
        }
    }
    pub fn pop_before(&mut self,) -> Option<T> {
        let list = list!(self.list);

        list.nodes[self.view].prev
        .map(|node| {
            if node == list.ends.expect("pop_before: 1").0 {
                list.ends.expect("pop_before: 2").0 = self.view
            }

            list.shelf_node(node,)
        })
    }
    pub fn push_after(&mut self, value: T,) {
        use std::mem;

        let list = list!(self.list);
        let new_node = list.new_node(value,);

        list.nodes[new_node].next = Some(self.view);
        list.nodes[new_node].prev = mem::replace(&mut list.nodes[self.view].prev, Some(new_node));
        if self.view == list.ends.expect("push_after: 1").0 {
            list.ends.as_mut().expect("push_after: 2").0 = new_node;
        }
    }
    pub fn pop_after(&mut self,) -> Option<T> {
        let list = list!(self.list);

        list.nodes[self.view].next
        .map(|node| {
            if node == list.ends.expect("pop_after: 1").1 {
                list.ends.expect("pop_after: 2").1 = self.view
            }
            
            list.shelf_node(node,)
        })
    }
    pub fn delete(self,) -> T {
        use imply_option::*;

        let list = list!(self.list);

        if let Some((head, tail,)) = list.ends {
            if head == self.view {
                list.ends = list.nodes[self.view].next
                    .and_then(|head| (head != tail).then((head, tail,)))
            } else if tail == self.view {
                list.ends = list.nodes[self.view].next
                    .map(|tail| (head, tail,))
            }
        }

        list.shelf_node(self.view)
    }
    #[inline]
    pub fn as_mut(self) -> &'t mut T {
        &mut list!(self.list).nodes[self.view].value
    }
}

impl<'t, T: 't,> Deref for ViewMut<'t, T,> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &list!(self.list).nodes[self.view].value
    }
}

impl<'t, T: 't,> DerefMut for ViewMut<'t, T,> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut list!(self.list).nodes[self.view].value
    }
}

impl<'t, T: 't,> AsRef<T> for ViewMut<'t, T,> {
    #[inline]
    fn as_ref(&self) -> &T { &list!(self.list).nodes[self.view].value }
}

impl<'t, T: 't,> AsMut<T> for ViewMut<'t, T,> {
    #[inline]
    fn as_mut(&mut self) -> &mut T { &mut list!(self.list).nodes[self.view].value }
}

impl<'t, T: 't,> Borrow<T> for ViewMut<'t, T,> {
    #[inline]
    fn borrow(&self) -> &T { &list!(self.list).nodes[self.view].value }
}

impl<'t, T: 't,> BorrowMut<T> for ViewMut<'t, T,> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut T { &mut list!(self.list).nodes[self.view].value }
}

impl<'t, T: 't + PartialEq,> PartialEq for ViewMut<'t, T,> {
    #[inline]
    fn eq(&self, rhs: &Self) -> bool { T::eq(self, rhs,) }
}

impl<'i, 't, T: 't + PartialEq,> PartialEq<&'i T,> for ViewMut<'t, T,> {
    #[inline]
    fn eq(&self, rhs: &&T) -> bool { T::eq(self, rhs,) }
}

impl<'t, T: 't + PartialOrd,> PartialOrd for ViewMut<'t, T,> {
    #[inline]
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering,> { T::partial_cmp(self, rhs,) }
}

impl<'i, 't, T: 't + PartialOrd,> PartialOrd<&'i T,> for ViewMut<'t, T,> {
    #[inline]
    fn partial_cmp(&self, rhs: &&T) -> Option<Ordering,> { T::partial_cmp(self, rhs,) }
}

impl<'t, T: 't + Ord,> Ord for ViewMut<'t, T,> {
    #[inline]
    fn cmp(&self, rhs: &Self) -> Ordering { T::cmp(self, rhs,) }
}
