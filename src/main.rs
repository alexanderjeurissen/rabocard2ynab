extern crate pretty_env_logger;

use std::{error::Error};
use csv::{Reader, Writer};

use log::{info};
use std::path::PathBuf;
use std::time::Instant;

use structopt::StructOpt;

#[derive(Debug, serde::Deserialize)]
struct RaboCardTransaction {
  #[serde(rename = "Tegenrekening IBAN")]
  counter_party: String,
  #[serde(rename = "Munt")]
  currency: String,
  #[serde(rename = "Creditcard Nummer")]
  card_number: String,
  #[serde(rename = "Productnaam")]
  product_name: String,
  #[serde(rename = "Creditcard Regel1")]
  line_one: String,
  #[serde(rename = "Creditcard Regel2")]
  line_two: String,
  #[serde(rename = "Transactiereferentie")]
  memo: String,
  #[serde(rename = "Datum")]
  date: String,
  #[serde(rename = "Bedrag")]
  amount: String,
  #[serde(rename = "Omschrijving")]
  payee: String,
  #[serde(rename = "Oorspr bedrag")]
  paid_amount: String,
  #[serde(rename = "Oorspr munt")]
  paid_currency: String,
  #[serde(rename = "Koers")]
  exchange_rate: String,
}

#[derive(Debug, serde::Serialize)]
struct YnabTransaction {
  #[serde(rename(serialize = "Date"))]
  date: String,
  #[serde(rename(serialize = "Payee"))]
  payee: String,
  #[serde(rename(serialize = "Memo"))]
  memo: String,
  #[serde(rename(serialize = "Amount"))]
  amount: String
}


#[derive(StructOpt, Debug)]
#[structopt(name = "rabocard2ynab", about = "Explanation of rabocard2ynab usage.")]
struct Cli {
    #[structopt(
        long,
        help = "Path to Rabobank creditcard transaction csv export.",
        parse(from_os_str)
    )]
    input: PathBuf,

    #[structopt(
        long,
        help = "Path to store converted Ynab compatible csv. defaults to rabocard_<timestamp>_ynab.csv in same directory as input.",
        parse(from_os_str)
    )]
    output: Option<PathBuf>,
}

fn process_csv(input_path: &PathBuf, output_path: &PathBuf) -> Result<(), Box<dyn Error>> {
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
}

fn main() -> Result<(), Box<dyn Error>> {
  let start = Instant::now();
  pretty_env_logger::init_custom_env("LOG");

  let args = Cli::from_args();

  let output_path: PathBuf = output_path(&args.input, &args.output);

  if let Err(err) = process_csv(&args.input, &output_path) {
    panic!("error processing csv: {}", err);
  }

  // NOTE: we are done log total execution time
  let duration = start.elapsed();
  info!("'FINISHED after {:?}", duration);

  return Ok(());
}

fn output_path(input_path: &PathBuf, output_path: &Option<PathBuf>) -> PathBuf {
  match output_path {
    Some(path) => { path.to_path_buf() }
    None => {
      let file_name: String = format!("rabocard_{}_ynab", chrono::offset::Local::now().timestamp());

      input_path.with_file_name(file_name).with_extension("csv")
    }
  }
}
