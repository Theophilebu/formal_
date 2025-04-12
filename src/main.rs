mod grammar;
mod parsing;
mod lexing;
mod symbol;
mod datastructures;

use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{EnumCount as EnumCountMacro, EnumIter};

const tifhr: [usize; 2] = [0, 1];
const tifhr2: [[usize; 2]; 2] = [[0, 1], [2, 3]];

#[derive(EnumCountMacro, EnumIter)]
enum ExmplNonTermSymb {
    A,
    B,
    C,
}

fn main() {
    let value: usize = ExmplNonTermSymb::A as usize;
    for x in ExmplNonTermSymb::iter() {
        println!("{}", ExmplNonTermSymb::COUNT);

    }
    println!("{}", value); // prints 7
}
