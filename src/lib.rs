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
    let headers: StringRecord = reader.headers()?.iter().map(|s| s.trim()).collect();
    writer.write_record(&headers)?;
    for record in reader.records() {
        let record: StringRecord = record?.iter().map(|s| s.trim()).collect();
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
        let record: StringRecord = record?.iter().map(|s| s.trim()).collect();
        let record_iter = record.iter().map(|s| s.trim().to_string());
        let header_iter = headers.iter().map(|s| s.trim().to_string());
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
