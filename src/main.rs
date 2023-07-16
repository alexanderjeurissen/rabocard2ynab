extern crate pretty_env_logger;
#[macro_use] extern crate log;

use std::env;
use log::{info, error, Level};
use std::path::PathBuf;
use std::time::Instant;
use clap::Parser;
use csvsc::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(
    long,
    help = "Path to one or many Rabobank creditcard transaction csv exports.",
    num_args = 1..,
    value_delimiter = ' '
  )]
  files: Vec<PathBuf>,

  #[arg(
    long,
    help = "Path to store converted Ynab compatible csv. defaults to rabocard_<timestamp>_ynab.csv in current working directory.",
  )]
  output: Option<PathBuf>,

  #[arg(
    long,
    env = "LOG",
    default_value = "info",
    help = "desired verbosity of logging during execution (trace, debug, info, warning, error)."
  )]
  log_level: log::Level
}

fn process_files(files: &Vec<PathBuf>, output: &str) {
  let rabo_card_columns: Vec<&str> = vec![
    "Datum",                // Date
    "Omschrijving",         // Payee
    "Transactiereferentie", // Memo
    "Bedrag",               // Amount
  ];

  let chain = InputStreamBuilder::from_paths(files)
    .unwrap().build().unwrap() // TODO: figure out if we can have decent error handling instead of unwrapping everything
    .select(rabo_card_columns) // NOTE: only select columns we will keep
    // NOTE: rename the columns to match Ynab expected naming
    .rename("Datum", "Date")
    .rename("Omschrijving", "Payee")
    .rename("Transactiereferentie", "Memo")
    .rename("Bedrag", "Amount")

    .flush(Target::path(output)).unwrap()
    .into_iter();

  // NOTE: consume the stream, reporting any errors to stderr.
  for record in chain {
    match record {
      Ok(r) => { debug!("Successfully processed record: {:?}", r) }
      Err(e) => { error!("Encountered error while processing record: {:?}", e) }
    }
  }
}

fn main() {
  let start = Instant::now();
  let args = Args::parse();

  set_log_level(args.log_level);

  info!("Processing {:?} csv files...", &args.files.len());

  let output_path: String = determine_output_path(&args.output);

  process_files(&args.files, &output_path);

  // NOTE: we are done log total execution time
  let duration = start.elapsed();
  info!("Finished processing. total duration: {:?}", duration);
}

fn set_log_level(log_level: Level) {
  // NOTE: workaround to allow setting the pretty_env log level
  // through CLI argument.
  env::set_var("PRETTY_ENV_LOG_LEVEL", log_level.to_string());
  pretty_env_logger::init_custom_env("PRETTY_ENV_LOG_LEVEL");
}

fn determine_output_path(output_path: &Option<PathBuf>) -> String {
  let path: PathBuf = match output_path {
    Some(path) => {
      if path.is_dir() {
        error!("Provided output path is a directory.");
        default_output_path()
      } else if path.is_file() {
        error!("Provided output path already exists.");
        default_output_path()
      } else {
        info!("Will store results in provided output path: {:?}", path.to_path_buf());
        path.to_path_buf()
      }
    }
    None => { default_output_path() }
  };

  match path.to_str() {
    Some(path_str) => { String::from(path_str) }
    None => {
      panic!("output path is not valid unicode");
    }
  }
}

fn default_output_path() -> PathBuf {
  let work_dir: PathBuf = match env::current_dir() {
    Ok(p) => { p }
    Err(e) => { panic!("Could not obtain current working directory: {:?}", e) }
  };

  let default_file_name: String = format!("rabocard_{}_ynab", chrono::offset::Local::now().timestamp());

  let default_path: PathBuf = work_dir.with_file_name(default_file_name).with_extension("csv");

  info!("No (valid) output path provided, defaulting to {:?}", default_path);

  default_path
}
