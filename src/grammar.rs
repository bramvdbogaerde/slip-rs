
use std::ops::Deref;

use crate::memory::{Memory, MemPtr, Any, ChunkContent, Cell};

pub struct Number {
    pub raw: usize,
}

impl Number {
    pub fn new<'t>(mem: &Memory, n: usize) -> MemPtr<'t, Number> {
        let mut num = mem.allocate::<Number>(0);
        num.raw = n;
        num
    }
}

impl ChunkContent for Number {
    const CELLS: u32 = 1;
    const TAG: u32 = 0;
}

pub struct Pair<'t> {
    pub _car: Cell<'t, Any>,
    pub _cdr: Cell<'t, Any>
}

impl<'t> Pair<'t> {
    pub fn new(mem: &Memory) -> MemPtr<'t, Pair>  {   
        let mut pai = mem.allocate::<Pair>(0);
        pai._car = Cell::Empty; 
        pai._cdr = Cell::Empty;
        pai
    }

    pub fn car(&self) -> &MemPtr<Any> {
        self._car.deref()
    }

    pub fn cdr(&self) -> &MemPtr<Any> {
        self._cdr.deref()
    }
}

impl<'t> ChunkContent for Pair<'t> {
    const CELLS: u32 = 2;
    const TAG: u32 = 1;
}
