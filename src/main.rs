//! Simple Payment Engine
//!
//! CLI tool that simulates a Payment Engine
//! Receiving transaction data in CSV format, and output final Client Accounts to Std Out (in CSV Format)
//!
//! ## Usage:
//! ```
//! cargo run -- transactions.csv > accounts.csv
//! ```
//!
//! ## CSV Input File:
//! Input CSV must have the following fields: type of transaction, client ID, transaction ID, amount. E.g.:
//! ```
//! type, client, tx, amount
//! deposit, 1, 1, 2.0
//! withdrawal, 1, 2, 1.5
//! ```
//!
//! ## Supported Transaction Types:
//! - Deposit: Increase funds
//! - Withdrawal: Decrease Available funds, if enough
//! - Dispute: Mark transaction for reversal investigation. (Done by tx, amount not needed)
//! - Resolve: Resolves dispute, making funds available. (Done by tx, amount not needed)
//! - Chargeback: Withdraw funds under dispute. Account is locked afterwards. (Done by tx, amount not needed)
//!
//! ## Implementation
//! Implementation details on README.md

mod csv_processor;
mod db;
mod domain;

use crate::db::{ClientAccountDB, TransactionDB};
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;

fn run() -> Result<(), Box<dyn Error>> {
    let file_path = get_first_arg()?;
    let file = File::open(file_path)?;

    let transaction_db_path = "transactions.db";
    let client_account_db_path = "client_accounts.db";
    let transaction_db = TransactionDB::new(transaction_db_path)?;
    let client_account_db = ClientAccountDB::new(client_account_db_path)?;

    csv_processor::process_csv(file, &transaction_db, &client_account_db)?;

    let accounts_csv = csv_processor::get_all_accounts_as_csv(&client_account_db)?;
    println!("{}", accounts_csv);

    // Delete the .db files at the end of the run to make re-testing easier
    let _ = std::fs::remove_file(transaction_db_path);
    let _ = std::fs::remove_file(client_account_db_path);

    Ok(())
}

fn get_first_arg() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expected 1 argument, but got none")),
        Some(file_path) => Ok(file_path),
    }
}

fn main() {
    if let Err(err) = run() {
        println!("{}", err);
    }
}
