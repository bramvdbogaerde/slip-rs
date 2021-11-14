mod memory;
mod grammar;
mod printer;

use memory::{Memory, Cell};
use grammar::*;
use printer::print;



fn main() {
    let mut mem = Memory::new(1000);
    let mut pai =  mem.allocate::<Pair>(0);

    let mut pai2 =  mem.allocate::<Pair>(0);
    let one = Number::new(&mut mem, 1);
    let two = Number::new(&mut mem, 2);

    pai._car.set(&one.any());
    pai2._car.set(&one.any());
    pai2._cdr.set(&two.any());
    pai2._cdr.set(&pai2.any());

    mem.gc();

    // the following below should not be allowed after GC, the "pai" should either be rooted
    // or should be retrieved from some other rooted memory
    print(&pai.any());

    println!("");
}
