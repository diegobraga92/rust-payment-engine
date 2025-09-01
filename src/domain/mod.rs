mod client_account;
mod transaction;

pub use client_account::ClientAccount;
pub use transaction::{Transaction, TransactionType};

// Custom serializer for 4 decimal places
use serde::Serializer;
pub fn serialize_f64_4<S>(x: &f64, s: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	s.serialize_str(&format!("{:.4}", x))
}
