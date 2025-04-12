use std::num::NonZeroUsize;

pub trait Table1D<T> {
    fn get(&self, i: usize) -> &T;
    fn size(&self) -> NonZeroUsize;
}

impl <T> Table1D<T> for Vec<T> {
    // should not be empty, or it will lead to unrecoverable panics

    fn get(&self, i: usize) -> &T {
        &self[i]
    }

    fn size(&self) -> NonZeroUsize {
        self.len().try_into().unwrap()
    }
}


impl <T, const SIZE: usize> Table1D<T> for [T; SIZE] {
    // SIZE should be at least one, or it will lead to unrecoverable panics
    // NonZeroUsize is incompatible with const generic parameters

    fn get(&self, i: usize) -> &T {
        &self[i]
    }

    fn size(&self) -> NonZeroUsize {
        SIZE.try_into().unwrap()
    }

}