use num::{Unsigned, PrimInt};
use std::{fmt::Binary, marker::PhantomData};

use super::table1d::Table1D;

pub struct BitSet<UINT: Unsigned + PrimInt + Binary, TABLE: Table1D<UINT>> 
{
    data: TABLE,
    size: usize,
    phantom: PhantomData<UINT>,
}

/* 

impl <UINT> BitSet<UINT, Vec<UINT>>
where
    UINT: Unsigned + PrimInt + Binary,
{
    pub fn new_empty(size: usize)
    -> Self {
        // the size is known at run-time
        let nbr_uints: usize = (size+7)/(8*size_of::<UINT>());
        BitSet{ 
            data: vec![UINT::min_value(); nbr_uints], 
            size,
            phantom: PhantomData,
        }
    }

    pub fn new_full(size: usize) 
    -> Self {
        // the size is known at run-time
        let nbr_uints: usize = (size+7)/(8*size_of::<UINT>());
        BitSet{ 
            data: vec![UINT::max_value(); nbr_uints], 
            size,
            phantom: PhantomData,
        }
    }
}


impl <UINT, const NBR_UINTS: usize> BitSet<UINT, [UINT; NBR_UINTS]>
where
    UINT: Unsigned + PrimInt + Binary,
{
    pub fn new_empty(size: usize)
    -> Self {
        // the memory size is known at compile-time
        assert!((size + 7) / (8 * size_of::<UINT>()) <= NBR_UINTS,
            "NBR_UINTS should be greater or equal to (size + 7) / (8 * size_of::<UINT>()).
            If you can't guaretee this, use new_empty_var_size");

        BitSet{ 
            data: [UINT::min_value(); NBR_UINTS], 
            size: size,
            phantom: PhantomData,
        }
    }

    pub fn new_full(size: usize)
    -> Self {
        // the memory size is known at compile-time
        assert!((size + 7) / (8 * size_of::<UINT>()) <= NBR_UINTS,
            "NBR_UINTS should be greater or equal to (size + 7) / (8 * size_of::<UINT>()).
            If you can't guaretee this, use new_full_var_size");

        BitSet{ 
            data: [UINT::max_value(); NBR_UINTS], 
            size: size,
            phantom: PhantomData,
        }
    }
}

*/
// - - - - - - - - - - - - - - - - 

