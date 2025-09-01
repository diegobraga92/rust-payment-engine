use crate::domain::ClientAccount;
use rusqlite::{Connection, params};
use serde_rusqlite::{from_rows, to_params_named};
use std::error::Error;

pub struct ClientAccountDB {
    conn: Connection,
}

impl ClientAccountDB {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS client_accounts (
                client INTEGER PRIMARY KEY,
                available REAL NOT NULL,
                held REAL NOT NULL,
                total REAL NOT NULL,
                locked BOOL
            )",
            [],
        )?;

        Ok(ClientAccountDB { conn })
    }

    pub fn does_account_exist(&self, client_id: u16) -> Result<bool, Box<dyn Error>> {
        let mut stmt = self
            .conn
            .prepare("SELECT COUNT(*) FROM client_accounts WHERE client = ?")?;
        let count: u32 = stmt.query_row(params![client_id], |row| row.get(0))?;
        Ok(count > 0)
    }

    pub fn include_client_account(&self, account: &ClientAccount) -> Result<(), Box<dyn Error>> {
        self.conn
            .execute(
                "INSERT INTO client_accounts (client, available, held, total, locked)
             VALUES (:client, :available, :held, :total, :locked)",
                to_params_named(&account).unwrap().to_slice().as_slice(),
            )
            .unwrap();

        Ok(())
    }

    pub fn update_client_account(&self, account: &ClientAccount) -> Result<(), Box<dyn Error>> {
        self.conn.execute(
            "UPDATE client_accounts SET available = :available, held = :held, total = :total, locked = :locked
             WHERE client = :client",
            to_params_named(&account).unwrap().to_slice().as_slice()).unwrap();

        Ok(())
    }

    pub fn get_account(&self, client_id: u16) -> Result<ClientAccount, Box<dyn Error>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM client_accounts WHERE client = ?")?;
        let account = from_rows::<ClientAccount>(stmt.query(params![client_id]).unwrap())
            .next()
            .unwrap()?;
        Ok(account)
    }

    // Get all client accounts. As Client ID is u16, everything can be loaded to memory safely
    pub fn get_all_accounts(&self) -> Result<Vec<ClientAccount>, Box<dyn Error>> {
        let mut stmt = self.conn.prepare("SELECT * FROM client_accounts")?;
        let accounts = from_rows::<ClientAccount>(stmt.query([]).unwrap())
            .collect::<Result<Vec<ClientAccount>, _>>()?;
        Ok(accounts)
    }
}
