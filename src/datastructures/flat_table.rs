use std::ops::Index;
use num::{Unsigned, PrimInt};
use std::fmt::Debug;

/// stores values of type T
pub struct FlatTable<T, Idxx>
where
    Idxx: Unsigned + PrimInt,
    Idxx: TryFrom<usize>,
    usize: From<Idxx>,
    <Idxx as TryFrom<usize>>::Error: Debug,
{
    pub table: Vec<T>,
    // points to the start of each subarray representing a row
    pub rows: Vec<Idxx>,
}

impl <T, Idxx> FlatTable<T, Idxx>
where
    Idxx: Unsigned + PrimInt,
    Idxx: TryFrom<usize>,
    usize: From<Idxx>,
    <Idxx as TryFrom<usize>>::Error: Debug,
{
    pub fn new(table: Vec<T>, rows: Vec<Idxx>) -> Self {
        Self {table, rows}
    }

    pub fn from_vec_vec(mut table2d: Vec<Vec<T>>) -> Self {
        let total_size: usize = table2d.iter().map(|v| v.len()).sum();
        let mut table: Vec<T> = Vec::with_capacity(total_size);
        let mut rows: Vec<Idxx> = Vec::with_capacity(table2d.len());

        for values in table2d.iter_mut() {
            rows.push(table.len().try_into().unwrap());
            table.append(values);
        }

        Self { table, rows }
    }

    pub fn size(&self) -> Idxx {
        self.table.len().try_into().unwrap()
    }

    pub fn get_by_id(&self, id: Idxx) -> &T {
        &self.table[usize::from(id)]
    }
}

impl <T, Idxx> Index<Idxx> for FlatTable<T, Idxx>
where
    Idxx: Unsigned + PrimInt,
    Idxx: TryFrom<usize>,
    usize: From<Idxx>,
    <Idxx as TryFrom<usize>>::Error: Debug,
{
    type Output = [T];

    fn index(&self, index: Idxx) -> &[T] {
        let id: usize = index.try_into().unwrap();
        let start: usize = self.rows[id].into();
        let end: usize = (*self.rows.get(id+1).unwrap_or(&self.size())).into();
        &self.table[start..end]
    }
}

pub struct RectFlatTable<T, Idxx>
where
    Idxx: Unsigned + PrimInt,
    Idxx: TryFrom<usize>,
    usize: From<Idxx>,
    <Idxx as TryFrom<usize>>::Error: Debug,
{
    pub table: Vec<T>,
    pub height: Idxx,
    pub width: usize,
}

impl <T, Idxx> RectFlatTable<T, Idxx>
where
    Idxx: Unsigned + PrimInt,
    Idxx: TryFrom<usize>,
    usize: From<Idxx>,
    <Idxx as TryFrom<usize>>::Error: Debug,
{
    pub fn new(table: Vec<T>, height: Idxx, width: usize) -> Self {
        Self {table, height, width}
    }

    pub fn from_vec_vec(mut table2d: Vec<Vec<T>>) -> Self {
        let height: Idxx = table2d.len().try_into().unwrap();
        let width: usize = { if height==Idxx::zero() {0} else {table2d[0].len()}};
        let mut table: Vec<T> = Vec::with_capacity(usize::from(height)*width);

        for values in table2d.iter_mut() {
            table.append(values);
        }

        Self { table, height, width }
    }
}

impl <T ,Idxx> Index<Idxx> for RectFlatTable<T, Idxx>
where
    Idxx: Unsigned + PrimInt,
    Idxx: TryFrom<usize>,
    usize: From<Idxx>,
    <Idxx as TryFrom<usize>>::Error: Debug,
{
    type Output = [T];

    fn index(&self, index: Idxx) -> &[T] {
        &self.table[usize::from(index)*self.width..usize::from(index)*(self.width+1)]
    }
}