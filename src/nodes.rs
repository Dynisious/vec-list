
use {VecList,};
use std::mem::ManuallyDrop;

/// A node in a double linked list.
pub struct Node<T,> {
  /// The value inside this [`Node`].
  pub value: ManuallyDrop<T>,
  /// The index of the next [`Node`].
  pub prev: Option<usize>,
  /// The index of the previous [`Node`].
  pub next: Option<usize>,
}

impl<T,> Node<T,> {
  /// Create a new, populated [`Node`].
  /// 
  /// # Params
  /// 
  /// value --- The value to populate the [`Node`] with.  
  #[inline]
  pub fn new(value: T,) -> Self {
    Self { value: ManuallyDrop::new(value,), prev: None, next: None, }
  }
  /// Get the previous [`Node`].
  /// 
  /// # Panics
  /// 
  /// * If there is no previous [`Node`].
  #[inline]
  pub fn prev(&self,) -> usize {
    self.prev.expect("`Node::prev` no previous `Node`")
  }
  /// Get the next [`Node`].
  /// 
  /// # Panics
  /// 
  /// * If there is no next [`Node`].
  #[inline]
  pub fn next(&self,) -> usize {
    self.next.expect("`Node::next` no next `Node`")
  }
  /// Removes the [`Node`] from a doubley linked list.
  /// 
  /// # Params
  /// 
  /// list --- The [`VecList`] this [`Node`] is inside.
  pub fn disconnect(&mut self, list: &mut VecList<T,>,) {
    //Update the next pointer of the previous `Node`.
    if let Some(prev) = self.prev {
      unsafe { &mut *list.node_mut(prev) }.next = self.next;
    }
    //Update the previous pointer of the next `Node` and clear the current `Node`.
    if let Some(next) = self.next.take() {
      unsafe { &mut *list.node_mut(next) }.prev = self.prev.take();
    }
  }
  /// Pushes this [`Node`] into the head of a stack.
  #[inline]
  pub fn stack_push(&mut self, next: usize,) { self.next = Some(next) }
  /// Pops this [`Node`] off the head of a stack.
  #[inline]
  pub fn stack_pop(&mut self,) -> Option<usize> { self.next.take() }
}
