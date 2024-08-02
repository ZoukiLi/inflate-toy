use miniz_oxide;
use std::collections::HashMap;
use std::env;
use std::io::Read;
use std::io::Result;
use std::path::{Path, PathBuf};

pub trait Inflate {
    fn inflate_to_vec(&self, data: &[u8]) -> Vec<u8>;
}

pub struct MinizOxideInflator;

impl Inflate for MinizOxideInflator {
    fn inflate_to_vec(&self, data: &[u8]) -> Vec<u8> {
        miniz_oxide::inflate::decompress_to_vec(data).unwrap()
    }
}

pub struct ToyInflator;

impl Inflate for ToyInflator {
    fn inflate_to_vec(&self, data: &[u8]) -> Vec<u8> {
        inflate_toy::inflate::inflate_to_vec(data).unwrap()
    }
}

fn get_test_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
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

fn test_inflator<P>(inflater: &dyn Inflate, deflate_path: &P, raw_data_path: &P) -> Result<()>
where
    P: AsRef<Path>,
{
    // Read the DEFLATE data
    let mut deflate_file = std::fs::File::open(&deflate_path)?;
    let mut deflate_data = Vec::new();
    deflate_file.read_to_end(&mut deflate_data)?;

    // Display the DEFLATE data
    println!(
        "DEFLATE data({}):\n{}",
        deflate_data.len(),
        display_data(&deflate_data)
    );

    // Decompress the DEFLATE data using your custom inflater
    let decompressed_data = inflater.inflate_to_vec(&deflate_data);

    // Display the DECOMPRESSED data
    println!(
        "DECOMPRESSED data({}):\n{}",
        decompressed_data.len(),
        display_data(&decompressed_data)
    );

    // Read the RAW data
    let mut raw_data_file = std::fs::File::open(&raw_data_path)?;
    let mut raw_data = Vec::new();
    raw_data_file.read_to_end(&mut raw_data)?;

    // Display the RAW data
    println!("RAW data({}):\n{}", raw_data.len(), display_data(&raw_data));

    // Compare the DECOMPRESSED data with the RAW data
    assert_eq!(decompressed_data, raw_data);

    Ok(())
}

const DATA_FILES_CONFIG: &str = "manifest.json";

/// Get the data_files map in json format
fn get_data_files(file: &str) -> HashMap<String, String> {
    let data_files_path = get_test_dir().join(file);
    let data_files = std::fs::read_to_string(data_files_path).unwrap();
    serde_json::from_str(&data_files).unwrap()
}

const INFLATORS: &[(&str, &dyn Inflate)] = &[
    ("MinizOxideInflator", &MinizOxideInflator),
    ("ToyInflator", &ToyInflator),
];

#[test]
fn test_inflators() -> Result<()> {
    let data_files = get_data_files(DATA_FILES_CONFIG);

    for (name, deflate_path) in data_files {
        println!("Test: {}", name);
        let raw_data_path = get_test_dir().join(&name);
        let deflate_path = get_test_dir().join(&deflate_path);
        for (inflator_name, inflator) in INFLATORS {
            println!("Inflator: {}", inflator_name);
            test_inflator(*inflator, &deflate_path, &raw_data_path)?;
        }
    }

    Ok(())
}
