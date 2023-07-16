extern crate pretty_env_logger;
#[macro_use] extern crate log;

use std::{error::Error, env};

use log::{info, Level};
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;

const RABO_CARD_COLUMNS: [&'static str; 13] = [
  "Tegenrekening IBAN",
  "Munt",
  "Creditcard Nummer",
  "Productnaam",
  "Creditcard Regel1",
  "Creditcard Regel2",
  "Transactiereferentie",
  "Datum",
  "Bedrag",
  "Omschrijving",
  "Oorspr bedrag",
  "Oorspr munt",
  "Koers"
];

const YNAB_COLUMNS: [&'static str; 4] = [
  "Date",
  "Payee",
  "Memo",
  "Amount"
];

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        long,
        help = "Path to one or many Rabobank creditcard transaction csv exports.",
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

/* fn process_csv(source_path: &PathBuf, destination: &PathBuf) -> Result<(), Box<dyn Error>> {
  let mut rdr = Reader::from_path(input_path.as_path())?;
  let mut writer = Writer::from_path(output_path.as_path())?;

  for result in rdr.deserialize() {
    let transaction: RaboCardTransaction = result?;

    let mut memo: String = format!("reference: {}", transaction.memo);

    if transaction.paid_currency != "" {
      memo += format!(", paid_amount: {} {}", transaction.paid_amount, transaction.paid_currency).as_str();
      memo += format!(", exchange_rate: {}", transaction.exchange_rate).as_str();
    }

    writer.serialize(YnabTransaction {
      date: transaction.date,
      payee: transaction.payee,
      memo: memo,
      amount: transaction.amount,
    })?;
  }

  Ok(())
} */

fn main() -> Result<(), Box<dyn Error>> {
  let start = Instant::now();
  let args = Args::parse();

  set_log_level(args.log_level);

  let _output_path: PathBuf = output_path(&args.output);

  /* if let Err(err) = process_csv(&args.files[0], &args.output) {
    panic!("error processing csv: {}", err);
  } */

  // NOTE: we are done log total execution time
  let duration = start.elapsed();
  debug!("'FINISHED after {:?}", duration);

  return Ok(());
}

fn set_log_level(log_level: Level) {
  // NOTE: workaround to allow setting the pretty_env log level
  // through CLI argument.
  env::set_var("PRETTY_ENV_LOG_LEVEL", log_level.to_string());
  pretty_env_logger::init_custom_env("PRETTY_ENV_LOG_LEVEL");
}

fn output_path(output_path: &Option<PathBuf>) -> PathBuf {
  match output_path {
    Some(path) => { path.to_path_buf() }
    None => {
      let path: PathBuf = match env::current_dir() {
        Ok(p) => { p }
        Err(e) => { panic!("Could not obtain current working directory: {}", e) }
      };

      let file_name: String = format!("rabocard_{}_ynab", chrono::offset::Local::now().timestamp());
      let destination: PathBuf = path.with_file_name(file_name).with_extension("csv");

      info!("No output path provided, defaulting to {:?}", destination);

      destination
    }
  }
}
