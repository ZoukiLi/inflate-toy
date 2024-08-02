//! A module that provides a struct to read bits from a byte array.
use std::{default, ops};

/// A struct representing the position of a bit in a byte array.
/// The position is represented by the byte index and the bit index within the byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BitPosition {
    pub byte_index: usize,
    pub bit_index: usize,
}

/// The number of bits in a byte.
const BITS_PER_BYTE: usize = 8;

impl BitPosition {
    /// Create a new BitPosition at the beginning of the byte array.
    fn new() -> Self {
        Self {
            byte_index: 0,
            bit_index: 0,
        }
    }

    /// Try to add the given number of bits to the current position.
    fn try_add_bits(&self, bits: usize) -> Option<Self> {
        let byte_added = bits / BITS_PER_BYTE;
        let bit_rem = bits % BITS_PER_BYTE;

        let byte_index = self.byte_index.checked_add(byte_added)?;
        let bit_index = self.bit_index + bit_rem;

        if bit_index >= BITS_PER_BYTE {
            Some(Self {
                byte_index: byte_index.checked_add(1)?,
                bit_index: bit_index - BITS_PER_BYTE,
            })
        } else {
            Some(Self {
                byte_index,
                bit_index,
            })
        }
    }

    /// Add the given number of bits to the current position.
    fn add_bits(&self, bits: usize) -> Self {
        self.try_add_bits(bits).unwrap()
    }

    /// Try to add the position of another BitPosition to the current position.
    fn try_add_inner(&self, other: &Self) -> Option<Self> {
        let bits_added = self.try_add_bits(other.bit_index)?;
        Some(Self {
            byte_index: bits_added.byte_index.checked_add(other.byte_index)?,
            bit_index: bits_added.bit_index,
        })
    }

    /// Add the position of another BitPosition to the current position.
    fn add_inner(&self, other: &Self) -> Self {
        self.try_add_inner(other).unwrap()
    }
}

impl default::Default for BitPosition {
    fn default() -> Self {
        Self::new()
    }
}

impl From<usize> for BitPosition {
    fn from(value: usize) -> Self {
        Self::new().add_bits(value)
    }
}

impl ops::Add for BitPosition {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        self.add_inner(&other)
    }
}

impl ops::AddAssign for BitPosition {
    fn add_assign(&mut self, other: Self) {
        *self = self.add_inner(&other)
    }
}

/// A struct that reads bits from a byte array.
#[derive(Debug)]
pub struct BitReader<'a> {
    data: &'a [u8],
    position: BitPosition,
    eof: bool,
}

