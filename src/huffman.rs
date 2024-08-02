//! Deal with Huffman encoding and decoding.
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

    /// Create a fixed literal/length table.
    /// Defined in RFC 1951, section 3.2.6.
    pub fn fixed_literal_table() -> Self {
        let mut code_len = vec![0; 288];
        (0..144).for_each(|i| code_len[i] = 8);
        (144..256).for_each(|i| code_len[i] = 9);
        (256..280).for_each(|i| code_len[i] = 7);
        (280..288).for_each(|i| code_len[i] = 8);
        Self::new(&code_len, 9)
    }

    /// Create a fixed distance table.
    /// Defined in RFC 1951, section 3.2.6.
    pub fn fixed_distance_table() -> Self {
        let code_len = vec![5; 32];
        Self::new(&code_len, 5)
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

    #[test]
    fn test_fixed_literal_table() {
        let huffman_table = HuffmanLookupTable::fixed_literal_table();
        assert_eq!(huffman_table.max_bits, 9);
        assert_eq!(huffman_table.table[0b0_01111111], (252, 9));
        assert_eq!(huffman_table.table[0b1_10000000], (256, 7));
        assert_eq!(huffman_table.table[0b1_00000011], (280, 8));
        assert_eq!(huffman_table.table[0b0_00001100], (0, 8));
        assert_eq!(huffman_table.table[0b1_11111111], (255, 9));
    }

    #[test]
    fn test_fixed_distance_table() {
        let huffman_table = HuffmanLookupTable::fixed_distance_table();
        assert_eq!(huffman_table.max_bits, 5);
        assert_eq!(huffman_table.table[0b00000], (0, 5));
        assert_eq!(huffman_table.table[0b11100], (7, 5));
        assert_eq!(huffman_table.table[0b11111], (31, 5));
    }

}
