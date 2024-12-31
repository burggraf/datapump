// src/csv_reader.rs

use std::io::{BufRead, BufReader};

/// Count the total number of lines in the file (for progress reporting).
pub fn count_rows(file_path: &str) -> Result<usize, String> {
    let file = std::fs::File::open(file_path)
        .map_err(|e| format!("Failed to open CSV file for row count: {}", e))?;

    let reader = BufReader::new(file);
    let total_lines = reader.lines().count();

    Ok(total_lines)
}

/// Detect whether we should use comma or tab as the delimiter.
pub fn detect_delimiter(file_path: &str) -> Result<u8, String> {
    let file = std::fs::File::open(file_path)
        .map_err(|e| format!("Failed to open CSV file for delimiter detection: {}", e))?;
    let mut reader = BufReader::new(file);

    let mut first_line = String::new();
    reader
        .read_line(&mut first_line)
        .map_err(|e| format!("Failed to read first line: {}", e))?;

    let comma_count = first_line.matches(',').count();
    let tab_count = first_line.matches('\t').count();

    // Decide which delimiter to use
    let delimiter = if tab_count > comma_count { b'\t' } else { b',' };
    Ok(delimiter)
}

/// Create a fresh `csv::Reader` starting from the beginning of the file
/// with the chosen delimiter, skipping the headers, etc.
pub fn create_csv_reader(
    file_path: &str,
    delimiter: u8,
) -> Result<csv::Reader<std::fs::File>, String> {
    let file =
        std::fs::File::open(file_path).map_err(|e| format!("Failed to reopen CSV file: {}", e))?;

    let rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(true)
        .flexible(false)
        .from_reader(file);

    Ok(rdr)
}
