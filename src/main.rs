use clap::{Parser, Subcommand};
use csvtool::Task;
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
    Rewrite {
        /// CSV input file
        #[arg(short, long)]
        infile: String,

        /// CSV output file
        #[arg(short, long)]
        outfile: String,
    },
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
            Commands::Rewrite { infile, outfile } => Task::Rewrite {
                infile: PathBuf::from(infile),
                outfile: PathBuf::from(outfile),
            },
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

fn main() {
    let cli = Args::parse();
    let task: Task = cli.command.into();
    if let Err(e) = task.execute() {
        eprintln!("error: {e}");
    }
}
