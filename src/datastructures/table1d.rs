use std::num::NonZeroUsize;

/*
I am implementing a bit set in Rust. It is a struct that owns some data that implements the Table1d trait. This way, it can both hold data created at run-time or known at compile-time.
I need to implement a method "new" which creates an empty bit set, common to every implementation 
*/

pub trait Table1D<T> {
    fn get(&self, i: usize) -> &T;
    fn get_mut(&mut self, i: usize) -> &mut T;
    fn set(&mut self, value: T, i: usize);
    fn size(&self) -> NonZeroUsize;
    fn new_filled(value: T, size: usize) -> Self;
}

impl <T: Clone> Table1D<T> for Vec<T> {
    // should not be empty, or it will lead to unrecoverable panics

    fn get(&self, i: usize) -> &T {
        &self[i]
    }

    fn get_mut(&mut self, i: usize) -> &mut T {
        &mut self[i]
    }

    fn set(&mut self, value: T, i: usize) {
        self[i] = value;
    }

    fn size(&self) -> NonZeroUsize {
        self.len().try_into().unwrap()
    }

    fn new_filled(value: T, size: usize) -> Self {
        vec![value; size]
    }
}


impl <T: Copy, const SIZE: usize> Table1D<T> for [T; SIZE] {
    // SIZE should be at least one, or it will lead to unrecoverable panics
    // NonZeroUsize is incompatible with const generic parameters

    fn get(&self, i: usize) -> &T {
        &self[i]
    }

    fn get_mut(&mut self, i: usize) -> &mut T {
        &mut self[i]
    }

    fn set(&mut self, value: T, i: usize) {
        self[i] = value;
    }

    fn size(&self) -> NonZeroUsize {
        SIZE.try_into().unwrap()
    }

    fn new_filled(value: T, size: usize) -> Self {
        assert_eq!(SIZE, size);
        [value; SIZE]
    }

}
