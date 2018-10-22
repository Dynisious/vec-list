
use {VecList, NonZeroUsize,};
use std::{iter::*, ops::Drop,};

/// Creates a new [`Drain`] iterator.
/// 
/// # Params
/// 
/// list --- The [`VecList`] being iterated over.  
/// ends --- The ends of the range being iterated over.  
pub fn drain<'t, T: 't,>(list: &'t mut VecList<T,>, ends: Option<(usize, usize,)>,) -> Drain<'t, T,> {
  Drain { list, ends, }
}

/// An iterator which removes values from a range in a [`VecList`].
/// 
/// The values in the range will be removed even if they are not iterated over.
pub struct Drain<'t, T: 't,> {
  /// The [`VecList`] being drained.
  list: &'t mut VecList<T,>,
  /// The ends of the range being drained over.
  ends: Option<(usize, usize,)>,
}

impl<'t, T: 't,> Iterator for Drain<'t, T,> {
  type Item = T;

  #[inline]
  fn next(&mut self,) -> Option<Self::Item> {
    self.ends.map(|(front, back,)| { unsafe {
      use imply_option::ImplyOption;
      use std::hint;

      //The `Node` of the `front` pointer.
      let front_node = &*self.list.node(front,);

      //Update the ends of the range being iterated over.
      self.ends = (front != back).then_do(|| (front_node.next(), back,),);
      //Update the ends of the `VecList`.
      match self.list.ends {
        //Iteration would not be happening if the `VecList` was empty.
        None => hint::unreachable_unchecked(),
        //If there is only a single node, the `VecList` is now empty.
        Some((len, head, tail,)) => {
          let nlen = NonZeroUsize::new_unchecked(len.get() - 1,);
          
          if head == tail { self.list.ends = None }
          //If the head has been drained, update the head pointer.
          else if head == front { self.list.ends = Some((nlen, front_node.next(), tail,)) }
          //If the tail has been drained, update the tail pointer.
          else if tail == front { self.list.ends = Some((nlen, head, front_node.prev(),)) }
        },
      }

      //Deallocate the `Node`.
      self.list.dealloc_node(front,)
    }})
  }
  fn size_hint(&self) -> (usize, Option<usize>,) {
    //If there are ends there is at least one more value.
    if self.ends.is_some() { (1, None,) }
    //Else there are no values.
    else { (0, Some(0),) }
  }
}

impl<'t, T: 't,> DoubleEndedIterator for Drain<'t, T,> {
  #[inline]
  fn next_back(&mut self,) -> Option<Self::Item> {
    self.ends.map(|(front, back,)| { unsafe {
      use imply_option::ImplyOption;
      use std::hint;

      //The `Node` of the `back` pointer.
      let back_node = &*self.list.node(back,);

      //Update the ends of the range being iterated over.
      self.ends = (front != back).then_do(|| (front, back_node.prev(),),);
      //Update the ends of the `VecList`.
      match self.list.ends {
        //Iteration would not be happening if the `VecList` was empty.
        None => hint::unreachable_unchecked(),
        //If there is only a single node, the `VecList` is now empty.
        Some((len, head, tail,)) => {
          let nlen = NonZeroUsize::new_unchecked(len.get() - 1,);
          
          if head == tail { self.list.ends = None }
          //If the head has been drained, update the head pointer.
          else if head == back { self.list.ends = Some((nlen, back_node.next(), tail,)) }
          //If the tail has been drained, update the tail pointer.
          else if tail == back { self.list.ends = Some((nlen, head, back_node.prev(),)) }
        },
      }

      //Deallocate the `Node`.
      self.list.dealloc_node(back,)
    }})
  }
}

impl<'t, T: 't,> Drop for Drain<'t, T,> {
  #[inline]
  fn drop(&mut self,) { self.for_each(|_| ()) }
}

#[cfg(test)]
mod tests {
  use super::*;
  use testdrop::*;  

  #[test]
  fn test_drain() {
    let test_drop = TestDrop::new();
    let mut list = VecList::<Item>::with_capacity(3,);
    let mut ids = Vec::with_capacity(3,);

    for _ in 0..5 {
      let (id, item,) = test_drop.new_item();

      list.push_back(item,);
      ids.push(id,);
    }
    
    list.drain(1..4);
    
    test_drop.assert_no_drop(ids[0]);
    test_drop.assert_drop(ids[1]);
    test_drop.assert_drop(ids[2]);
    test_drop.assert_drop(ids[3]);
    test_drop.assert_no_drop(ids[4]);
  }
}
