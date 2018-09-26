
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

/// Creates a new `ViewMut` value from parts.
/// 
/// # Params
/// 
/// list --- The `VecList` to iterate over.  
/// view --- The index in the `VecList` to view.
#[inline]
pub fn new_view_mut<'i, 't: 'i, T: 't,>(list: &'i mut VecList<'t, T,>, view: usize,) -> ViewMut<'i, T,> {
    ViewMut { list, view, }
}

#[derive(Eq, Debug,)]
pub struct ViewMut<'t, T: 't,> {
    /// The `VecList` to iterate over.
    list: *mut VecList<'t, T,>,
    /// The postion to view at.
    view: usize,
}

impl<'t, T: 't,> ViewMut<'t, T,> {
    /// Advance the `ViewMut` to the next postion.
    /// 
    /// Returns `Ok` if the view was advanced.  
    /// Returns `Err` if the view couldn't be advanced.
    pub fn next(&mut self) -> Result<&mut Self, &mut Self> {
        match list!(self.list).nodes[self.view].next {
            Some(view) => { self.view = view; Ok(self) },
            _ => Err(self),
        }
    }
    /// Advance the `ViewMut` to the previous postion.
    /// 
    /// Returns `Ok` if the view was advanced.  
    /// Returns `Err` if the view couldn't be advanced.
    pub fn prev(&mut self) -> Result<&mut Self, &mut Self> {
        match list!(self.list).nodes[self.view].prev {
            Some(view) => { self.view = view; Ok(self) },
            _ => Err(self),
        }
    }
    /// Push a value to before the position of the `ViewMut` in the `VecList`.
    /// 
    /// # Params
    /// 
    /// value --- The value to push before the `ViewMut`.
    pub fn insert_before(&mut self, value: T,) {
        use std::mem;

        let list = list!(self.list);
        //Get a new `Node`.
        let new_node = list.new_node(value,);

        //Make the new `Node` point to this `Node`.
        list.nodes[new_node].next = Some(self.view);
        //Make the new `Node` point to this `Node`s previous value.
        //  Also make this `Node` point to the new `Node` as the previous value.
        list.nodes[new_node].prev = mem::replace(&mut list.nodes[self.view].prev, Some(new_node));
        //If the new `Node` has a previous value, make it point to the new `Node` as its next value.
        list.nodes[new_node].prev
            .map(|prev| list.nodes[prev].next = Some(new_node));
        
        //Update the head pointer of the `VecList` if necessary.
        if self.view == list.ends.expect("insert_before: 1").0 {
            list.ends.as_mut().expect("insert_before: 2").0 = new_node;
        }
    }
    /// Remove the value from before the position of the `ViewMut` in the `VecList` if
    /// it exists.
    pub fn pop_before(&mut self,) -> Option<T> {
        let list = list!(self.list);

        list.nodes[self.view].prev
        .map(|node| {
            //If the `Node` before the `ViewMut` is the head `Node`, this `ViewMut` is
            //  now the head node.
            if node == list.ends.expect("pop_before: 1").0 {
                list.ends.expect("pop_before: 2").0 = self.view
            }

            //Remove the `Node`.
            list.remove_node(node,)
        })
    }
    /// Push a value to after the position of the `ViewMut` in the `VecList`.
    /// 
    /// # Params
    /// 
    /// value --- The value to push after the `ViewMut`.
    pub fn insert_after(&mut self, value: T,) {
        use std::mem;

        let list = list!(self.list);
        //Get a new `Node`.
        let new_node = list.new_node(value,);

        //Make the new `Node` point to this `Node`.
        list.nodes[new_node].prev = Some(self.view);
        //Make the new `Node` point to this `Node`s next value.
        //  Also make this `Node` point to the new `Node` as the next value.
        list.nodes[new_node].next = mem::replace(&mut list.nodes[self.view].next, Some(new_node));
        //If the new `Node` has a next value, make it point to the new `Node` as its prev value.
        list.nodes[new_node].next
            .map(|next| list.nodes[next].prev = Some(new_node));
        
        //Update the tail pointer of the `VecList` if necessary.
        if self.view == list.ends.expect("insert_before: 1").1 {
            list.ends.as_mut().expect("insert_before: 2").1 = new_node;
        }
    }
    /// Remove the value from after the position of the `ViewMut` in the `VecList` if
    /// it exists.
    pub fn pop_after(&mut self,) -> Option<T> {
        let list = list!(self.list);

        list.nodes[self.view].next
        .map(|node| {
            //If the `Node` before the `ViewMut` is the tail `Node`, this `ViewMut` is
            //  now the tail node.
            if node == list.ends.expect("pop_after: 1").1 {
                list.ends.expect("pop_after: 2").1 = self.view
            }

            //Remove the `Node`.
            list.remove_node(node,)
        })
    }
    /// Delete this `Node` from the `VecList` and return its value.
    pub fn delete(self,) -> T {
        let list = list!(self.list);

        //Update the list ends if necessary.
        if let Some((head, tail,)) = list.ends {
            //If the `ViewMut` is the head pointer, update it.
            if head == self.view {
                //Point the head at the next `Node`.
                list.ends = list.nodes[self.view].next
                    .map(|head| (head, tail,))
            //If the `ViewMut` is the tail pointer update it.
            } else if tail == self.view {
                //Point the tail at the previous `Node`.
                list.ends = list.nodes[self.view].prev
                    .map(|tail| (head, tail,))
            }
        }

        //Remove this `Node`.
        list.remove_node(self.view)
    }
    /// Convert this `ViewMut` into a mutable reference to the value.
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
