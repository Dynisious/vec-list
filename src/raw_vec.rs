
use std::{mem, ptr::{self, NonNull,}, alloc::{Global, Alloc,},};

/// Generates an error message for an allocation error.
macro_rules! alloc_err {
  ($fn:tt, $e:ident,) => {
    format!("{}:{}", concat!("`", $fn, "` error allocating buffer: ", file!(), ":", line!(), ":", column!(),), $e)
  };
}

/// A heap allocated buffer of `T` aligned slots.
pub struct RawVec<T,> {
  /// The heap buffer.
  buf: *mut T,
  /// The capacity of the buffer.
  cap: usize,
}

impl<T,> RawVec<T,> {
  /// Allocates a new [`RawVec`] with the passed capacity.
  /// 
  /// # Notes
  /// 
  /// * If `T` is zero sized; the allocation is a null pointer with a capacity of usize::MAX_SIZE.
  /// * If `cap` is `0`; the allocation is a null pointer.
  /// 
  /// # Params
  /// 
  /// cap --- The capacity of the new buffer.
  /// 
  /// # Panics
  /// 
  /// * If the allocation could not be made.
  #[inline]
  pub fn with_capacity(mut cap: usize,) -> Self {
    //Create the buffer.
    let buf = if mem::size_of::<T>() == 0 { cap = usize::max_value(); ptr::null_mut() }
      else if cap == 0 { ptr::null_mut() }
      //Allocate the array.
      else { match Global.alloc_array::<T>(cap,) {
        Ok(buf) => buf,
        Err(e) => panic!(alloc_err!("RawVec::with_capacity", e,)),
      }.as_ptr() };

    Self { buf, cap, }
  }
  /// Returns the capacity of the allocated buffer.
  #[inline]
  pub const fn cap(&self,) -> usize { self.cap }
  /// Reserves enough space for exactly `additional` more values.
  /// 
  /// # Params
  /// 
  /// used_cap --- The used capacity of the buffer.  
  /// additional --- The additional spaces to allocate.  
  pub fn reserve_exact(&mut self, mut used_cap: usize, additional: usize,) {
    used_cap = used_cap.checked_add(additional,)
        .expect("`RawVec::reserve_exact` additional overflowed usize");

    //Allocate only if necessary.
    if used_cap > self.cap() {
      //Calculate the new capacity.
      let new_cap = self.cap().checked_add(additional,)
        .expect("`RawVec::reserve_exact` additional overflowed usize");

      //Allocate a new `RawVec` if there was no allocation.
      if self.cap() == 0 { *self = RawVec::with_capacity(new_cap,) }
      else {
        //Reallocate the buffer.
        self.buf = match unsafe { Global.realloc_array(NonNull::new_unchecked(self.buf,), self.cap(), new_cap,) } {
          Ok(buf) => buf,
          Err(e) => panic!(alloc_err!("RawVec::with_capacity", e,)),
        }.as_ptr();
        //Update the capacity.
        self.cap = new_cap;
      }
    }
  }
  /// Reserves enough space for at least `additional` more values.
  /// 
  /// # Params
  /// 
  /// used_cap --- The used capacity of the buffer.  
  /// additional --- The additional spaces to allocate.  
  pub fn reserve(&mut self, mut used_cap: usize, additional: usize,) {
    used_cap = used_cap.checked_add(additional,)
        .expect("`RawVec::reserve_exact` additional overflowed usize");

    //Allocate only if necessary.
    if used_cap > self.cap() {
      //Calculate the new capacity.
      let new_cap = usize::max(self.cap().saturating_mul(2,), used_cap,);
      
      //If there was not allocation just create an allocation.
      if self.cap() == 0 { *self = RawVec::with_capacity(new_cap,) }
      else {
        //Reallocate the buffer.
        self.buf = match unsafe { Global.realloc_array(NonNull::new_unchecked(self.buf,), self.cap(), new_cap,) } {
          Ok(buf) => buf,
          Err(e) => panic!(alloc_err!("RawVec::with_capacity", e,)),
        }.as_ptr();
        //Set the new capacity.
        self.cap = new_cap;
      }
    }
  }
  /// Gets the pointer to the start of the buffer.
  #[inline]
  pub const fn ptr(&self,) -> *mut T { self.buf }
}

impl<T,> Drop for RawVec<T,> {
  fn drop(&mut self,) {
    //Deallocate only if there was an allocation.
    if self.buf != ptr::null_mut() {
      //Deallocate the buffer.
      if let Err(e) = unsafe { Global.dealloc_array(NonNull::new_unchecked(self.buf,), self.cap,) } {
        panic!(alloc_err!("`RawVec::drop`", e,),)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_raw_vec() {
    let vec = RawVec::<i32,>::with_capacity(0,);

    assert_eq!(vec.cap(), 0, "`RawVev::with_capaicty` cap was not `0` when created with capacity of 0",);
    
    let mut vec = RawVec::<i32,>::with_capacity(10,);

    assert_eq!(vec.cap(), 10, "`RawVev::with_capacity` cap was not `10` when created with capacity of 10",);

    vec.reserve_exact(10, 10,);
    assert_eq!(vec.cap(), 20, "`RawVev::reserve_exact` cap was not `20` when 10 was added exactly",);

    vec.reserve(10, 1,);
    assert_eq!(vec.cap(), 20, "`RawVev::reserve` cap was not `20` when 1 was added with space left",);
    vec.reserve(20, 1,);
    assert_eq!(vec.cap(), 40, "`RawVev::reserve` cap was not `40` when 1 was added with no space left",);
  }
}
