mod grammars;
mod parsing;
mod lexing;
mod datastructures;
mod formal_language;

use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{EnumCount as EnumCountMacro, EnumIter};


// #[derive(EnumCountMacro, EnumIter)]
// enum ExmplNonTermSymb {
//     A,
//     B,
//     C,
// }
// let value: usize = ExmplNonTermSymb::A as usize;
// for x in ExmplNonTermSymb::iter() {
//     println!("{}", ExmplNonTermSymb::COUNT);

// }
// println!("{}", value); // prints 7

fn main() {
    println!("{}", std::mem::size_of::<Vec<u8>>());
}
