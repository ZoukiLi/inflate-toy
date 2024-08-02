use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use inflate_toy::bit_stream;

/// Struct to manage test setup
pub struct TestSetup {
    pub temp_dir: PathBuf,
    pub test_data_dir: PathBuf,
}

impl TestSetup {
    /// Creates a new test setup with a temporary directory and a predefined test data directory
    pub fn new(temp_dir: PathBuf, test_data_dir: PathBuf) -> io::Result<Self> {
        // Create the temporary directory if it doesn't exist
        if !temp_dir.exists() {
            std::fs::create_dir_all(&temp_dir)?;
        }

        // Create the test data directory if it doesn't exist
        if !test_data_dir.exists() {
            std::fs::create_dir_all(&test_data_dir)?;
        }
        Ok(Self {
            temp_dir,
            test_data_dir,
        })
    }

    /// Creates a file with the given content in the temporary directory
    pub fn create_file(&self, file_name: &str, content: &[u8]) -> io::Result<PathBuf> {
        let file_path = self.temp_dir.join(file_name);
        let mut file = File::create(&file_path)?;
        file.write_all(content)?;
        Ok(file_path)
    }

    /// Reads the content of a file in the temporary directory
    pub fn read_file(&self, file_name: &str) -> io::Result<Vec<u8>> {
        let file_path = self.temp_dir.join(file_name);
        let mut file = File::open(&file_path)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        Ok(content)
    }

    /// Reads the content of a predefined test file from the test data directory
    pub fn read_test_file(&self, file_name: &str) -> io::Result<Vec<u8>> {
        let file_path = self.test_data_dir.join(file_name);
        let mut file = File::open(&file_path)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        Ok(content)
    }

    /// Compares the content of two files
    pub fn compare_files(&self, file1: &Path, file2: &Path) -> io::Result<bool> {
        let mut f1 = File::open(file1)?;
        let mut f2 = File::open(file2)?;
        let mut buf1 = Vec::new();
        let mut buf2 = Vec::new();
        f1.read_to_end(&mut buf1)?;
        f2.read_to_end(&mut buf2)?;
        Ok(buf1 == buf2)
    }
}

/// Macro to standardize test execution
#[macro_export]
macro_rules! run_test {
    ($test_name:ident, $test_in_dir:expr, $test_out_dir:expr, $test_body:expr) => {
        #[test]
        fn $test_name() -> std::io::Result<()> {
            let setup = TestSetup::new(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join($test_out_dir),
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join($test_in_dir),
            )?;
            $test_body(setup)
        }
    };
}

/// Macro to standardize test execution for a batch of tests
#[macro_export]
macro_rules! run_tests {
    ($test_name:ident, $test_in_dir:expr, $test_out_dir:expr, $input_files:expr, $test_body:expr) => {
        #[test]
        fn $test_name() -> std::io::Result<()> {
            let setup = TestSetup::new(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join($test_out_dir),
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join($test_in_dir),
            )?;
            for _file in $input_files {
                let _input = setup.read_test_file(_file)?;
                $test_body(&setup, _input)?;
            }
            Ok(())
        }
    };
}

// Example usage of the test paradigm

run_test!(
    example_test,
    "tests/data",
    "tests/out",
    |setup: TestSetup| {
        let test_content = setup.read_test_file("random_data.deflate")?;
        let file_path = setup.create_file("test.txt", &test_content)?;
        let content = setup.read_file("test.txt")?;
        assert_eq!(content, test_content);

        let another_file_path = setup.create_file("another_test.txt", b"Hello, Rust!")?;
        assert!(setup.compare_files(&file_path, &another_file_path)? == false);

        Ok(())
    }
);

