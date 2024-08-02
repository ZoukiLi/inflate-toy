use std::f32::consts::E;
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

/// Resolve one symbol from the Huffman table
fn resolve_symbol(
    bit_stream: &mut bit_stream::BitReader,
    huffman_table: &inflate_toy::huffman::HuffmanLookupTable,
) -> Option<usize> {
    let peek_code = bit_stream.try_peek_bits(huffman_table.max_bits as usize)?;
    let (symbol, len) = huffman_table.get(peek_code)?;
    bit_stream.try_advance(len as usize)?;
    Some(symbol)
}

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