impl <UINT, TABLE> BitSet<UINT, TABLE>
where 
    UINT: Unsigned + PrimInt + Binary,
    TABLE: Table1D<UINT>,
    for <'a> &'a TABLE: IntoIterator,
    for <'a> <&'a TABLE as IntoIterator>::Item : Into<&'a UINT>,
    // Item of the iterator can be converted into UINT
    // apparently, writing " for<'a> <&'a T as IntoIterator>::Item == UINT " is unstable
{

    pub fn size(&self) -> usize {
        self.size
    }

    fn nbr_uints(&self) -> usize {
        self.data.size().into()
    }

    fn nbr_used_uints(&self) -> usize {
        let size: usize = self.data.size().into();
        (size + 7) / (8 * size_of::<UINT>())
    }

    fn new_filled(value: bool, size: usize) -> BitSet<UINT, TABLE> {
        let uint_value = match value {
            true => {UINT::max_value()}
            false => {UINT::min_value()}
        };
        BitSet {
            data: TABLE::new_filled(uint_value, (size + 7) / (8 * size_of::<UINT>())),
            size: size,
            phantom: PhantomData
        }
    }

    pub fn len(&self) -> usize {
        // |!| not constant time
        (&self.data)
            .into_iter()
            .map(|x: <&TABLE as IntoIterator>::Item| (Into::<&UINT>::into(x)).count_ones() )
            .sum::<u32>() 
            as usize
    }


    pub fn contains(&self, n: usize) -> bool{
        if n>=self.size(){
            panic!("index out of bounds");
        }
        let uint_index: usize = n/(8*size_of::<UINT>()) as usize;   // index in data
        let position_in_uint: usize = n - (8*size_of::<UINT>())*uint_index;
        let bit_mask: UINT = UINT::one() << (8*size_of::<UINT>() - 1 - position_in_uint);
        // return (bit_mask & self.data[uint_index])>UINT::zero();
        return (bit_mask & *self.data.get(uint_index))>UINT::zero();
    }

    pub fn insert(&mut self, n: usize) {
        if n>=self.size(){
            panic!("index out of bounds");
        }
        let uint_index: usize = n/(8*size_of::<UINT>()) as usize;   // index in data
        let position_in_uint: usize = n - (8*size_of::<UINT>())*uint_index;
        let bit_mask: UINT = UINT::one() << (8*size_of::<UINT>() - 1 - position_in_uint);
        *self.data.get_mut(uint_index) = bit_mask | *self.data.get(uint_index);
    }

    pub fn remove(&mut self, n: usize) {
        if n>=self.size(){
            panic!("index out of bounds");
        }
        let uint_index: usize = n/(8*size_of::<UINT>()) as usize;   // index in data
        let position_in_uint: usize = n - (8*size_of::<UINT>())*uint_index;
        let bit_mask: UINT = UINT::one() << (8*size_of::<UINT>() - 1 - position_in_uint);
        *self.data.get_mut(uint_index) = bit_mask & *self.data.get(uint_index);
    }

    pub fn set_value(&mut self, n: usize, value: bool){

        if n>=self.size(){
            panic!("index out of bounds");
        }
        if value{
            self.insert(n);
        } else{
            self.remove(n);
        }
    }

    
    pub fn union(&self, other: &Self) -> Self {
    
        if self.size() != other.size() {
            panic!("operations on bitsets with different sizes are not allowed");
        }

        let new_bitset: BitSet<UINT, TABLE> = BitSet::new_filled(false, self.size);
        let tmp: &UINT = new_bitset.data.get(0);
        *tmp = UINT::zero();
        new_bitset.data;

        for i in 0..self.nbr_uints() {
            *new_bitset.data.get(i) = *self.data.get(i) | *other.data.get(i);
            *TABLE::get(&new_bitset.data, i) = UINT::zero();
        }

        new_bitset
    }

    pub fn intersection(&self, other: &Self) 
    -> Self{
        if self.size() != other.size() {
            panic!("operations on bitsets with different sizes are not allowed");
        }

        let mut new_bitset: BitSet<UINT> = BitSet::new_empty(self.size);

        for i in 0..self.nbr_uints() {
            new_bitset.data[i] = self.data[i] & other.data[i];
        }

        new_bitset
    }

    pub fn difference(&self, other: &Self) 
    -> Self{
        if self.size() != other.size() {
            panic!("operations on bitsets with different sizes are not allowed");
        }

        let mut new_bitset: BitSet<UINT> = BitSet::new_empty(self.size);

        for i in 0..self.nbr_uints() {
            new_bitset.data[i] = self.data[i] & !other.data[i];
        }

        new_bitset
    }

    pub fn symmetric_difference(&self, other: &Self) 
    -> Self{
        if self.size() != other.size() {
            panic!("operations on bitsets with different sizes are not allowed");
        }

        let mut new_bitset: BitSet<UINT> = BitSet::new_empty(self.size);

        for i in 0..self.nbr_uints() {
            new_bitset.data[i] = self.data[i] ^ other.data[i];
        }

        new_bitset
    }

    pub fn complement(&self) 
    -> Self{
        let mut new_bitset: BitSet<UINT> = BitSet::new_empty(self.size);

        for i in 0..self.nbr_uints() {
            new_bitset.data[i] = !self.data[i];
        }

        new_bitset
    }


    pub fn concatenate(&self, other: &Self) -> Self{
        let mut new_bitset: BitSet<UINT> = BitSet::new_empty(self.size()+other.size());

        for i in 0..self.nbr_uints() {
            new_bitset.data[i] = self.data[i];
        }

        // number of bits NOT USED in the last uint of the data of self
        let right: usize = 8*size_of::<UINT>()*self.nbr_uints() - self.size();

        // number of bits USED in the last uint of the data of self
        let left: usize = 8*size_of::<UINT>() - right;


        // -1 to avoid index out of bounds
        for index_in_uint in 0..other.nbr_uints()-1 {
            new_bitset.data[self.nbr_uints()+index_in_uint-1] = 
                new_bitset.data[self.nbr_uints()-1] | other.data[index_in_uint] >> left;
            new_bitset.data[self.nbr_uints()+index_in_uint] = other.data[index_in_uint] << right;            
        }
        new_bitset.data[self.nbr_uints()+other.nbr_uints()-2] = 
            new_bitset.data[self.nbr_uints()+other.nbr_uints()-2] | other.data[other.nbr_uints()-1] >> left;
        new_bitset
    }


    pub fn update_union(&mut self, other: &Self) {
        if self.size() != other.size() {
            panic!("operations on bitsets with different sizes are not allowed");
        }

        for i in 0..self.nbr_uints() {
            self.data[i] = self.data[i] | other.data[i];
        }
    }

    pub fn update_intersection(&mut self, other: &Self) {
        if self.size() != other.size() {
            panic!("operations on bitsets with different sizes are not allowed");
        }

        for i in 0..self.nbr_uints() {
            self.data[i] = self.data[i] & other.data[i];
        }
    }

    pub fn update_difference(&mut self, other: &Self) {
        if self.size() != other.size() {
            panic!("operations on bitsets with different sizes are not allowed");
        }

        for i in 0..self.nbr_uints() {
            self.data[i] = self.data[i] & !other.data[i];
        }
    }

    pub fn update_symmetric_difference(&mut self, other: &Self) {
        if self.size() != other.size() {
            panic!("operations on bitsets with different sizes are not allowed");
        }

        for i in 0..self.nbr_uints() {
            self.data[i] = self.data[i] ^ other.data[i];
        }
    }

    pub fn update_complement(&mut self) {
        for i in 0..self.nbr_uints() {
            self.data[i] = !self.data[i];
        }
    }


    pub fn clear(&mut self){
        for i in 0..self.nbr_uints(){
            self.data[i] = UINT::min_value();
        }
    }

    pub fn fill(&mut self){
        for i in 0..self.nbr_uints(){
            self.data[i] = UINT::max_value();
        }
    }


    pub fn is_disjoint(&self, other: &Self) -> bool{
        if self.size() != other.size() {
            panic!("operations on bitsets with different sizes are not allowed");
        }

        for i in 0..self.nbr_uints(){
            if self.data[i] & other.data[i] > UINT::zero() {
                return false;
            }
        }
        return true;
    }

    pub fn is_empty(&self) -> bool{
        for uint in &self.data{
            if *uint>UINT::zero(){
                return false;
            }
        }
        true
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        for i in 0..self.nbr_uints(){
            if self.data[i] & !other.data[i] > UINT::zero() {
                return false;
            }
        }
        true
    }

    pub fn is_superset(&self, other: &Self) -> bool {
        for i in 0..self.nbr_uints(){
            if !self.data[i] & other.data[i] > UINT::zero() {
                return false;
            }
        }
        true
    }


    pub fn print(&self){
        let mut s: String = String::from("");
        for uint in &self.data{
            let uint_str: String = format!("{uint:b}");
            // total starts with 0 bits
            let mut total: String = std::iter::repeat("0")
                .take(8-uint_str.len())
                .collect();

            total.push_str(&uint_str);
            s.push_str(&total);
        }
        println!("{s}");
    }


    pub fn iter<'se>(&'se self) -> BitSetIter<'se, UINT> {
        BitSetIter::new(self)
    }

}