run_tests!(
    deflate_test_smoke,
    "tests/data",
    "tests/out",
    [
        "repeat_1_data.deflate",
        "repeat_2_data.deflate",
        "random_data.deflate",
        "lorem_ipsum_data.deflate"
    ],
    |_: &TestSetup, input: Vec<u8>| -> io::Result<()> {
        let mut bit_reader = bit_stream::BitReader::new(&input);
        let mut output = Vec::new();
        loop {
            let b_final = bit_reader.read_bits(1);
            let b_type = bit_reader.read_bits(2);

            match b_type {
                0b00 => {
                    // Literal block
                    bit_reader.advance_to_byte_boundary();
                    let len = bit_reader.read_bits(16) as u16;
                    let nlen = bit_reader.read_bits(16) as u16;
                    assert_eq!(len, !nlen);
                    println!("Literal block: len={}, nlen={}", len, nlen);

                    let mut literal_data = vec![0; len as usize];
                    bit_reader.read_bytes_to_slice(len as usize, &mut literal_data);
                    output.extend(literal_data);
                }
                0b01 => {
                    // Fixed Huffman block
                    println!("Fixed Huffman block");
                    let lit_tb = inflate_toy::huffman::HuffmanLookupTable::fixed_literal_table();
                    let dis_tb = inflate_toy::huffman::HuffmanLookupTable::fixed_distance_table();

                    let bytes_outputted =
                        inflate_block(&mut bit_reader, &mut output, &lit_tb, &dis_tb)?;
                    println!("Bytes outputted: {}", bytes_outputted);
                }
                0b10 => {
                    // Dynamic Huffman block
                    println!("Dynamic Huffman block");
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid block type",
                ))?,
            }

            if b_final == 1 {
                break;
            }
        }
        // print the output in hex format
        println!("Output: \n{}", display_data(&output));
        Ok(())
    }
);

/// Resolve one symbol from the Huffman table.
fn resolve_symbol(
    bit_reader: &mut bit_stream::BitReader,
    huffman_table: &inflate_toy::huffman::HuffmanLookupTable,
) -> Option<usize> {
    let peek_code = bit_reader.try_peek_bits(huffman_table.max_bits as usize)?;
    let (symbol, len) = huffman_table.get(peek_code)?;
    bit_reader.try_advance(len as usize)?;
    Some(symbol)
}

fn invalid_huffman_symbol() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "Invalid Huffman symbol")
}

/// Inflate block with literal and distance huffman tables.
/// Because a duplicated string reference may refer to a string in a previous block,
/// we need the whole output to be able to resolve the references.
/// Returns the number of bytes outputted.
fn inflate_block(
    bit_reader: &mut bit_stream::BitReader,
    output: &mut Vec<u8>,
    lit_tb: &inflate_toy::huffman::HuffmanLookupTable,
    dis_tb: &inflate_toy::huffman::HuffmanLookupTable,
) -> io::Result<usize> {
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

const END_BLOCK_CODE: usize = 256;
const LITERAL_CODE_MAX: usize = 255;
const LITERAL_CODE_BASE: usize = 0;
const LENGTH_CODE_MAX: usize = 285;
const LENGTH_CODE_BASE: usize = 257;

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
fn get_length_by_code(code: usize, bit_reader: &mut bit_stream::BitReader) -> Option<usize> {
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
fn get_distance_by_code(code: usize, bit_reader: &mut bit_stream::BitReader) -> Option<usize> {
    let (distance_code, distance_base, extra_bits) = DISTANCE_CODE_TABLE.get(code).cloned()?;
    assert!(distance_code == code);
    Some(distance_base + bit_reader.read_bits(extra_bits))
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

/// Display the data in hex format.
fn display_data(data: &[u8]) -> String {
    let mut result = String::new();

    for (i, chunk) in data.chunks(16).enumerate() {
        // Print the offset
        result.push_str(&format!("{:08x}: ", i * 16));

        // Print the byte values in hex
        for byte in chunk {
            result.push_str(&format!("{:02x} ", byte));
        }

        // If the chunk is less than 16 bytes, fill the gap
        for _ in 0..(16 - chunk.len()) {
            result.push_str("   ");
        }

        // Print the ASCII representation
        result.push_str(" |");
        for byte in chunk {
            if byte.is_ascii_graphic() {
                result.push(*byte as char);
            } else {
                result.push('.');
            }
        }
        result.push_str("|\n");
    }

    result
}
