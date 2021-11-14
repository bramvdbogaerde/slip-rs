use std::{convert::TryInto, marker::PhantomData, mem, ops::{Deref, DerefMut}, cell::RefCell};

const CELL_SIZE: usize = mem::size_of::<u64>();

pub type Header = u64;

fn hdr_tag(hdr: Header) -> u32 {
    (hdr >> 32).try_into().expect("tag to fit in 32-bits")
}

fn hdr_size(hdr: Header) -> u32 {
    (hdr & 0x00000000FFFFFFFF).try_into().expect("size to fit in 32-bits")
}

fn make_hdr(size: u32, tag: u32) -> Header {
    let size64 = size as u64;
    let tag64  = tag as u64;
    (tag64 << 32) | size64
}

#[repr(C)]
pub struct Any {
    hdr: Header
}

#[repr(C)]
pub struct Chunk<T> {
    hdr: Header,
    ext: T
}

pub trait ChunkContent {
    const CELLS: u32;
    const TAG: u32;
}

pub struct Memory {
    memory: *mut u64,
    current_free: RefCell<usize>,
    size: usize, 
}

#[repr(C)]
pub struct MemPtr<'t, T> {
    pd: PhantomData<&'t T>,
    ptr: *mut Chunk<T>
}

impl<'t, T> MemPtr<'t, T> {
    fn new(ptr: *mut Chunk<T>) -> MemPtr<'t, T> {
        MemPtr { pd: PhantomData, ptr }
    }

    /// Runtime cast of the given pointer
    pub fn cast<S>(&self) -> Option<MemPtr<'t, S>> 
        where S: ChunkContent {
        let tag = unsafe {
            hdr_tag((*self.ptr).hdr)
        };

        if tag == S::TAG {
            Some(MemPtr::new(self.ptr as *mut Chunk<S>))
        } else {
            None
        }
    }

    /// You can cast any type to the any cell
    pub fn any(&self) -> MemPtr<'t, Any> {
        MemPtr::new(self.ptr as *mut Chunk<Any>)
    }

    /// Returns the type of the cell
    pub fn tag(&self) -> u32 {
        unsafe { hdr_tag((*self.ptr).hdr) }
    }

    unsafe fn unsafe_clone(&self) -> Self {
        MemPtr {
            pd: PhantomData,
            ptr: self.ptr
        }
    }
}

pub enum Cell<'t, T> {
    Ptr { data: MemPtr<'t,T> },
    Empty
}

impl<'t, T> Cell<'t, T> {
    pub unsafe fn new(v: &MemPtr<'t, T>) -> Cell<'t, T> {
        // SAFETY: same argument as the `set`
        Cell::Ptr {
            data:  v.unsafe_clone()
        }
    }
    pub fn set(&mut self, v: &MemPtr<'t, T>) {
        // SAFETY: we assume that since `set` is called on 
        // a cell, the cell is part of a structure that is rooted,
        // and since it is rooted, its pointer will be updated correctly
        // when GC'en. Because the Memory::allocate function only returns 
        // exclusive references, any use of unrooted memory will be rejected by the compiler.
        match self {
            Cell::Ptr { ref mut data } => *data = unsafe { v.unsafe_clone() },
            Cell::Empty => *self = Cell::Ptr { data: unsafe { v.unsafe_clone() } }
        }
    }

    pub fn deref(&self) -> &MemPtr<'t, T> {
        match self {
            Cell::Ptr { ref data } => data,
            Cell::Empty => panic!("Cannot dereference empty cell")
        }
    }
}



impl<'t, T> Deref for MemPtr<'t, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &((*self.ptr).ext) }
    }
}

impl<'t, T> DerefMut for MemPtr<'t, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut (*self.ptr).ext }
    }
}

impl<'t> Memory {
    pub fn new(size: usize) -> Memory {
        let mut memory = vec![ 0 as u64 ; size ];
        Memory {
            memory: memory.as_mut_ptr(),
            current_free: RefCell::new(0),
            size
        }
    }

    pub fn allocate<T: ChunkContent>(&self, extra_cells: u32) -> MemPtr<'t, T> {
        let size = 1 + T::CELLS + extra_cells;
        let ptr: *mut Chunk<T> = unsafe { self.memory.offset(*self.current_free.borrow() as isize) } as *mut Chunk<T>;
        unsafe { (*ptr).hdr = make_hdr(size, T::TAG) }
        self.current_free.replace_with(|free| *free+(size as usize)); 
        MemPtr::new(ptr)
    }

    pub fn gc(mut self) -> Memory { 
        self.memory = vec![ 0 as u64 ; self.size ].as_mut_ptr();
        drop(self);
        Memory::new(1000)
    }
}
