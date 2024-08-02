//! Decompress data compressed with the DEFLATE algorithm.
//! This module focuses on the decompression process.
//!
//! The DEFLATE algorithm is a combination of LZ77 and Huffman coding.
//! The decompression process is the reverse of the compression process.

use crate::bit_stream::BitReader;
use crate::huffman::HuffmanLookupTable;
use std::io::{Error, ErrorKind, Result};

// constant values for the DEFLATE algorithm

const BFINAL_LEN: usize = 1;
const BFINAL_VALUE: usize = 1;

const BTYPE_LEN: usize = 2;
const BTYPE_NO_COMPRESSION: usize = 0b00;
const BTYPE_FIXED_HUFFMAN: usize = 0b01;
const BTYPE_DYNAMIC_HUFFMAN: usize = 0b10;

const LEN_LEN: usize = 16;
const NLEN_LEN: usize = 16;

const LITERAL_CODE_BASE: usize = 0;
const LITERAL_CODE_MAX: usize = 255;
const END_BLOCK_CODE: usize = 256;
const LENGTH_CODE_BASE: usize = 257;
const LENGTH_CODE_MAX: usize = 285;

const HLIT_LEN: usize = 5;
const HLIT_BASE: usize = 257;
const HDIST_LEN: usize = 5;
const HDIST_BASE: usize = 1;
const HCLEN_LEN: usize = 4;
const HCLEN_BASE: usize = 4;

const DYN_ALPHABET_CODE_NUM: usize = 19;
const DYN_ALPHABET_CODE_LEN: usize = 3;
const DYN_ALPHABET_TABLE_MAX_BITS: u8 = 7;
const DYN_TABLE_MAX_BITS: u8 = 15;

/// Length code table for DEFLATE.
/// length_code_table[i] = (length_code, length_base, extra_bits)
const LENGTH_CODE_TABLE: &[(usize, usize, usize)] = &[
    (257, 3, 0),
    (258, 4, 0),
    (259, 5, 0),
    (260, 6, 0),
    (261, 7, 0),
    (262, 8, 0),
    (263, 9, 0),
    (264, 10, 0),
    (265, 11, 1),
    (266, 13, 1),
    (267, 15, 1),
    (268, 17, 1),
    (269, 19, 2),
    (270, 23, 2),
    (271, 27, 2),
    (272, 31, 2),
    (273, 35, 3),
    (274, 43, 3),
    (275, 51, 3),
    (276, 59, 3),
    (277, 67, 4),
    (278, 83, 4),
    (279, 99, 4),
    (280, 115, 4),
    (281, 131, 5),
    (282, 163, 5),
    (283, 195, 5),
    (284, 227, 5),
    (285, 258, 0),
];

/// Get the length of the repeated data by the length code.
/// This function reads the extra bits if needed.
/// Returns None if the code is invalid.
fn get_length_by_code(code: usize, bit_reader: &mut BitReader) -> Option<usize> {
    let (length_code, length_base, extra_bits) =
        LENGTH_CODE_TABLE.get(code - LENGTH_CODE_BASE).cloned()?;
    assert!(length_code == code);
    Some(length_base + bit_reader.read_bits(extra_bits))
}

/// Distance code table for DEFLATE.
/// distance_code_table[i] = (distance_code, distance_base, extra_bits)
const DISTANCE_CODE_TABLE: &[(usize, usize, usize)] = &[
    (0, 1, 0),
    (1, 2, 0),
    (2, 3, 0),
    (3, 4, 0),
    (4, 5, 1),
    (5, 7, 1),
    (6, 9, 2),
    (7, 13, 2),
    (8, 17, 3),
    (9, 25, 3),
    (10, 33, 4),
    (11, 49, 4),
    (12, 65, 5),
    (13, 97, 5),
    (14, 129, 6),
    (15, 193, 6),
    (16, 257, 7),
    (17, 385, 7),
    (18, 513, 8),
    (19, 769, 8),
    (20, 1025, 9),
    (21, 1537, 9),
    (22, 2049, 10),
    (23, 3073, 10),
    (24, 4097, 11),
    (25, 6145, 11),
    (26, 8193, 12),
    (27, 12289, 12),
    (28, 16385, 13),
    (29, 24577, 13),
];

/// Get the distance of the repeated data by the distance code.
/// This function reads the extra bits if needed.
/// Returns None if the code is invalid.
fn get_distance_by_code(code: usize, bit_reader: &mut BitReader) -> Option<usize> {
    let (distance_code, distance_base, extra_bits) = DISTANCE_CODE_TABLE.get(code).cloned()?;
    assert!(distance_code == code);
    Some(distance_base + bit_reader.read_bits(extra_bits))
}

/// Create a fixed literal/length table.
/// Defined in RFC 1951, section 3.2.6.
fn fixed_literal_table() -> HuffmanLookupTable {
    let mut code_len = vec![0; 288];
    (0..144).for_each(|i| code_len[i] = 8);
    (144..256).for_each(|i| code_len[i] = 9);
    (256..280).for_each(|i| code_len[i] = 7);
    (280..288).for_each(|i| code_len[i] = 8);
    HuffmanLookupTable::new(&code_len, 9)
}

