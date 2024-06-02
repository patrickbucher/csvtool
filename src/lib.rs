mod duration;

use csv::{Reader, Writer};
use duration::DurationParser;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::iter;
use std::path::PathBuf;

#[derive(Debug)]
pub enum ProcessingError {
    FileAccess { path: String },
    Parsing { cause: String },
    // TODO: enhance
    CsvError,
}

impl Display for ProcessingError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ProcessingError::FileAccess { path: p } => write!(f, "unable to access file {p}"),
            ProcessingError::Parsing { cause: c } => write!(f, "{c}"),
            ProcessingError::CsvError => write!(f, "unable to process CSV file"),
        }
    }
}

impl From<csv::Error> for ProcessingError {
    fn from(_err: csv::Error) -> Self {
        // TODO: differentiate between different error causes
        ProcessingError::CsvError
    }
}

#[derive(Debug)]
pub enum Task {
    SumDuration {
        column: String,
        infile: PathBuf,
        outfile: PathBuf,
    },
    Rewrite {
        infile: PathBuf,
        outfile: PathBuf,
    },
}

impl Task {
    pub fn execute(&self) -> Result<(), ProcessingError> {
        match self {
            Task::Rewrite { infile, outfile } => {
                rewrite(infile.to_path_buf(), outfile.to_path_buf())
            }
            Task::SumDuration {
                column,
                infile,
                outfile,
            } => sum_duration(
                infile.to_path_buf(),
                outfile.to_path_buf(),
                column.to_string(),
            ),
        }
    }
}

fn rewrite(infile: PathBuf, outfile: PathBuf) -> Result<(), ProcessingError> {
    let mut reader = Reader::from_path(infile)?;
    let mut writer = Writer::from_path(outfile)?;
    let headers = reader.headers()?;
    writer.write_record(headers)?;
    for record in reader.records() {
        let record = record?;
        writer.write_record(&record)?;
    }
    Ok(())
}

fn sum_duration(infile: PathBuf, outfile: PathBuf, column: String) -> Result<(), ProcessingError> {
    let mut reader = Reader::from_path(infile)?;
    let mut writer = Writer::from_path(outfile)?;
    let headers = reader.headers()?.clone();
    writer.write_record(&headers)?;
    let mut durations: Vec<(usize, usize)> = Vec::new();
    let parser = DurationParser::new();
    for record in reader.records() {
        let record = record?;
        let record_iter = record.iter().map(|s| s.to_string());
        let header_iter = headers.iter().map(|s| s.to_string());
        let row: HashMap<String, String> = header_iter.zip(record_iter).collect();
        if let Some(duration) = row.get(&column) {
            match parser.parse_duration(duration) {
                Some(duration) => durations.push(duration),
                None => {
                    return Err(ProcessingError::Parsing {
                        cause: format!("parse '{duration}' as duration"),
                    })
                }
            }
        }
        writer.write_record(&record)?;
    }
    let total_mins = durations.iter().fold(0, |acc, (h, m)| acc + h * 60 + m);
    let hours = total_mins / 60;
    let minutes = total_mins - hours * 60;
    let total = format!("{hours}:{minutes:2}");
    writer.write_record(headers.iter().map(|h| match h == column {
        true => &total,
        false => "",
    }))?;
    Ok(())
}

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

pub fn sum_duration_deprecated(sum_col: String, csv: CsvData) -> Option<CsvData> {
    let parser = DurationParser::new();
    let mut total_mins: usize = 0;
    let mut result = CsvData { rows: Vec::new() };
    for row in &csv.rows {
        if !row.contains_key(&sum_col) {
            eprintln!("missing column {sum_col} in row {row:?}");
            return None;
        }
        let raw = row.get(&sum_col)?;
        if let Some((h, m)) = parser.parse_duration(raw) {
            total_mins += m + h * 60;
        }
        result.rows.push(row.clone());
    }
    let h = total_mins / 60;
    let m = total_mins - h * 60;
    let cols = csv.rows.first()?.keys();
    let mut sum_row: HashMap<String, String> = HashMap::new();
    for col in cols {
        sum_row.insert(
            String::from(col),
            match *col == sum_col {
                true => format!("{h}:{m:2}"),
                false => String::from(""),
            },
        );
    }
    result.rows.push(sum_row);
    Some(result)
}
