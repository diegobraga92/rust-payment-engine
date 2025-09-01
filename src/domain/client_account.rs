use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientAccount {
    #[serde(rename = "client")]
    id: u16,
    #[serde(serialize_with = "crate::domain::serialize_f64_4")] 
    available: f64,
    #[serde(serialize_with = "crate::domain::serialize_f64_4")] 
    held: f64,
    #[serde(serialize_with = "crate::domain::serialize_f64_4")] 
    total: f64, // Available + Held
    locked: bool,
}

impl ClientAccount {
    pub fn new(id: u16) -> Self {
        ClientAccount {
            id,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn add_funds(&mut self, amount: f64) {
        self.available += amount;
        self.total += amount;
    }

    pub fn withdraw_funds(&mut self, amount: f64) -> Result<(), String> {
        if self.available >= amount {
            self.available -= amount;
            self.total -= amount;
            Ok(())
        } else {
            Err("Insufficient funds".to_string())
        }
    }

    pub fn hold_funds(&mut self, amount: f64) -> Result<(), String> {
        if self.available >= amount {
            self.available -= amount;
            self.held += amount;
            Ok(())
        } else {
            Err("Insufficient funds".to_string())
        }
    }

    pub fn resolve_funds(&mut self, amount: f64) -> Result<(), String> {
        if self.held >= amount {
            self.held -= amount;
            self.available += amount;
            Ok(())
        } else {
            Err("Insufficient held funds".to_string())
        }
    }

    pub fn withdraw_from_held(&mut self, amount: f64) -> Result<(), String> {
        if self.held >= amount {
            self.held -= amount;
            self.total -= amount;
            Ok(())
        } else {
            Err("Insufficient held funds".to_string())
        }
    }

    pub fn lock_account(&mut self) {
        self.locked = true;
    }
}
