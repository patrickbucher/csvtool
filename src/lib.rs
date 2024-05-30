use csv::Reader;
use regex::Regex;
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

pub fn sum_duration(csv: CsvData, col: String) -> Option<CsvData> {
    let parser = DurationParser::new();
    let mut total_mins: usize = 0;
    for row in &csv.rows {
        if !row.contains_key(&col) {
            eprintln!("missing column {col} in row {row:?}");
            return None;
        }
        let raw = row.get(&col)?;
        if let Some((h, m)) = parser.parse_duration(raw) {
            total_mins += m + h * 60;
        }
    }
    let h = total_mins / 60;
    let m = total_mins - h * 60;
    println!("{h}:{m}");
    Some(csv)
}

struct DurationParser {
    pattern: Regex,
}

impl DurationParser {
    fn new() -> Self {
        let pattern = "([0-9]+):([0-9]+)";
        let pattern = Regex::new(pattern).expect(&format!("invalid pattern: '{pattern}'"));
        DurationParser { pattern }
    }

    fn parse_duration(&self, raw: &str) -> Option<(usize, usize)> {
        let caps: Vec<_> = self
            .pattern
            .captures_iter(raw)
            .map(|c| c.extract::<2>())
            .map(|(_, hm)| hm)
            .flatten()
            .map(|x| x.parse::<usize>())
            .map(|r| r.map_or(0, |v| v))
            .collect();
        let h = caps.get(0)?;
        let m = caps.get(1)?;
        Some((*h, *m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_parser() {
        let tests = [
            ("0:00", Some((0, 0))),
            ("1:00", Some((1, 0))),
            ("0:30", Some((0, 30))),
            ("0:0", Some((0, 0))),
            ("1.23", None),
            ("0.00", None),
        ];
        let parser = DurationParser::new();

        for (input, expected) in tests {
            let actual = parser.parse_duration(input);
            assert_eq!(actual, expected);
        }
    }
}
