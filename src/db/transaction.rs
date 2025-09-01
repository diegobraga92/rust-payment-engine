use crate::domain::Transaction;
use rusqlite::{Connection, params};
use serde_rusqlite::to_params_named;
use std::error::Error;

pub struct TransactionDB {
    conn: Connection,
}

impl TransactionDB {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                type TEXT NOT NULL,
                client INTEGER NOT NULL,
                tx INTEGER PRIMARY KEY,
                amount REAL,
                disputed BOOLEAN
            )",
            [],
        )?;

        Ok(TransactionDB { conn })
    }

    pub fn include_transaction(&self, tx: &Transaction) -> Result<(), Box<dyn Error>> {
        self.conn
            .execute(
                "INSERT INTO transactions (type, client, tx, amount, disputed)
             VALUES (:type, :client, :tx, :amount, :disputed)",
                to_params_named(&tx).unwrap().to_slice().as_slice(),
            )
            .unwrap();

        Ok(())
    }

    pub fn mark_disputed(&self, id: u32, is_disputed: bool) -> Result<(), Box<dyn Error>> {
        self.conn.execute(
            "UPDATE transactions SET disputed = ? WHERE tx = ?",
            params![is_disputed, id],
        )?;
        Ok(())
    }

    pub fn get_amount(&self, id: u32) -> Result<Option<f64>, Box<dyn Error>> {
        let mut stmt = self
            .conn
            .prepare("SELECT amount FROM transactions WHERE tx = ?")?;
        let amount: Option<f64> = stmt.query_row(params![id], |row| row.get(0))?;
        Ok(amount)
    }
}
