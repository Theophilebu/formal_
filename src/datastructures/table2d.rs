use std::num::NonZeroUsize;

pub trait Table2D<T> {
    fn get(&self, i: usize, j: usize) -> &T;
    fn get_mut(&mut self, i: usize, j: usize) -> &mut T;
    fn height(&self) -> NonZeroUsize;
    fn width(&self) -> NonZeroUsize;
    fn new_filled(value: T, height: usize, width: usize) -> Self;
}

impl <T: Clone> Table2D<T> for Vec<Vec<T>> {
    // no vector should be empty, or it will lead to unrecoverable panics

    fn get(&self, i: usize, j: usize) -> &T {
        &self[i][j]
    }

    fn get_mut(&mut self, i: usize, j: usize) -> &mut T {
        &mut self[i][j]
    }

    fn height(&self) -> NonZeroUsize {
        self.len().try_into().unwrap()
    }

    fn width(&self) -> NonZeroUsize {
        self[0].len().try_into().unwrap()
    }

    fn new_filled(value: T, height: usize, width: usize) -> Self {
        vec![vec![value; width]; height]
    }
}


impl <T: Copy, const HEIGHT: usize, const WIDTH: usize> Table2D<T> for [[T; WIDTH]; HEIGHT] {
    // HEIGHT and WIDTH should be at least one, or it will lead to unrecoverable panics
    // NonZeroUsize is incompatible with const generic parameters

    fn get(&self, i: usize, j: usize) -> &T {
        &self[i][j]
    }

    fn get_mut(&mut self, i: usize, j: usize) -> &mut T {
        &mut self[i][j]
    }

    fn height(&self) -> NonZeroUsize {
        HEIGHT.try_into().unwrap()
    }

    fn width(&self) -> NonZeroUsize {
        WIDTH.try_into().unwrap()
    }

    fn new_filled(value: T, height: usize, width: usize) -> Self {
        assert_eq!(HEIGHT, height);
        assert_eq!(WIDTH, width);
        [[value; WIDTH]; HEIGHT]
    }

}