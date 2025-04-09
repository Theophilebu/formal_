
mod grammar;
mod parsing;
mod lexing;
mod symbol;
mod bitset;

fn main() {
    println!("Hello, world!");
}

fn thing(mut s1: String) {
    s1.push('c');
    let s2 = &mut s1;
    s2.push('d');
}