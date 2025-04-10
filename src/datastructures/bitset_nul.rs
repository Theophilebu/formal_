use std::ops::Deref;


pub struct ContiguousList<'t, 'f, T, F>
where 
    T: PartialEq,
    F: Fn(&T) -> Option<u16>,
    't: 'f,
{
    values: Vec<&'t T>,  // max size: 2^16
    value_id: Option<&'f F>,
}

impl <'t, 'f, T, F> ContiguousList<'t, 'f, T, F>
where 
    T: PartialEq,
    F: Fn(&T) -> Option<u16>,
    't: 'f,
{
    pub fn new(values: Vec<&'t T>, value_id: Option<&'f F>) -> ContiguousList<'t, 'f, T, F> {
        ContiguousList { values, value_id }
    }

    pub fn get_value(&self, index: u16) -> &'t T{
        self.values[index as usize]
    }

    pub fn get_index(&self, value: &T) -> Option<u16>{

        match self.value_id{
            Some(f) => f(value),
            None => self.values
                        .iter()
                        .position(|x: &&T| *x == value)
                        .map(|x| (x as u16)),
            
        }
    }

    pub fn len(&self) -> u16 {
        return self.values.len() as u16;
        
    }

    pub fn iter(&self) -> std::slice::Iter<'_, &T>{
        self.values.iter()
    }

}


impl <'t, 'f, T, F> IntoIterator for &ContiguousList<'t, 'f, T, F>
where 
    T: PartialEq,
    F: Fn(&T) -> Option<u16>,
    't: 'f,
{
    type Item = &'t T;
    type IntoIter = std::iter::Map<std::slice::Iter<'t, &'t T>, fn (&&'t T) -> &'t T>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.iter().map(deref)   // |x: &&'t T| *x
    }
}

fn deref<'t,T>(x: &&'t T) -> &'t T{
    *x
}

fn f(x: &&u8){
    deref(x);
}

pub struct BitSet<'list, 'symbol, 'f, SYMBOL, F>
where
    SYMBOL: PartialEq,
    F: Fn(&SYMBOL) -> Option<u16>,
    'symbol: 'list,
    'symbol: 'f,
{
    bytes: Vec<u8>,
    list: &'list ContiguousList<'symbol, 'f, SYMBOL, F>,
}



