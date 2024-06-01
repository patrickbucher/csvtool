use clap::{Parser, Subcommand};
use csvtool::Task;
use std::error::Error;
use std::path::PathBuf;

/// Operations on CSV files.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    SumDuration {
        /// column to sum up
        #[arg(short, long)]
        column: String,

        /// CSV input file
        #[arg(short, long)]
        infile: String,

        /// CSV output file
        #[arg(short, long)]
        outfile: String,
    },
}

impl From<Commands> for Task {
    fn from(cmd: Commands) -> Self {
        match cmd {
            Commands::SumDuration {
                column,
                infile,
                outfile,
            } => Task::SumDuration {
                column,
                infile: PathBuf::from(infile),
                outfile: PathBuf::from(outfile),
            },
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Args::parse();
    let task: Task = cli.command.into();
    let result = task.execute();
    println!("{:?}", result);
    Ok(())
}
