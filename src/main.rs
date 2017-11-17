extern crate csv;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate steel_cent;

use std::error::Error;
use std::io;
use std::process;

use steel_cent::Money;
use steel_cent::formatting::us_style;

#[derive(Debug, Deserialize)]
struct Transaction {
    #[serde(rename = "Date")]
    date: String, // 04/28/2017
    #[serde(rename = "Investment")]
    investment: String, // any string
    #[serde(rename = "Transaction Type")]
    txtype: String, // one of: 'DISABILITY PREMIUM', 'Change in Market Value', or 'CONTRIBUTION'
    #[serde(rename = "Amount")]
    #[serde(deserialize_with = "parse_money")]
    amount: Money, //two decimal places with commas
    #[serde(rename = "Shares/Unit")]
    #[serde(deserialize_with = "parse_money")]
    share: Money, //three decimal places with commas
}

use serde::de;
use serde::{Deserialize, Deserializer};

fn parse_money<'de, D>(deserializer: D) -> Result<Money, D::Error>
    where D: Deserializer<'de>
{
    struct MaybeMoney(Money);

    impl<'de> de::Visitor<'de> for MaybeMoney {
        type Value = Money;

        fn expecting(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            // write!(f, "a money")
            f.write_str("a money")
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where E: ::serde::de::Error
        {
            Ok(us_style().parser().parse(v).unwrap())
        }
    }
    deserializer.deserialize_any(MaybeMoney)
}

fn main() {
    if let Err(err) = run() {
        println!("error running: {}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<Error>> {
    let mut reader =
        csv::ReaderBuilder::new().has_headers(true).flexible(false).from_reader(io::stdin());
    for row in reader.deserialize() {
        let tx: Transaction = row.expect("Unable to parse row");
        println!("{:?}", tx);
    }
    Ok(())
}
