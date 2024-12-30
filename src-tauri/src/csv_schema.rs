use csv::{ReaderBuilder, StringRecord};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn get_csv_schema(file_path: &str) -> Result<String, String> {
    let file = File::open(file_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);

    // Read first few lines to detect delimiter
    let mut buffer = String::new();
    let mut temp_reader =
        BufReader::new(File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?);

    // Read up to 5 lines to detect delimiter
    let mut tab_count = 0;
    let mut comma_count = 0;
    let mut lines_read = 0;

    while lines_read < 5 {
        buffer.clear();
        let bytes_read = temp_reader
            .read_line(&mut buffer)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        if bytes_read == 0 {
            break;
        }

        tab_count += buffer.matches('\t').count();
        comma_count += buffer.matches(',').count();
        lines_read += 1;
    }

    // Validate delimiter detection
    if tab_count == 0 && comma_count == 0 {
        return Err("Unable to detect delimiter - no tabs or commas found".to_string());
    }

    // Use tab delimiter if significantly more tabs than commas
    let delimiter = if tab_count > comma_count * 2 {
        b'\t'
    } else {
        b','
    };

    // First attempt with detected delimiter
    // Try parsing with detected delimiter first
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .delimiter(delimiter)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(reader);

    let headers = match csv_reader.headers() {
        Ok(h) => h.clone(),
        Err(_) => {
            // If headers fail, try with opposite delimiter
            let file = File::open(file_path).map_err(|e| e.to_string())?;
            let reader = BufReader::new(file);
            let alt_delimiter = if delimiter == b'\t' { b',' } else { b'\t' };
            csv_reader = ReaderBuilder::new()
                .has_headers(true)
                .delimiter(alt_delimiter)
                .flexible(true)
                .trim(csv::Trim::All)
                .from_reader(reader);
            csv_reader.headers().map_err(|e| e.to_string())?.clone()
        }
    };

    let mut field_types = Vec::new();
    let mut max_fields = headers.len();

    // Track the number of fields in the first valid record
    let mut first_record_fields = 0;

    let mut max_fields = headers.len();

    for result in csv_reader.records() {
        let record = match result {
            Ok(r) => {
                // Skip empty records
                if r.is_empty() {
                    continue;
                }
                // Update max fields if this record has more
                if r.len() > max_fields {
                    max_fields = r.len();
                }
                r
            }
            Err(e) => {
                // Log the error but continue processing
                eprintln!("Warning: {}", e);
                continue;
            }
        };

        // If this is the first valid record, note its field count
        if first_record_fields == 0 {
            first_record_fields = record.len();
        }

        // Ensure we have enough field types
        while field_types.len() < record.len() {
            field_types.push("text".to_string());
        }

        for (i, field) in record.iter().enumerate() {
            let current_type = if field.parse::<i64>().is_ok() {
                "integer"
            } else if field.parse::<f64>().is_ok() {
                "real"
            } else {
                "text"
            };
            if field_types.len() <= i {
                field_types.push(current_type.to_string());
            } else if field_types[i] != "text" && current_type == "text" {
                field_types[i] = "text".to_string();
            } else if field_types[i] != "real" && current_type == "real" {
                field_types[i] = "real".to_string();
            }
        }
        break;
    }

    let schema = headers
        .iter()
        .zip(field_types.iter())
        .map(|(header, field_type)| format!("{}:{}", header, field_type))
        .collect::<Vec<String>>()
        .join(",");

    Ok(schema)
}