/// Create a fixed distance table.
/// Defined in RFC 1951, section 3.2.6.
fn fixed_distance_table() -> HuffmanLookupTable {
    let code_len = vec![5; 32];
    HuffmanLookupTable::new(&code_len, 5)
}

/// Dynamic Huffman Tree code lengths alphabet order.
/// Defined in RFC 1951, section 3.2.7.
const DYNAMIC_HUFFMAN_TREE_ORDER: [usize; DYN_ALPHABET_CODE_NUM] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

/// Resolve one symbol from the Huffman table.
fn resolve_symbol(bit_reader: &mut BitReader, huffman_table: &HuffmanLookupTable) -> Option<usize> {
    let peek_code = bit_reader.try_peek_bits(huffman_table.max_bits as usize)?;
    let (symbol, len) = huffman_table.get(peek_code)?;
    bit_reader.try_advance(len as usize)?;
    Some(symbol)
}

/// Inflate block with literal and distance huffman tables.
/// Because a duplicated string reference may refer to a string in a previous block,
/// we need the whole output to be able to resolve the references.
/// Returns the number of bytes outputted.
fn inflate_compressed_block(
    bit_reader: &mut BitReader,
    output: &mut Vec<u8>,
    lit_tb: &HuffmanLookupTable,
    dis_tb: &HuffmanLookupTable,
) -> Result<usize> {
    let mut bytes_outputted = 0;
    loop {
        let symbol = resolve_symbol(bit_reader, lit_tb).ok_or_else(invalid_huffman_symbol)?;
        match symbol {
            END_BLOCK_CODE => {
                // End of block
                break;
            }
            LITERAL_CODE_BASE..=LITERAL_CODE_MAX => {
                // Literal
                output.push(symbol as u8);
                bytes_outputted += 1;
            }
            LENGTH_CODE_BASE..=LENGTH_CODE_MAX => {
                // Length
                // get the length of the repeated data
                let len =
                    get_length_by_code(symbol, bit_reader).ok_or_else(invalid_huffman_symbol)?;
                // the distance code
                let dist_code =
                    resolve_symbol(bit_reader, dis_tb).ok_or_else(invalid_huffman_symbol)?;
                // get the distance of the repeated data
                let dist = get_distance_by_code(dist_code, bit_reader)
                    .ok_or_else(invalid_huffman_symbol)?;
                // repeat the data
                bytes_outputted +=
                    repeat_with_overlap(output, dist, len).ok_or_else(invalid_huffman_symbol)?;
            }
            _ => Err(invalid_huffman_symbol())?,
        }
    }
    Ok(bytes_outputted)
}

/// Deal with reapeted data in the output.
fn repeat_with_overlap(output: &mut Vec<u8>, dist: usize, len: usize) -> Option<usize> {
    let mut bytes_out = 0usize;
    for _ in 0..len {
        let read_pos = output.len() - dist;
        if let Some(byte) = output.get(read_pos) {
            output.push(*byte);
            bytes_out += 1;
        } else {
            break;
        }
    }
    Some(bytes_out)
}

/// For the sake of simplicity, we use the io::Error type for all errors.
/// Invalid Huffman symbol error.
fn invalid_huffman_symbol() -> Error {
    Error::new(ErrorKind::InvalidData, "Invalid Huffman symbol")
}

/// For the sake of simplicity, we use the io::Error type for all errors.
/// Invalid LEN and NLEN error.
fn invalid_len_nlen() -> Error {
    Error::new(ErrorKind::InvalidData, "Invalid LEN and NLEN")
}

/// For the sake of simplicity, we use the io::Error type for all errors.
/// Invalid block type error.
fn invalid_block_type() -> Error {
    Error::new(ErrorKind::InvalidData, "Invalid block type")
}

/// Read dynamic Huffman tables.
/// Returns a tuple of (literal table, distance table).
/// Defined in RFC 1951, section 3.2.7.
fn read_dynamic_huffman_tables(
    bit_reader: &mut BitReader,
) -> Result<(HuffmanLookupTable, HuffmanLookupTable)> {
    let hlit = bit_reader.read_bits(HLIT_LEN) + HLIT_BASE;
    let hdist = bit_reader.read_bits(HDIST_LEN) + HDIST_BASE;
    let hclen = bit_reader.read_bits(HCLEN_LEN) + HCLEN_BASE;
    let mut alphabet_code_len = vec![0; DYN_ALPHABET_CODE_NUM];
    (0..hclen).for_each(|i| {
        alphabet_code_len[DYNAMIC_HUFFMAN_TREE_ORDER[i]] =
            bit_reader.read_bits(DYN_ALPHABET_CODE_LEN) as u8;
    });
    let alphabet_code_len_table =
        HuffmanLookupTable::new(&alphabet_code_len, DYN_ALPHABET_TABLE_MAX_BITS);

    let lit_code_len = read_code_lengths(bit_reader, &alphabet_code_len_table, hlit)?;
    let dis_code_len = read_code_lengths(bit_reader, &alphabet_code_len_table, hdist)?;

    let lit_tb = HuffmanLookupTable::new(&lit_code_len, DYN_TABLE_MAX_BITS);
    let dis_tb = HuffmanLookupTable::new(&dis_code_len, DYN_TABLE_MAX_BITS);

    Ok((lit_tb, dis_tb))
}