impl<'a> BitReader<'a> {
    /// Create a new BitReader with the given byte array.
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            position: BitPosition::new(),
            eof: false,
        }
    }

    /// Check if the reader has reached the end of the data.
    pub fn eof(&self) -> bool {
        self.eof
    }

    /// Peek bits with given bit length without advancing the position.
    ///
    pub fn try_peek_bits(&self, n_bits: usize) -> Option<usize> {
        if n_bits == 0 {
            return Some(0);
        }
        // We can't read more than usize::BITS bits at once.
        if n_bits > usize::BITS as usize {
            return None;
        }

        let mut result = 0;
        let mut n_bits_rem = n_bits;

        let mut cur_pos = self.position;

        while n_bits_rem > 0 {
            // If we reach the end of the data, set the eof flag and return the result.
            if cur_pos.byte_index >= self.data.len() {
                return Some(result);
            }

            // Current byte to read bits from.
            let cur_byte = self.data[cur_pos.byte_index];
            // Number of bits to read from the current byte.
            // Max is the remaining bits in the current byte or the bits to read.
            let bits_to_read = n_bits_rem.min(BITS_PER_BYTE - cur_pos.bit_index);

            // Create a mask to extract the bits to read.
            // For example, if bits_to_read is 3, the mask will be 0b00000111.
            let mask = (1usize << bits_to_read) - 1;

            // Extract the bits from the current byte.
            let read_bits = (cur_byte as usize >> cur_pos.bit_index) & mask;

            // Shift the bits to the correct position in the result.
            result |= read_bits << (n_bits - n_bits_rem);

            // Update the remaining bits and the current bit index.
            n_bits_rem -= bits_to_read;
            cur_pos = cur_pos.add_bits(bits_to_read);
        }
        Some(result)
    }

    /// Try to advance the position by the given number of bits.
    pub fn try_advance(&mut self, n_bits: usize) -> Option<()> {
        if self.eof {
            return Some(());
        }

        let new_pos = self.position.try_add_bits(n_bits)?;
        if new_pos.byte_index >= self.data.len() {
            self.eof = true;
            self.position = BitPosition {
                byte_index: self.data.len(),
                bit_index: 0,
            };
        } else {
            self.position = new_pos;
        }
        Some(())
    }

    /// Advance the position by the given number of bits.
    pub fn advance(&mut self, n_bits: usize) {
        self.try_advance(n_bits).unwrap();
    }

    /// Try to advance the position to the next byte boundary.
    pub fn try_advance_to_byte_boundary(&mut self) -> Option<()> {
        if self.position.bit_index == 0 {
            return Some(());
        }
        let bits_to_boundary = BITS_PER_BYTE - self.position.bit_index;
        self.try_advance(bits_to_boundary)?;
        Some(())
    }

    /// Advance the position to the next byte boundary.
    pub fn advance_to_byte_boundary(&mut self) {
        self.try_advance_to_byte_boundary().unwrap();
    }

    /// Try to read the given number of bits and advance the position.
    pub fn try_read_bits(&mut self, bits: usize) -> Option<usize> {
        let result = self.try_peek_bits(bits)?;
        self.try_advance(bits)?;
        Some(result)
    }

    /// Peek bits with given bit length without advancing the position.
    pub fn peek_bits(&self, bits: usize) -> usize {
        self.try_peek_bits(bits).unwrap()
    }

    /// Read the given number of bits and advance the position.
    pub fn read_bits(&mut self, bits: usize) -> usize {
        self.try_read_bits(bits).unwrap()
    }

    /// Try to read a byte and advance the position.
    pub fn try_read_byte(&mut self) -> Option<u8> {
        let byte = self.try_read_bits(BITS_PER_BYTE)?;
        Some(byte as u8)
    }

    /// Read a byte and advance the position.
    pub fn read_byte(&mut self) -> u8 {
        self.try_read_byte().unwrap()
    }

    /// Try to read the given number of bytes and fill the buffer.
    /// Return the number of bytes read.
    pub fn try_read_bytes_to_slice(&mut self, n_bytes: usize, buf: &mut [u8]) -> Option<usize> {
        for byte in buf.iter_mut().take(n_bytes) {
            *byte = self.try_read_byte()?;
        }
        Some(n_bytes.min(buf.len()))
    }

    /// Read the given number of bytes and fill the buffer.
    /// Return the number of bytes read.
    pub fn read_bytes_to_slice(&mut self, n_bytes: usize, buf: &mut [u8]) -> usize {
        self.try_read_bytes_to_slice(n_bytes, buf).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_position_new() {
        let pos = BitPosition::new();
        assert_eq!(pos.byte_index, 0);
        assert_eq!(pos.bit_index, 0);
    }

    #[test]
    fn test_bit_position_add_bits() {
        let pos = BitPosition::new();
        let new_pos = pos.add_bits(10);
        assert_eq!(new_pos.byte_index, 1);
        assert_eq!(new_pos.bit_index, 2);
    }

    #[test]
    fn test_bit_position_try_add_bits_overflow() {
        let pos = BitPosition {
            byte_index: usize::MAX,
            bit_index: 7,
        };
        assert!(pos.try_add_bits(2).is_none());
    }

    #[test]
    fn test_bit_position_add_inner() {
        let pos1 = BitPosition::new().add_bits(10);
        let pos2 = BitPosition::new().add_bits(6);
        let new_pos = pos1.add_inner(&pos2);
        assert_eq!(new_pos.byte_index, 2);
        assert_eq!(new_pos.bit_index, 0);
    }

    #[test]
    fn test_bit_reader_new() {
        let data = [0xFF, 0x00];
        let reader = BitReader::new(&data);
        assert_eq!(reader.position.byte_index, 0);
        assert_eq!(reader.position.bit_index, 0);
        assert!(!reader.eof());
    }

    #[test]
    fn test_bit_reader_peek_bits() {
        let data = [0b10101100, 0b01010101];
        let reader = BitReader::new(&data);
        let bits = reader.peek_bits(4);
        assert_eq!(bits, 0b1100);
        let bits = reader.peek_bits(4); // Ensure the position hasn't changed
        assert_eq!(bits, 0b1100);
    }

    #[test]
    fn test_bit_reader_read_bits() {
        let data = [0b10101100, 0b01010101];
        let mut reader = BitReader::new(&data);
        let bits = reader.read_bits(4);
        assert_eq!(bits, 0b1100);
        let bits = reader.read_bits(4);
        assert_eq!(bits, 0b1010);
        let bits = reader.read_bits(8);
        assert_eq!(bits, 0b01010101);
    }

    #[test]
    fn test_bit_reader_read_bits_across_bytes() {
        let data = [0b10101100, 0b01010101];
        let mut reader = BitReader::new(&data);
        let bits = reader.read_bits(12);
        assert_eq!(bits, 0b010110101100);
    }

    #[test]
    fn test_bit_reader_eof() {
        let data = [0b10101100];
        let mut reader = BitReader::new(&data);
        let _ = reader.read_bits(7);
        assert!(!reader.eof());
        let _ = reader.read_bits(1);
        assert!(reader.eof());
    }

    #[test]
    fn test_bit_reader_peek_bits_not_enough_data() {
        let data = [0b10101100];
        let reader = BitReader::new(&data);
        let bits = reader.peek_bits(12); // Should not panic, return what is available
        assert_eq!(bits, 0b10101100);
    }

    #[test]
    fn test_bit_reader_read_bits_not_enough_data() {
        let data = [0b10101100];
        let mut reader = BitReader::new(&data);
        let bits = reader.read_bits(12); // Should not panic, return what is available
        assert_eq!(bits, 0b10101100);
        assert!(reader.eof());
    }

    #[test]
    #[should_panic]
    fn test_bit_reader_peek_bits_panic() {
        let data = [0b10101100];
        let reader = BitReader::new(&data);
        let _ = reader.peek_bits(65); // Should panic as we can't read more than 64 bits
    }

    #[test]
    #[should_panic]
    fn test_bit_reader_read_bits_panic() {
        let data = [0b10101100];
        let mut reader = BitReader::new(&data);
        let _ = reader.read_bits(65); // Should panic as we can't read more than 64 bits
    }
}