impl <UINT: Unsigned + PrimInt + Binary> Clone for BitSet<UINT>
{

    fn clone(&self) -> Self {
        BitSet {
            data: self.data.clone(),
            size: self.size,
        }
    }
}


// -------------------------- Iterator -----------------------


pub struct BitSetIter<'bitset, UINT: Unsigned + PrimInt + Binary>
{
    bitset: &'bitset BitSet<UINT>,

    uint_index: usize,
    index_in_uint: usize,
    
    working_uint: UINT,       // not really the actual uint, it gets modified
    nbr_ones_in_uint: u32,    // same, it gets decremented
}


impl <'bitset, UINT: Unsigned + PrimInt + Binary> BitSetIter<'bitset, UINT>
{

    pub fn new(bitset: &'bitset BitSet<UINT>) -> BitSetIter<'bitset, UINT> {
        BitSetIter{
            bitset,

            uint_index: 0,
            index_in_uint: 0,

            working_uint: bitset.data[0],
            nbr_ones_in_uint: bitset.data[0].count_ones(),
        }
    }

}


impl <'bitset, UINT: Unsigned + PrimInt + Binary> IntoIterator for &'bitset BitSet<UINT>
{
    type Item = usize;
    type IntoIter = BitSetIter<'bitset, UINT>;

