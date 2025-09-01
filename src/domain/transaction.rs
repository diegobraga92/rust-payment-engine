use serde::{Deserialize, Serialize};

// TODO: Handle all transaction types

/// Each transaction type corresponds to a specific action on the account
///
/// # Types:
///
/// - Deposit: Increase Available and Total funds of Account
/// - Withdrawal: Decrease Available and Total funds from account
/// - Dispute: Client claim that transaction needs to be reversed. Done by TX, not amount
/// - Resolve: Resolve Dispute, releasing funds from Held to Available
/// - Chargeback: Withdrawn of TX Held funds. Freeze account when this happens
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// Transactions correspond to each row in the CSV
///
/// # Notes:
///
/// - The disputed field is not expected in the CSV, but is used to control eventual disputes
#[derive(Debug, Deserialize, Serialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub id: u32,
    #[serde(deserialize_with = "csv::invalid_option")]
    pub amount: Option<f64>,
    #[serde(deserialize_with = "csv::invalid_option", default)]
    pub disputed: Option<bool>,
}
