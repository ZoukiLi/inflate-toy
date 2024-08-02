use miniz_oxide;
use std::collections::HashMap;
use std::env;
use std::io::Read;
use std::io::Result;
use std::path::{Path, PathBuf};

pub trait Inflate {
    fn inflate_to_vec(&self, data: &[u8]) -> Vec<u8>;
}

pub struct MinizOxideInflater;

impl Inflate for MinizOxideInflater {
    fn inflate_to_vec(&self, data: &[u8]) -> Vec<u8> {
        miniz_oxide::inflate::decompress_to_vec(data).unwrap()
    }
}
fn get_test_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
}

const DISPLAY_LEN: usize = 16;

fn display_data(data: &[u8]) -> String {
    data.iter()
        .take(DISPLAY_LEN)
        .map(|&b| format!("{:02x} ", b))
        .collect::<String>()
}

fn test_inflator<P>(inflater: Box<dyn Inflate>, deflate_path: P, raw_data_path: P) -> Result<()>
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

#[test]
fn test_miniz_oxide_inflate() -> Result<()> {
    let data_files = get_data_files(DATA_FILES_CONFIG);
    for (raw, deflate) in data_files {
        let inflater = Box::new(MinizOxideInflater);
        let raw_data_path = get_test_dir().join(raw);
        let deflate_path = get_test_dir().join(deflate);
        test_inflator(inflater, deflate_path, raw_data_path)?;
    }
    Ok(())
}
