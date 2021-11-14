
use crate::memory::{MemPtr, Any, ChunkContent};
use crate::grammar::*;

pub fn print<'t>(m: &MemPtr<Any>) -> () {
    match m.tag() {
        Pair::TAG => {
            let pai = m.cast::<Pair>().unwrap();
            print!("(");
            print(&pai.car());
            print!(" . ");
            print(&pai.cdr());
            print!(")");

        },
        Number::TAG => {
            let num = m.cast::<Number>().unwrap();
            print!("{}", num.raw);
        }
         _ => panic!("Unknown type")
    }
}
