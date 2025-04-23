#[cfg(feature = "serde_support")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Bits(#[cfg_attr(feature = "serde_support", serde(with = "serde_bytes"))] Vec<u8>);
impl Bits {
    pub fn new(size_hint: usize) -> Self {
        Bits(vec![0; size_hint.div_ceil(8)])
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len() * 8
    }

    #[inline]
    pub fn get_uint(&self, position: usize, size: usize) -> u64 {
        let mut value = 0;
        let start = position / 8;
        let end = (position + size).div_ceil(8);
        for (i, &b) in self.0[start..end].iter().enumerate() {
            value |= u64::from(b) << (i * 8);
        }

        let offset = position % 8;
        let mask = (1 << size) - 1;
        (value >> offset) & mask
    }

    #[inline]
    pub fn set_uint(&mut self, position: usize, mut size: usize, mut value: u64) {
        let mut offset = position % 8;
        for b in &mut self.0[position / 8..] {
            let high = ((u64::from(*b) >> (size + offset)) << (size + offset)) as u8;
            let middle = (value << offset) as u8;
            let low = *b & ((1 << offset) - 1);
            *b = high | middle | low;

            let drop_bits = 8 - offset;
            if size <= drop_bits {
                break;
            }
            size -= drop_bits;
            value >>= drop_bits;
            offset = 0;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let mut bits = Bits::new(12345);
        assert_eq!(bits.len(), 12352);

        assert_eq!(bits.get_uint(0, 1), 0);
        bits.set_uint(0, 1, 1);
        assert_eq!(bits.get_uint(0, 1), 1);

        assert_eq!(bits.get_uint(333, 10), 0);
        bits.set_uint(333, 10, 0b10_1101_0001);
        assert_eq!(bits.get_uint(333, 10), 0b10_1101_0001);

        assert_eq!(bits.get_uint(335, 4), 0b0100);
        bits.set_uint(335, 4, 0b1010);
        assert_eq!(bits.get_uint(335, 4), 0b1010);
        assert_eq!(bits.get_uint(333, 10), 0b10_1110_1001);
    }

    #[test]
    fn test_high_bits() {
        let mut bits = Bits::new(320);
        assert_eq!(bits.len(), 320);

        assert_eq!(bits.get_uint(290, 5), 0);
        bits.set_uint(290, 5, 31);
        assert_eq!(bits.get_uint(290, 5), 31);
        bits.set_uint(290, 5, 21);
        assert_eq!(bits.get_uint(290, 5), 21);
    }
}
