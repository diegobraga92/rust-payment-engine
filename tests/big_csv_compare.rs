use std::fs::File;
use std::io::{Read};
use std::path::Path;
use rust_payment_engine::db::{ClientAccountDB, TransactionDB};
use rust_payment_engine::csv_processor::{process_csv, get_all_accounts_as_csv};

#[test]
fn test_big_csv_matches_expected_output() {
    let input_path = Path::new("tests/resources/big_input.csv");
    let output_path = Path::new("tests/resources/big_output.csv");
    let file = File::open(input_path).expect("Failed to open input CSV");
    let transaction_db = TransactionDB::new(":memory:").expect("Failed to create TransactionDB");
    let client_account_db = ClientAccountDB::new(":memory:").expect("Failed to create ClientAccountDB");
    process_csv(file, &transaction_db, &client_account_db).expect("Failed to process CSV");
    let actual = get_all_accounts_as_csv(&client_account_db).expect("Failed to get output CSV");
    let mut expected = String::new();
    File::open(output_path).expect("Failed to open expected output CSV").read_to_string(&mut expected).expect("Failed to read expected output");
    // Normalize line endings for comparison
    let actual = actual.replace("\r\n", "\n");
    let expected = expected.replace("\r\n", "\n");
    assert_eq!(actual.trim(), expected.trim(), "Output CSV does not match expected");
}