    fn into_iter(self) -> Self::IntoIter {
        BitSetIter::new(self)
    }
}

impl <'bitset, UINT: Unsigned + PrimInt + Binary> Iterator for BitSetIter<'bitset, UINT>
{
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        

        // finds a non-empty uint, or returns None if the end is reached without encountering any zeros
        while self.nbr_ones_in_uint == 0 {
            // println!("cherche uint");
            
            self.uint_index += 1;
            if self.uint_index == self.bitset.nbr_uints() {
                return None;
            }
            
            self.working_uint = self.bitset.data[self.uint_index];
            self.nbr_ones_in_uint = self.working_uint.count_ones();
        }
        
        /*
        println!("uint_index: {}, index_in_uint: {}, working_uint: {:b}, nbr_ones_in_uint: {}",
        self.uint_index, self.index_in_uint, self.working_uint, self.nbr_ones_in_uint);
        println!("index: {}", 8*size_of::<UINT>()*self.uint_index + self.index_in_uint);
         */
        let one_left: UINT = UINT::one() << (8*size_of::<UINT>() - 1);
        loop {
            
            if (self.working_uint & one_left) == one_left {
                self.nbr_ones_in_uint -= 1;

                let res: Option<usize> = Some(8*size_of::<UINT>()*self.uint_index + self.index_in_uint);
                if res.unwrap() >= self.bitset.size() {
                    return None;
                }

                if self.nbr_ones_in_uint == 0 {
                    self.index_in_uint = 0; // returns to the start of the next uint
                }
                else {
                    self.working_uint = self.working_uint << 1; // skips to the next bit
                    self.index_in_uint += 1;
                }
                return res;
            }            
            else {
                self.working_uint = self.working_uint << 1;
                self.index_in_uint += 1;
            }
        }

    }

}

// -------------------------- Iterator mutable ref -----------------------


pub struct MutBitSetIter<'bitset, UINT: Unsigned + PrimInt + Binary>
{
    pub bitset: &'bitset mut BitSet<UINT>,

    uint_index: usize,
    index_in_uint: usize,
    
    working_uint: UINT,       // not really the actual uint, it gets modified
    nbr_ones_in_uint: u32,    // same, it gets decremented
}


impl <'bitset, UINT: Unsigned + PrimInt + Binary> MutBitSetIter<'bitset, UINT>
{

    pub fn new(bitset: &'bitset mut BitSet<UINT>) -> MutBitSetIter<'bitset, UINT> {
        let working_uint = bitset.data[0];
        let nbr_ones_in_uint = bitset.data[0].count_ones();
        MutBitSetIter{
            bitset,

            uint_index: 0,
            index_in_uint: 0,

            working_uint,
            nbr_ones_in_uint,
        }
    }

    pub fn reset(&mut self) {
        self.uint_index = 0;
        self.index_in_uint = 0;

        self.working_uint = self.bitset.data[0];
        self.nbr_ones_in_uint = self.bitset.data[0].count_ones();
    }

}


impl <'bitset, UINT: Unsigned + PrimInt + Binary> IntoIterator for &'bitset mut BitSet<UINT>
{
    type Item = usize;
    type IntoIter = MutBitSetIter<'bitset, UINT>;

