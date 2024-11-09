use std::{borrow::{Borrow, BorrowMut}, cell::RefCell, marker::PhantomData};

use anyhow::{anyhow, Result};
use bitfield_struct::bitfield;

/// A pointer to an untyped memory chunk
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ptr<'t> {
    ptr: *const u64,
    pd: PhantomData<&'t ()>
}

impl<'t> Ptr<'t> {
    /// Returns true if the given reference has the same pointer
    /// value as the current pointer.
    fn equal<T>(&self, that: &T) -> bool {
        std::ptr::eq(self.ptr as *const T, that)
    }

    /// Applies the given function to the value if the memory chunk
    /// contains a value of the correct type, otherwise panics.
    pub fn modify<T: Element>(&self, f: impl FnOnce(&mut T)) {
        f(self.cast_mut::<T>().unwrap());
    }

    /// Safe casting mechanism, looks at the tag the pointer is pointing to 
    /// returns a shared reference inside a Result which can result in a 
    /// runtime error when casting was not allowed.
    pub fn cast<T: Element>(&'t self) -> Result<&'t T> {
        unsafe {
            // SAFETY: Dereferencing the raw point is safe 
            // since it is created by the `Memory`. Thus, 
            // it has an initialized header with the correct types.
            let hdr: Header = *(self.ptr as *const Header);
            // safe to convert the chunk to a pointer to 
            // the required type.
            println!("found tag: {}, expected tag: {}", hdr.tag(), T::tag());
            if hdr.tag() == T::tag() {
                Ok(&*(self.ptr as *const T))
            } else {
                Err(anyhow!("invalid memory chunk"))
            }
        }
    }

    /// Same as `cast` but returns an exclusive mutable reference
    pub fn cast_mut<T: Element>(&'t self) -> Result<&'t mut T> {
        unsafe {
            // SAFETY: Dereferencing the raw point is safe 
            // since it is created by the `Memory`. Thus, 
            // it has an initialized header with the correct types.
            let hdr: Header = *(self.ptr as *const Header);
            // safe to convert the chunk to a pointer to 
            // the required type.
            if hdr.tag() == T::tag() {
                Ok(&mut *(self.ptr as *mut T))
            } else {
                Err(anyhow!("invalid memory chunk"))
            }
        }
    }
}

#[bitfield(u64)]
pub struct Header {
    /// Bit set when the chunk is considered to be "raw" (i.e., should not be considered by the 
    /// garbage collector)
    #[bits(1)]
    is_raw: bool,
    #[bits(7)]
    tag: usize,
    #[bits(56)]
    size: usize
}

impl Header {
    fn initialize(is_raw: bool, tag: usize,  size: impl Into<usize>) -> Header {
        Header::new().with_is_raw(is_raw).with_tag(tag).with_size(size.into())
    }
}

/// Basic structure of an untyped memory chunk
pub struct Chunk {
    hdr: Header, 
}

/// Memory abstraction
pub struct Memory<'t> {
    /// Free pointer into linear
    free_pointer: RefCell<*const u64>,
    /// Linear memory map
    _linear: &'t [u64]
}

impl<'t> Memory<'t> {
    /// Create a new memory instance with the given array
    /// as its backing storage
    pub fn new<'s : 't>(memory: &'s mut [u64]) -> Memory<'t> {
        Memory { free_pointer: memory.as_ptr().into(), _linear: memory }
    }

    fn allocate_<T: Element>(&'t self, additional_size: isize, is_raw: bool) -> Ptr<'t> {
       let size = T::size() + additional_size;
       unsafe {
            // SAFETY: this just increases the free pointer with the required size,
            // no other unsafe operations are performed on this pointer.
            let current = *self.free_pointer.borrow();
            *self.free_pointer.borrow_mut() = current.offset(size + 1);
            // create memory structure
            let hdr = Header::initialize(is_raw, T::tag(), size.unsigned_abs());
            *(current as *mut Header) = hdr;
            Ptr { ptr: current, pd: PhantomData }
       }
    }

    /// Allocate a memory chunk for the given 
    /// type with the given number of cells.
    ///
    /// The returned pointer can only live as long as the memory does,
    /// so that the following code does not compile: 
    ///
    /// ```compile_fail
    /// let data : [ u64 ; 5 ] = [ 0 ; 5 ];
    /// let mem = Memory::new(&mut data);
    /// ptr = mem.allocate::<()>();
    /// mem.destroy();
    /// println("{:?}", ptr);
    /// ```
    pub fn allocate<T: Element>(&'t self, additional_size: isize) -> Ptr<'t> {
        self.allocate_::<T>(additional_size, false)
    }

    /// Allocate a raw memory chunk for 
    /// the given type
    pub fn allocate_raw<T: Element>(&'t self, additional_size: isize) -> Ptr<'t> {
        self.allocate_::<T>(additional_size, true)
    }

    /// Destroy the memory
    pub fn destroy(self) { }

    /// Garbage collect with the given pointer as roots, requires
    /// exclusive access to the memory as well as the roots.
    pub fn collect<T: Element>(&self, roots: &mut (&mut T)) {}

}

/// A struct can be a memory chunk if the required 
/// number of cells is known ahead of time.
pub trait Element {
    fn size() -> isize;
    fn tag() -> usize;
}

#[cfg(test)]
mod test {
    use super::*;
    
    struct Number {
        _hdr: Header, 
        n: u64
    }

    impl Element for Number {
        fn size() -> isize { 1 }
        fn tag() -> usize { 2 }
    }


    struct Pair<'t> {
        _hdr: Header,
        car: Ptr<'t>,
        cdr: Ptr<'t>
    }

    impl<'t> Element for Pair<'t> {
        fn size() -> isize { 2 }
        fn tag() -> usize { 1 }
    }

    #[test]
    fn test_ptr_size() {
        // ensure that the pointer size is 8 bytes (i.e., 64 bits)
        assert!(size_of::<Ptr<'static>>() == 8)
    }

    #[test]
    fn test_pair() {
        let mut data: [u64 ; 1000] = [ 0 ; 1000 ];
        let mem = Memory::new(&mut data);
        let ptr = mem.allocate::<Pair>(0);
        let n = mem.allocate_raw::<Number>(0);
        n.modify::<Number>(|nv| {
            nv.n = 42
        });
        ptr.modify::<Pair>(|pai| { 
            pai.car = n.clone();
            pai.cdr = n.clone(); 
        });

        let pai = ptr.cast::<Pair>().unwrap();
        assert!(pai.car == n);
        assert!(pai.cdr == n);
        assert!(pai.car.cast::<Number>().unwrap().n == 42);
    }
}
