use clap::{Parser, Subcommand};
use csvtool::{parse_csv, sum_duration, CsvData};
use std::error::Error;

/// Operations on CSV files.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    #[arg()]
    file: String,
}

#[derive(Subcommand)]
enum Commands {
    SumDuration {
        /// column to sum up
        #[arg(short, long)]
        column: String,
    },
}

#[derive(Debug)]
pub enum Task {
    SumDuration { column: String },
}

impl From<Commands> for Task {
    fn from(cmd: Commands) -> Self {
        match cmd {
            Commands::SumDuration { column: col } => Task::SumDuration { column: col },
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Args::parse();
    let data = parse_csv(cli.file)?;
    let task: Task = cli.command.into();
    dispatch(data, task);
    Ok(())
}

pub fn dispatch(csv: CsvData, task: Task) -> CsvData {
    println!("{:?}\n{:?}", csv, task);
    match task {
        Task::SumDuration { column: col } => sum_duration(csv, col),
    }
}