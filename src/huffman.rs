//! Deal with Huffman encoding and decoding.
//! This mod focuses on the deflation-independent part of Huffman encoding and decoding.
//!

/// Huffman tree lookup table.
/// A lookup table is used to speed up the encoding and decoding process.
/// In this table, each code is mapped to a symbol and a code length.
///
/// Because the DEFLATE format uses a variable-length code, the code length is needed to determine.
///
/// The table size is 2^max_bits. The max_bits is the maximum code length in the Huffman tree.
/// For the use of lookup table, all index that has a suffix of one code will be filled with the same symbol.
/// That means, if the max bits is 8, and one code is 0b101, then the table[0b*****101]
/// will all be the same symbol that the code 0b101 represents.
/// This will make the lookup process faster.
///
#[derive(Debug, Clone)]
pub struct HuffmanLookupTable {
    pub table: Vec<(usize, u8)>,
    pub max_bits: u8,
}

impl HuffmanLookupTable {
    /// Create a new lookup table.
    ///
    /// # Arguments
    ///
    /// * `max_bits` - The maximum code length in the Huffman tree.
    /// * `table` - The lookup table.
    ///
    pub fn new(code_len: &[u8], max_bits: u8) -> Self {
        assert!(max_bits <= usize::BITS as u8);
        let mut table = vec![(0, 0); 1 << max_bits];

        // Count the number of codes for each code length.
        let mut bl_count = vec![0; max_bits as usize + 1];
        code_len.iter().for_each(|&len| bl_count[len as usize] += 1);

        // Find the numerical value of the smallest code for each code length.
        let mut next_code = vec![0usize; max_bits as usize + 2];
        let mut code = 0;
        bl_count.iter().enumerate().for_each(|(bits, &count)| {
            code = (code + count) << 1;
            next_code[bits + 1] = code;
        });

        // Fill the lookup table.
        code_len
            .iter()
            .enumerate()
            .filter(|(_, &len)| len != 0)
            .for_each(|(symbol, &len)| {
                let code = next_code[len as usize];
                next_code[len as usize] += 1;

                // code is len bits long, so there are max_bits - len bits left.
                let shift = max_bits - len;
                let start = code << shift;
                let end = start + (1 << shift);

                for i in start..end {
                    // Fill the table with the symbol and the code length.
                    // Huffman code is big-endian, so the code should be reversed.
                    let rev = i.reverse_bits();
                    // Get the leftmost max_bits bits.
                    let rev_left = rev >> (usize::BITS as u8 - max_bits);
                    table[rev_left] = (symbol, len);
                }
            });

        Self { table, max_bits }
    }

    pub fn get(&self, code: usize) -> Option<(usize, u8)> {
        // Only use the least significant max_bits bits.
        let mask = (1 << self.max_bits) - 1;
        let code = code & mask;
        self.table.get(code).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_huffman_lookup_table() {
        let code_lengths = vec![3, 3, 3, 3, 3, 2, 4, 4];
        let max_bits = 4;
        let huffman_table = HuffmanLookupTable::new(&code_lengths, max_bits);

        huffman_table.table.iter().for_each(|&(symbol, len)| {
            assert_eq!(len, code_lengths[symbol]);
        });
    }
}
