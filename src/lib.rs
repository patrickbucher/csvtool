mod duration;

use csv::{Reader, StringRecord, Writer};
use duration::DurationParser;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
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
        // for CSV errors, reporting the failing line would be useful
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
            } => rewrite_with_accumulator(
                infile.to_path_buf(),
                outfile.to_path_buf(),
                column.into(),
                extract_column(column, String::from("0:00")),
                // TODO: consider passing colum to sum_durations(), then move the code to produce
                // the sum line into the closure returned by sum_durations, which then also needs
                // the headers.
                sum_durations(),
            ),
        }
    }
}

fn rewrite(infile: PathBuf, outfile: PathBuf) -> Result<(), ProcessingError> {
    let mut reader = Reader::from_path(infile)?;
    let mut writer = Writer::from_path(outfile)?;
    let headers: StringRecord = reader.headers()?.iter().map(|s| s.trim()).collect();
    writer.write_record(&headers)?;
    for record in reader.records() {
        let record: StringRecord = record?.iter().map(|s| s.trim()).collect();
        writer.write_record(&record)?;
    }
    Ok(())
}

fn rewrite_with_accumulator(
    infile: PathBuf,
    outfile: PathBuf,
    column: String,
    extractor: impl Fn(StringRecord, StringRecord) -> String,
    accumulator: impl Fn(&Vec<String>) -> String,
) -> Result<(), ProcessingError> {
    let mut reader = Reader::from_path(infile)?;
    let mut writer = Writer::from_path(outfile)?;
    let headers: StringRecord = reader.headers()?.iter().map(|s| s.trim()).collect();
    writer.write_record(&headers)?;
    let mut collection: Vec<String> = Vec::new();
    for record in reader.records() {
        let record: StringRecord = record?.iter().map(|s| s.trim()).collect();
        writer.write_record(&record)?;
        let value = extractor(headers.clone(), record);
        collection.push(value);
    }
    let total = accumulator(&collection);
    writer.write_record(headers.iter().map(|h| match h == column {
        true => &total,
        false => "",
    }))?;
    Ok(())
}

fn extract_column(
    column: &str,
    fallback: String,
) -> impl Fn(StringRecord, StringRecord) -> String + '_ {
    move |headers: StringRecord, record: StringRecord| {
        let record_iter = record.iter().map(|s| s.trim().to_string());
        let header_iter = headers.iter().map(|s| s.trim().to_string());
        let row: HashMap<String, String> = header_iter.zip(record_iter).collect();
        row.get(column).unwrap_or(&fallback).to_string()
    }
}

// TODO: return Result instead of falling back to 0:00 in the case of an error.
fn sum_durations() -> impl Fn(&Vec<String>) -> String {
    let parser = DurationParser::new();
    move |durations: &Vec<String>| {
        let minutes = durations
            .iter()
            .map(|d| parser.parse_duration(d))
            .map(|d| d.unwrap_or((0, 0)))
            .fold(0, |acc: usize, (h, m)| acc + 60 * h + m);
        let hours = minutes / 60;
        let minutes = minutes - hours * 60;
        format!("{hours}:{minutes:02}")
    }
}