/// Read code lengths using the alphabet code length table.
/// Returns a vector of code lengths.
/// Defined in RFC 1951, section 3.2.7.
/// The code lengths are used to create the dynamic Huffman tables.
fn read_code_lengths(
    bit_reader: &mut BitReader,
    alphabet_code_len_table: &HuffmanLookupTable,
    num: usize,
) -> Result<Vec<u8>> {
    let mut code_lengths = vec![0; num];
    let mut i = 0;
    while i < num {
        let symbol = resolve_symbol(bit_reader, alphabet_code_len_table)
            .ok_or_else(invalid_huffman_symbol)?;
        match symbol {
            0..=15 => {
                // 0-15: represent code lengths of 0-15
                code_lengths[i] = symbol as u8;
                i += 1;
            }
            16 => {
                // 16: copy the previous code length 3-6 times
                let repeat_len = 3 + bit_reader.read_bits(2);
                let prev_len = *code_lengths
                    .get(i.wrapping_sub(1))
                    .ok_or_else(invalid_huffman_symbol)?;
                for _ in 0..repeat_len {
                    code_lengths[i] = prev_len;
                    i += 1;
                }
            }
            17 => {
                // 17: repeat code length of 0 for 3-10 times
                let repeat_len = 3 + bit_reader.read_bits(3);
                for _ in 0..repeat_len {
                    code_lengths[i] = 0;
                    i += 1;
                }
            }
            18 => {
                // 18: repeat code length of 0 for 11-138 times
                let repeat_len = 11 + bit_reader.read_bits(7);
                for _ in 0..repeat_len {
                    code_lengths[i] = 0;
                    i += 1;
                }
            }
            _ => Err(invalid_huffman_symbol())?,
        }
    }
    Ok(code_lengths)
}

/// Inflate a DEFLATE file into a Vec<u8>.
/// This function decompresses the DEFLATE data and returns the decompressed data as a Vec<u8>.
/// The input data should be the compressed DEFLATE data.
pub fn inflate_to_vec(data: &[u8]) -> Result<Vec<u8>> {
    let mut bit_reader = BitReader::new(data);
    let mut output = Vec::new();
    loop {
        let b_final = bit_reader.read_bits(BFINAL_LEN);
        let b_type = bit_reader.read_bits(BTYPE_LEN);
        match b_type {
            BTYPE_NO_COMPRESSION => {
                // No compression
                bit_reader.advance_to_byte_boundary();
                let len = bit_reader.read_bits(LEN_LEN) as u16;
                let nlen = bit_reader.read_bits(NLEN_LEN) as u16;
                if len != !nlen {
                    return Err(invalid_len_nlen());
                }
                let mut literal_data = vec![0; len as usize];
                bit_reader.read_bytes_to_slice(len as usize, &mut literal_data);
                output.extend(literal_data);
            }
            BTYPE_FIXED_HUFFMAN => {
                // Fixed Huffman block
                let lit_tb = fixed_literal_table();
                let dis_tb = fixed_distance_table();
                inflate_compressed_block(&mut bit_reader, &mut output, &lit_tb, &dis_tb)?;
            }
            BTYPE_DYNAMIC_HUFFMAN => {
                // Dynamic Huffman block
                let (lit_tb, dis_tb) = read_dynamic_huffman_tables(&mut bit_reader)?;
                inflate_compressed_block(&mut bit_reader, &mut output, &lit_tb, &dis_tb)?;
            }
            _ => return Err(invalid_block_type()),
        }
        if b_final == BFINAL_VALUE {
            break;
        }
    }
    Ok(output)
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
        let huffman_table = fixed_literal_table();
        assert_eq!(huffman_table.max_bits, 9);
        assert_eq!(huffman_table.table[0b0_01111111], (252, 9));
        assert_eq!(huffman_table.table[0b1_10000000], (256, 7));
        assert_eq!(huffman_table.table[0b1_00000011], (280, 8));
        assert_eq!(huffman_table.table[0b0_00001100], (0, 8));
        assert_eq!(huffman_table.table[0b1_11111111], (255, 9));
    }

    #[test]
    fn test_fixed_distance_table() {
        let huffman_table = fixed_distance_table();
        assert_eq!(huffman_table.max_bits, 5);
        assert_eq!(huffman_table.table[0b00000], (0, 5));
        assert_eq!(huffman_table.table[0b11100], (7, 5));
        assert_eq!(huffman_table.table[0b11111], (31, 5));
    }
}
