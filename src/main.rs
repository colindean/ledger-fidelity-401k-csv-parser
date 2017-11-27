extern crate csv;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate steel_cent;
extern crate chrono;

use std::error::Error;
use std::io;
use std::process;

use steel_cent::Money;
use steel_cent::Currency;
use steel_cent::formatting::FormatSpec;
use steel_cent::formatting::FormatPart::*;
use steel_cent::formatting::FormatPart;

use chrono::prelude::*;

use std::ops::Neg;

#[derive(Debug, Deserialize)]
struct Transaction {
    #[serde(rename = "Date")]
    #[serde(deserialize_with = "parse_date")]
    date: NaiveDate, // 04/28/2017
    #[serde(rename = "Investment")]
    investment: String, // any string
    #[serde(rename = "Transaction Type")]
    txtype: String, // one of: 'DISABILITY PREMIUM', 'Change in Market Value', or 'CONTRIBUTION'
    #[serde(rename = "Amount")]
    #[serde(deserialize_with = "parse_money")]
    amount: Money, // two decimal places with commas
    #[serde(rename = "Shares/Unit")]
    #[serde(deserialize_with = "parse_shares")]
    share: Money, // three decimal places with commas
}

use serde::de;
use serde::{Deserialize, Deserializer};

use steel_cent::currency::USD;
fn dollar_formatter() -> FormatSpec {
    usd_formatter(vec![OptionalMinus, Amount, CurrencySymbol])
}
fn dollar_formatter_without_symbol() -> FormatSpec {
    usd_formatter(vec![OptionalMinus, Amount])
}
fn usd_formatter(template: Vec<FormatPart>) -> FormatSpec {
    comma_and_period_formatter(template).with_short_symbol(USD, String::from("$"))
}

fn share_currency() -> Currency { Currency::new("SHR", 999, 3) }

fn share_formatter() -> FormatSpec {
    shr_formatter(vec![OptionalMinus, Amount, CurrencySymbol])
}
fn share_formatter_without_symbol() -> FormatSpec {
    shr_formatter(vec![OptionalMinus, Amount])
}
fn shr_formatter(template: Vec<FormatPart>) -> FormatSpec {
    comma_and_period_formatter(template).with_short_symbol(share_currency(), String::from("$"))
}
fn comma_and_period_formatter(template: Vec<FormatPart>) -> FormatSpec {
    FormatSpec::new(',', '.', template)
}

fn add_currency_symbol(num: &str) -> String {
    format!("{}$", String::from(num))
}

//FIXME: I hate the duplication between these functions
// but Rust seems to make it really hard to return functions
fn parse_shares<'de, D>(deserializer: D) -> Result<Money, D::Error>
    where D: Deserializer<'de>
{
    let as_string = String::deserialize(deserializer)?;
    Ok(share_formatter()
        .parser()
        .parse(add_currency_symbol(as_string.as_str()).as_str())
        .expect("amount could not be parsed"))
}

fn parse_money<'de, D>(deserializer: D) -> Result<Money, D::Error>
    where D: Deserializer<'de>
{
    let as_string = String::deserialize(deserializer)?;
    Ok(dollar_formatter()
        .parser()
        .parse(add_currency_symbol(as_string.as_str()).as_str())
        .expect("amount could not be parsed"))
}

fn parse_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where D: Deserializer<'de>
{
    let as_string = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(as_string.as_str(), "%m/%d/%Y")
        .map_err(de::Error::custom)
        .map(|d|d)
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
        println!("{}", format_txn_as_ledger(tx, String::from("Assets:Cash"), String::from("Assets:Investments")));
    }
    Ok(())
}

fn money_is_negative(money: Money) -> bool {
    money.minor_amount() < 0
}

fn format_txn_as_ledger(txn: Transaction, cash_account: String, shares_account: String) -> String {
    let cash_amount = if money_is_negative(txn.share) {
        txn.amount
    } else {
        txn.amount.neg()
    };
    let shares = steel_cent::formatting::format(&share_formatter_without_symbol(), &txn.share);
    let cash = steel_cent::formatting::format(&dollar_formatter_without_symbol(), &cash_amount);

    format!("{date} {payee}\n  {to_acct}\t\t{to_amount} {to_currency}\n  {from_acct}\t\t{from_amount} {from_currency}\n\n",
        date = txn.date,
        payee = txn.txtype,
        to_acct = shares_account,
        to_amount = shares,
        to_currency = txn.investment,
        from_acct = cash_account,
        from_amount = cash,
        from_currency = "USD"
    )
}

#[test]
fn test_format() {
    let txn = Transaction {
        date: NaiveDate::parse_from_str("04/28/2017", "%m/%d/%Y").unwrap(),
        investment: String::from("FOOS"),
        txtype: String::from("CONTRIBUTIONS"),
        amount: Money::of_major(USD, 15),
        share: Money::of_major(share_currency(), 2),
    };
    println!("{}", format_txn_as_ledger(txn, String::from("Assets:Cash"), String::from("Assets:Investments")));
}

#[test]
fn test_share_positive() {
    assert_eq!(Ok(Money::of_major_minor(share_currency(), 15, 83)),
               share_formatter().parser().parse(add_currency_symbol("15.083").as_str()));
}

#[test]
fn test_share_negative() {
    assert_eq!(Ok(Money::of_major_minor(share_currency(), -15, -83)),
               share_formatter().parser().parse(add_currency_symbol("-15.083").as_str()));
}

#[test]
fn test_positive() {
    assert_eq!(Ok(Money::of_major_minor(USD, 15, 8)),
               dollar_formatter().parser().parse(add_currency_symbol("15.08").as_str()));
}
#[test]
fn test_negative() {
    assert_eq!(Ok(Money::of_major_minor(USD, -15, -8)),
               dollar_formatter().parser().parse(add_currency_symbol("-15.08").as_str()));
}
#[test]
fn test_commas() {
    assert_eq!(Ok(Money::of_major(USD, 1500)),
               dollar_formatter().parser().parse(add_currency_symbol("1,500.00").as_str()));
}
#[test]
fn test_negative_commas() {
    assert_eq!(Ok(Money::of_major(USD, -1500)),
               dollar_formatter().parser().parse(add_currency_symbol("-1,500.00").as_str()));
}
#[test]
fn test_workaround() {
    assert_eq!(Ok(Money::of_major(USD, 1)),
               dollar_formatter().parser().parse(add_currency_symbol("1.00").as_str()));
}