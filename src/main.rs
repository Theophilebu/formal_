
mod grammar;
mod parsing;
mod lexing;
mod symbol;
mod datastructures;

fn main() {
    let mut x = 5;
    let y = &mut x;
    *y += 1;
    x+=1;
    println!("{}", x); // prints 7
}
