#![allow(unused)]

use crate::util::{div_ceil, process_pairs};

#[derive(Debug, Clone, PartialEq)]
pub struct FixedBits {
    data: Box<[usize]>,
    bits: usize,
}

const USIZE_BYTES: usize = std::mem::size_of::<usize>();
const USIZE_BITS: usize = USIZE_BYTES * 8;

impl FixedBits {
    pub fn new(bits: usize) -> Self {
        Self {
            data: vec![0; div_ceil(bits, USIZE_BYTES)].into_boxed_slice(),
            bits,
        }
    }

    pub fn from_slice<T>(slice: &[T], f: impl Fn(&T) -> bool) -> Self {
        let mut bits = Self::new(slice.len());

        for (chunk, dest) in slice.chunks(USIZE_BITS).zip(bits.data.iter_mut()) {
            let mut v = 0;
            for (j, b) in chunk.iter().enumerate() {
                v |= (f(b) as usize) << j;
            }
            *dest = v;
        }

        bits
    }

    pub fn len(&self) -> usize {
        self.bits
    }
    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }

    fn fix_tail(&mut self) {
        let tail_len = self.bits % USIZE_BITS;
        if tail_len > 0 {
            let mask = (1 << tail_len) - 1;
            *self.data.last_mut().unwrap() &= mask;
        }
    }

    pub fn get(&self, index: usize) -> Option<bool> {
        if index < self.len() {
            let part = self.data[index / USIZE_BITS];
            let mask = 1 << (index % USIZE_BITS);
            Some((part & mask) != 0)
        } else {
            None
        }
    }
    pub fn set(&mut self, index: usize, value: bool) {
        assert!(index < self.len());
        let part = &mut self.data[index / USIZE_BITS];
        let mask = 1 << (index % USIZE_BITS);
        *part = if value { *part | mask } else { *part & !mask };
    }

    pub fn fill(&mut self, value: bool) {
        let fill = if value { usize::MAX } else { 0 };
        self.data.fill(fill);
        self.fix_tail();
    }

    pub fn and(&mut self, other: &Self) {
        assert_eq!(self.bits, other.bits);
        for (a, b) in self.data.iter_mut().zip(other.data.iter()) {
            *a &= b;
        }
    }
    pub fn or(&mut self, other: &Self) {
        assert_eq!(self.bits, other.bits);
        for (a, b) in self.data.iter_mut().zip(other.data.iter()) {
            *a |= b;
        }
    }

    pub fn expand_one(&mut self) {
        for part in self.data.iter_mut() {
            *part |= (*part >> 1) | (*part << 1)
        }
        process_pairs(self.data.iter_mut(), |a, b| {
            *a |= *b << (USIZE_BITS - 1);
            *b |= *a >> (USIZE_BITS - 1);
        });
        self.fix_tail();
    }
}

impl FromIterator<bool> for FixedBits {
    fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
        let iter = iter.into_iter();

        let mut v = Vec::with_capacity(iter.size_hint().0);

        let mut current = 0;
        let mut current_bit = 0;
        for b in iter {
            current |= (b as usize) << current_bit;
            current_bit = (current_bit + 1) % USIZE_BITS;
            if current_bit == 0 {
                v.push(current);
                current = 0;
            }
        }

        let mut bits = v.len() * USIZE_BITS;
        if current_bit != 0 {
            v.push(current);
            bits += current_bit;
        }

        Self {
            data: v.into_boxed_slice(),
            bits,
        }
    }
}