    fn into_iter(self) -> Self::IntoIter {
        MutBitSetIter::new(self)
    }
}

impl <'bitset, UINT: Unsigned + PrimInt + Binary> Iterator for MutBitSetIter<'bitset, UINT>
{
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        

        // finds a non-empty uint, or returns None if the end is reached without encountering any zeros
        while self.nbr_ones_in_uint == 0 {
            // println!("cherche uint");
            
            self.uint_index += 1;
            if self.uint_index == self.bitset.nbr_uints() {
                return None;
            }
            
            self.working_uint = self.bitset.data[self.uint_index];
            self.nbr_ones_in_uint = self.working_uint.count_ones();
        }
        
        /*
        println!("uint_index: {}, index_in_uint: {}, working_uint: {:b}, nbr_ones_in_uint: {}",
        self.uint_index, self.index_in_uint, self.working_uint, self.nbr_ones_in_uint);
        println!("index: {}", 8*size_of::<UINT>()*self.uint_index + self.index_in_uint);
         */
        let one_left: UINT = UINT::one() << (8*size_of::<UINT>() - 1);
        loop {
            
            if (self.working_uint & one_left) == one_left {
                self.nbr_ones_in_uint -= 1;

                let res: Option<usize> = Some(8*size_of::<UINT>()*self.uint_index + self.index_in_uint);
                if res.unwrap() >= self.bitset.size() {
                    return None;
                }

                if self.nbr_ones_in_uint == 0 {
                    self.index_in_uint = 0; // returns to the start of the next uint
                }
                else {
                    self.working_uint = self.working_uint << 1; // skips to the next bit
                    self.index_in_uint += 1;
                }
                return res;
            }            
            else {
                self.working_uint = self.working_uint << 1;
                self.index_in_uint += 1;
            }
        }

    }

}


#[cfg(test)]
mod tests{
    use crate::datastructures::bitset::{BitSet, BitSetIter, MutBitSetIter};

    #[test]
    fn test1(){
        let mut bitset1:BitSet<u8>  = BitSet::new_empty(10);
        let mut bitset2: BitSet<u8> = BitSet::new_empty(10);


        for i in 0..bitset1.nbr_uints() {
            let rand_value: u8 = ((197 + i*157)%255) as u8 & 0b01101001;
            bitset1.data[i as usize] = rand_value;
        }
        bitset1.data[1] = bitset1.data[1] & 0b11000000;

        

        for i in 0..bitset2.nbr_uints (){
            let rand_value: u8 = ((100 + i*37)%255) as u8 & 0b11001011;
            bitset2.data[i as usize] = rand_value;
        }
        bitset2.data[1] = bitset2.data[1] & 0b11000000;

        bitset1.print();
        bitset2.print();

        bitset1.insert(3);
        bitset1.print();

        (bitset1.union(&bitset2)).print();
        (bitset1.intersection(&bitset2)).print();
        (bitset1.difference(&bitset2)).print();
        (bitset1.symmetric_difference(&bitset2)).print();

        bitset1.print();
        bitset2.print();

        for (i, value) in (&bitset1).into_iter().enumerate(){
            println!("i: {}, value: {}", i, value);
        }

        (bitset1.concatenate(&bitset2)).print();
        bitset1.print();

        let mut it: MutBitSetIter<u8> = (&mut bitset1).into_iter();

        for i in 0..5 {
            it.bitset.insert(i);
            while let Some(symb) = it.next() {
                println!("value youpi: {}", symb);
            }
            it.reset();
        }
        
    }
}