impl <'se, 'list, 'symbol, 'f, SYMBOL, F> BitSet<'list, 'symbol, 'f, SYMBOL, F>
where
    SYMBOL: PartialEq,
    F: Fn(&SYMBOL) -> Option<u16>,
    'symbol: 'list,
    'symbol: 'f,
{
    //type BS = BitSet<'list, ContiguousList<'symbol, 'f, SYMBOL, F>>;(

    pub fn size(&self) -> u16 {
        self.list.len()
    }

    pub fn nbr_bytes(&self) -> u16 {
        self.bytes.len() as u16
    }


    pub fn new(contiguous_list: &'list ContiguousList<'symbol, 'f, SYMBOL, F>) 
    -> Self {
        let mut nbr_bytes: u16 = contiguous_list.len()/8;
        if 8*nbr_bytes < contiguous_list.len(){
            nbr_bytes+=1;
        }
        BitSet{ 
            bytes: vec![u8::MIN; nbr_bytes as usize], 
            list: contiguous_list,
        }
    }

    pub fn new_full(contiguous_list: &'list ContiguousList<'symbol, 'f, SYMBOL, F>) 
    -> Self {
        let mut nbr_bytes: u16 = contiguous_list.len()/8;
        if 8*nbr_bytes < contiguous_list.len(){
            nbr_bytes+=1;
        }
        BitSet{ 
            bytes: vec![u8::MAX; nbr_bytes as usize], 
            list: contiguous_list,
        }
    }


    pub fn contains(&self, n: u16) -> bool{
        if n>=self.size(){
            panic!("index out of bounds");
        }
        let byte_index = n/8;
        let position_in_byte = n - 8*byte_index;
        let bit_mask: u8 = 1<<position_in_byte;
        return (bit_mask & self.bytes[byte_index as usize])>0;
    }

    pub fn insert(&mut self, n: u16){
        if n>=self.size(){
            panic!("index out of bounds");
        }

        let byte_index = n/8;
        let position_in_byte = n - 8*byte_index;
        let bit_mask: u8 = 1<<position_in_byte;
        self.bytes[byte_index as usize] = bit_mask | self.bytes[byte_index as usize];
    }

    pub fn remove(&mut self, n: u16){
        if n>=self.size(){
            panic!("index out of bounds");
        }
        let byte_index = n/8;
        let position_in_byte = n - 8*byte_index;
        let bit_mask: u8 = 1<<position_in_byte;
        self.bytes[byte_index as usize] = !bit_mask & self.bytes[byte_index as usize];
    }

    pub fn set(&mut self, n: u16, value: bool){

        if n>=self.size(){
            panic!("index out of bounds");
        }
        if value{
            self.insert(n);
        } else{
            self.remove(n);
        }
    }


    pub fn same_list(&self, other: &Self) -> bool{
        std::ptr::eq(self.list, other.list)
    }
    
    pub fn union(&self, other: &Self) 
    -> Self{
        if !self.same_list(other){
            panic!("operations on bitsets with different lists are not allowed");
        }

        let mut new_bytes: Vec<u8> = Vec::new();
        new_bytes.reserve(self.nbr_bytes() as usize);

        for i in 0..self.nbr_bytes(){
            new_bytes.push(self.bytes[i as usize] | other.bytes[i as usize]);
        }

        BitSet {
            bytes: new_bytes,
            list: self.list,
        }
    }

    pub fn intersection(&self, other: &Self) 
    -> Self{
        if !self.same_list(other){
            panic!("operations on bitsets with different lists are not allowed");
        }

        let mut new_bytes: Vec<u8> = Vec::new();
        new_bytes.reserve(self.nbr_bytes() as usize);

        for i in 0..self.nbr_bytes(){
            new_bytes.push(self.bytes[i as usize] & other.bytes[i as usize]);
        }

        BitSet {
            bytes: new_bytes,
            list: self.list,
        }
    }

    pub fn difference(&self, other: &Self) 
    -> Self{
        if !self.same_list(other){
            panic!("operations on bitsets with different lists are not allowed");
        }

        let mut new_bytes: Vec<u8> = Vec::new();
        new_bytes.reserve(self.nbr_bytes() as usize);

        for i in 0..self.nbr_bytes(){
            new_bytes.push(self.bytes[i as usize] & !other.bytes[i as usize]);
        }

        BitSet {
            bytes: new_bytes,
            list: self.list,
        }
    }

    pub fn symmetric_difference(&self, other: &Self) 
    -> Self{
        if !self.same_list(other){
            panic!("operations on bitsets with different lists are not allowed");
        }

        let mut new_bytes: Vec<u8> = Vec::new();
        new_bytes.reserve(self.nbr_bytes() as usize);

        for i in 0..self.nbr_bytes(){
            new_bytes.push(self.bytes[i as usize] ^ other.bytes[i as usize]);
        }

        BitSet {
            bytes: new_bytes,
            list: self.list,
        }
    }


    pub fn update_union(&mut self, other: &Self) {
        if !self.same_list(other){
            panic!("operations on bitsets with different lists are not allowed");
        }

        for i in 0..self.nbr_bytes(){
            self.bytes[i as usize] = self.bytes[i as usize] | other.bytes[i as usize];
        }
    }

    pub fn update_intersection(&mut self, other: &Self) {
        if !self.same_list(other){
            panic!("operations on bitsets with different lists are not allowed");
        }

        for i in 0..self.nbr_bytes(){
            self.bytes[i as usize] = self.bytes[i as usize] & other.bytes[i as usize];
        }
    }

    pub fn update_difference(&mut self, other: &Self) {
        if !self.same_list(other){
            panic!("operations on bitsets with different lists are not allowed");
        }

        for i in 0..self.nbr_bytes(){
            self.bytes[i as usize] = self.bytes[i as usize] & !other.bytes[i as usize];
        }
    }

    pub fn update_symmetric_difference(&mut self, other: &Self) {
        if !self.same_list(other){
            panic!("operations on bitsets with different lists are not allowed");
        }

        for i in 0..self.nbr_bytes(){
            self.bytes[i as usize] = self.bytes[i as usize] ^ other.bytes[i as usize];
        }
    }


    pub fn clear(&mut self){
        for i in 0..self.nbr_bytes(){
            self.bytes[i as usize] = u8::MIN;
        }
    }

    pub fn fill(&mut self){
        for i in 0..self.nbr_bytes(){
            self.bytes[i as usize] = u8::MAX;
        }
    }


    pub fn is_disjoint(&self, other: &Self) -> bool{
        if !self.same_list(other){
            panic!("operations on bitsets with different lists are not allowed");
        }

        for i in 0..self.nbr_bytes(){
            if self.bytes[i as usize] & other.bytes[i as usize] > 0 {
                return false;
            }
        }
        return true;
    }

    pub fn is_empty(&self) -> bool{
        for byte in &self.bytes{
            if *byte>0{
                return false;
            }
        }
        true
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        for i in 0..self.nbr_bytes(){
            if self.bytes[i as usize] & !other.bytes[i as usize] > 0 {
                return false;
            }
        }
        true
    }

    pub fn is_superset(&self, other: &Self) -> bool {
        for i in 0..self.nbr_bytes(){
            if !self.bytes[i as usize] & other.bytes[i as usize] > 0 {
                return false;
            }
        }
        true
    }


    pub fn len(&self) -> usize{
        let mut l: usize = 0;
        for byte in &self.bytes{
            l+=byte.count_ones() as usize;
        }
        l
    }


    pub fn print(&self){
        let mut s: String = String::from("");
        for byte in &self.bytes{
            let byte_str: String = format!("{byte:b}");
            // total starts with 0 bits
            let mut total: String = std::iter::repeat("0")
                .take(8-byte_str.len())
                .collect();

            total.push_str(&byte_str);

            // reverse string
            total = total
                .chars()
                .rev()
                .collect();
            s.push_str(&total);
        }
        println!("{s}");
    }


    pub fn iter(&'se self) -> BitSetIter<'se, 'list, 'symbol, 'f, SYMBOL, F> {
        BitSetIter::new(self)
    }

}


impl <'list, 'symbol, 'f, SYMBOL, F> Clone for BitSet<'list, 'symbol, 'f, SYMBOL, F>
where
    SYMBOL: PartialEq,
    F: Fn(&SYMBOL) -> Option<u16>,
    'symbol: 'list,
    'symbol: 'f,
{
    fn clone(&self) -> Self {
        BitSet {
            bytes: self.bytes.clone(),
            list: self.list,
        }
    }
}

impl <'bitset, 'list, 'symbol, 'f, SYMBOL, F> IntoIterator for &'bitset BitSet<'list, 'symbol, 'f, SYMBOL, F>
where
    SYMBOL: PartialEq,
    F: Fn(&SYMBOL) -> Option<u16>,
    'symbol: 'list,
    'symbol: 'f,
    'list: 'bitset,
{
    type Item = &'symbol SYMBOL;
    type IntoIter = BitSetIter<'bitset, 'list, 'symbol, 'f, SYMBOL, F>;

    fn into_iter(self) -> Self::IntoIter {
        BitSetIter::new(self)
    }
}
// ------------------------------------------------------------

pub struct BitSetIter<'bitset, 'list, 'symbol, 'f, SYMBOL, F>
where
    SYMBOL: PartialEq,
    F: Fn(&SYMBOL) -> Option<u16>,
    'symbol: 'list,
    'symbol: 'f,
    'list: 'bitset,
{
    bitset: &'bitset BitSet<'list, 'symbol, 'f, SYMBOL, F>,

    byte_index: u16,
    index_in_byte: u8,
    
    current_byte: u8,       // not really the actual byte, it gets shifted
    nbr_ones_in_byte: u8,
}

impl <'bitset, 'list, 'symbol, 'f, SYMBOL, F> BitSetIter<'bitset, 'list, 'symbol, 'f, SYMBOL, F>
where
    SYMBOL: PartialEq,
    F: Fn(&SYMBOL) -> Option<u16>,
    'symbol: 'list,
    'symbol: 'f,
    'list: 'bitset,
{

    pub fn new(bitset: &'bitset BitSet<'list, 'symbol, 'f, SYMBOL, F>) -> BitSetIter<'bitset, 'list, 'symbol, 'f, SYMBOL, F> {
        BitSetIter{
            bitset,
            byte_index: 0,
            index_in_byte: 0,
            current_byte: bitset.bytes[0],
            nbr_ones_in_byte: bitset.bytes[0].count_ones() as u8,
        }
    }

}


impl <'bitset, 'list, 'symbol, 'f, SYMBOL, F> Iterator for BitSetIter<'bitset, 'list, 'symbol, 'f, SYMBOL, F>
where
    SYMBOL: PartialEq,
    F: Fn(&SYMBOL) -> Option<u16>,
    'symbol: 'list,
    'symbol: 'f,
    'list: 'bitset,
{
    type Item = &'symbol SYMBOL;

    fn next(&mut self) -> Option<&'symbol SYMBOL> {


        println!("byte_index: {}, index_in_byte: {}, current_byte: {}, nbr_ones_in_byte: {}",
        self.byte_index, self.index_in_byte, self.current_byte, self.nbr_ones_in_byte);


        // finds a non-empty byte, or returns None if the end is reached without encountering any zeros
        while self.nbr_ones_in_byte == 0 {
            
            self.byte_index = self.byte_index + 1;
            if self.byte_index == self.bitset.nbr_bytes(){
                return None;
            }
            
            self.current_byte = self.bitset.bytes[self.byte_index as usize];
        }
        
        

        let mut res: Option<&SYMBOL> = None;
        loop {
            
            if (self.current_byte & 1)==1 {
                res = Some(
                    self.bitset.list.get_value(
                    8*self.byte_index + self.index_in_byte as u16)
                );

            }

            self.current_byte >>= 1;
            self.index_in_byte += 1;
            

            if let Some(_) = res{
                
                if self.nbr_ones_in_byte == 0 {
                    self.index_in_byte = 0;
                }
                self.nbr_ones_in_byte -= 1;

                return res;
            }
        }

    }

}



// -------------------------- Owned bitset iterator --------------------

/*
pub struct OwnedBitSetIter<UINT: Unsigned + PrimInt + Binary>
{
    bitset: BitSet<UINT>,

    uint_index: usize,
    index_in_uint: usize,
    
    working_uint: UINT,       // not really the actual uint, it gets modified
    nbr_ones_in_uint: u32,    // same, it gets decremented
}


impl <UINT: Unsigned + PrimInt + Binary> OwnedBitSetIter<UINT>
{

    pub fn new(bitset: BitSet<UINT>) -> OwnedBitSetIter<UINT> {
        // gets some values before bitset is moved
        let working_uint: UINT = bitset.data[0];
        let nbr_ones_in_uint: u32 = bitset.data[0].count_ones();
        OwnedBitSetIter{
            bitset,

            uint_index: 0,
            index_in_uint: 0,

            working_uint,
            nbr_ones_in_uint,
        }
    }

    pub fn get_bitset(&mut self) -> &mut BitSet<UINT>{
        &mut self.bitset
    }

}




impl <UINT: Unsigned + PrimInt + Binary> IntoIterator for BitSet<UINT>
{
    type Item = usize;
    type IntoIter = OwnedBitSetIter<UINT>;

    fn into_iter(self) -> Self::IntoIter {
        OwnedBitSetIter::new(self)
    }
}

impl <UINT: Unsigned + PrimInt + Binary> Iterator for OwnedBitSetIter<UINT>
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

*/

// -------------------------- cycle Iterator ---------------------------
/*

pub struct BitSetCyclicIter<'bitset, UINT: Unsigned + PrimInt + Binary>
{
    bitset: &'bitset BitSet<UINT>,

    last_non_empty_uint_index: usize,

    uint_index: usize,
    index_in_uint: usize,
    
    working_uint: UINT,       // not really the actual uint, it gets modified
    nbr_ones_in_uint: u32,    // same, it gets decremented
}

impl <'bitset, UINT: Unsigned + PrimInt + Binary> BitSetCyclicIter<'bitset, UINT>
{

    pub fn new(bitset: &'bitset BitSet<UINT>) -> BitSetCyclicIter<'bitset, UINT> {
        BitSetCyclicIter{
            bitset,

            last_non_empty_uint_index: 0,

            uint_index: 0,
            index_in_uint: 0,

            working_uint: bitset.data[0],
            nbr_ones_in_uint: bitset.data[0].count_ones(),
        }
    }

}

impl <'bitset, UINT: Unsigned + PrimInt + Binary> Iterator for BitSetCyclicIter<'bitset, UINT>
{
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        
        // each iteration of this loop is a traversal(complete or uncomplete) of the bitset
        // this loops twice only if the end of the bitset is reached before a return
        // it can't loop more than twice because it would imply that the second traversal found no element,
        // in which case we should return None
        loop {
            // loop through the data
            while self.nbr_ones_in_uint == 0 {
                // println!("cherche uint");
                
                self.uint_index += 1;
                if self.uint_index == self.bitset.nbr_uints() {
                    self.uint_index = 0;
                }
                
                self.working_uint = self.bitset.data[self.uint_index];
                self.nbr_ones_in_uint = self.working_uint.count_ones();
    
                if (self.nbr_ones_in_uint > 0) {
                    self.last_non_empty_uint_index = self.uint_index;
                }
                else if self.last_non_empty_uint_index == self.uint_index {
                    return None;
                }
            }

            /*
            println!("uint_index: {}, index_in_uint: {}, working_uint: {:b}, nbr_ones_in_uint: {}",
            self.uint_index, self.index_in_uint, self.working_uint, self.nbr_ones_in_uint);
            println!("index: {}", 8*size_of::<UINT>()*self.uint_index + self.index_in_uint);
            */

            let one_left: UINT = UINT::one() << (8*size_of::<UINT>() - 1);
            // loops through the uint
            loop {
                
                if (self.working_uint & one_left) == one_left {
                    self.nbr_ones_in_uint -= 1;

                    let res: Option<usize> = Some(8*size_of::<UINT>()*self.uint_index + self.index_in_uint);
                    if res.unwrap() >= self.bitset.size() {
                        // end of the bitset, goes back to another loop through the bitset
                        
                        self.uint_index = 0;
                        self.index_in_uint = 0;

                        self.working_uint = self.bitset.data[0];
                        self.nbr_ones_in_uint = self.bitset.data[0].count_ones();

                        break;
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

}
*/
// ---------------------------------------------------------





#[cfg(test)]
mod tests{
    use crate::{bitset::{BitSet, BitSetIter}, symbol::Symbol};

    use super::ContiguousList;

    #[test]
    fn test1(){
        let symbols: [Symbol; 4] = [Symbol{id: 0}, Symbol{id: 1}, Symbol{id: 2}, Symbol{id: 3}];

        let list = ContiguousList::new(
            symbols.iter().collect(),
            Some(&(
                |s: &Symbol| (Some(s.id)))
            )
        );


        let mut bitset1 = BitSet::new(&list);
        let mut bitset2 = BitSet::new(&list);

        println!("ptr eq: {}", std::ptr::eq(bitset1.list, bitset2.list));

        for i in 0..bitset1.nbr_bytes() {
            let rand_value: u8 = ((197 + i*157)%255) as u8 & 0b00001111;
            bitset1.bytes[i as usize] = rand_value;
        }

        

        for i in 0..bitset2.nbr_bytes (){
            let rand_value: u8 = ((100 + i*37)%255) as u8 & 0b00001111;
            bitset2.bytes[i as usize] = rand_value;
        }

        bitset1.print();
        bitset2.print();

        bitset1.insert(3);
        bitset1.print();

        (bitset1.union(&bitset2)).print();
        (bitset1.intersection(&bitset2)).print();
        (bitset1.difference(&bitset2)).print();
        (bitset1.symmetric_difference(&bitset2)).print();

        for (i, symb) in bitset1.into_iter().enumerate(){
            println!("symb: {}", symb.id);
        }
        
    }
}

