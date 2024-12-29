use csv::ReaderBuilder;
use std::fs::File;
use std::io::BufReader;

pub fn get_csv_schema(file_path: &str) -> Result<String, String> {
    let file = File::open(file_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);

    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

    let headers = csv_reader.headers().map_err(|e| e.to_string())?.clone();
    let mut field_types = Vec::new();

    for result in csv_reader.records() {
        let record = result.map_err(|e| e.to_string())?;
        for (i, field) in record.iter().enumerate() {
            let current_type = if field.parse::<i64>().is_ok() {
                "integer"
            } else if field.parse::<f64>().is_ok() {
                "float"
            } else {
                "string"
            };
            if field_types.len() <= i {
                field_types.push(current_type.to_string());
            } else if field_types[i] != "string" && current_type == "string" {
                field_types[i] = "string".to_string();
            } else if field_types[i] != "float" && current_type == "float" {
                field_types[i] = "float".to_string();
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
