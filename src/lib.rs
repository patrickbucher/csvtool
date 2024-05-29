use csv::Reader;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::iter;

#[derive(Debug)]
pub struct CsvData {
    pub rows: Vec<HashMap<String, String>>,
}

pub fn parse_csv(path: String) -> Result<CsvData, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = Reader::from_reader(file);
    let header: Vec<String> = reader.headers()?.iter().map(|s| s.into()).collect();
    let mut lines: Vec<HashMap<String, String>> = Vec::new();
    for result in reader.records() {
        match result {
            Ok(record) => {
                let cols = record.iter().map(|s| s.to_string());
                let heads = header.iter().map(|s| s.to_string());
                let pairs: HashMap<String, String> = iter::zip(heads, cols).collect();
                lines.push(pairs);
            }
            Err(e) => return Err(Box::new(e)),
        }
    }
    Ok(CsvData { rows: lines })
}

pub fn sum_duration(csv: CsvData, _col: String) -> CsvData {
    csv
}
