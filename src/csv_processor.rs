use crate::db::{ClientAccountDB, TransactionDB};
use crate::domain::{ClientAccount, Transaction, TransactionType};
use csv::{ReaderBuilder, Trim, WriterBuilder};
use std::{error::Error, fs::File};

// Only keep track of Deposit and Withdrawal, as other operations interact with those two
fn add_transaction_to_db(tx: &Transaction, db: &TransactionDB) -> Result<(), Box<dyn Error>> {
    db.include_transaction(tx)?;
    Ok(())
}

fn process_transaction(
    tx: &Transaction,
    transaction_db: &TransactionDB,
    client_account_db: &ClientAccountDB,
) -> Result<(), Box<dyn Error>> {
    let account_exists = client_account_db.does_account_exist(tx.client_id)?;

    if !account_exists {
        let new_account = ClientAccount::new(tx.client_id);
        client_account_db.include_client_account(&new_account)?;
    }

    let mut account = client_account_db.get_account(tx.client_id)?;

    // Skip any transaction if account is locked
    if account.is_locked() {
        return Ok(());
    }

    match tx.transaction_type {
        TransactionType::Deposit => {
            if let Some(amount) = tx.amount {
                account.add_funds(amount);
                client_account_db.update_client_account(&account)?;
                add_transaction_to_db(&tx, transaction_db)?;
            }
        }
        TransactionType::Withdrawal => {
            if let Some(amount) = tx.amount {
                if account.withdraw_funds(amount).is_ok() {
                    client_account_db.update_client_account(&account)?;
                }
                add_transaction_to_db(&tx, transaction_db)?;
            }
        }
        TransactionType::Dispute => {
            if let Some(amount) = transaction_db.get_amount(tx.id)? {
                if account.hold_funds(amount).is_ok() {
                    client_account_db.update_client_account(&account)?;
                    transaction_db.mark_disputed(tx.id, true)?;
                }
            }
        }
        TransactionType::Resolve => {
            if let Some(amount) = transaction_db.get_amount(tx.id)? {
                if account.resolve_funds(amount).is_ok() {
                    client_account_db.update_client_account(&account)?;
                    transaction_db.mark_disputed(tx.id, false)?;
                }
            }
        }
        TransactionType::Chargeback => {
            if let Some(amount) = transaction_db.get_amount(tx.id)? {
                if account.withdraw_from_held(amount).is_ok() {
                    account.lock_account();
                    client_account_db.update_client_account(&account)?;
                }
            }
        }
    }

    Ok(())
}

pub fn process_csv(
    file: File,
    transaction_db: &TransactionDB,
    client_account_db: &ClientAccountDB,
) -> Result<(), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().trim(Trim::All).from_reader(file); // Use Trim::All to remove possible whitespaces
    for result in rdr.deserialize() {
        let record: Transaction = result?;
        process_transaction(&record, transaction_db, client_account_db)?;
    }
    Ok(())
}

pub fn get_all_accounts_as_csv(
    client_account_db: &ClientAccountDB,
) -> Result<String, Box<dyn Error>> {
    let mut wtr = WriterBuilder::new().from_writer(vec![]);
    for account in client_account_db.get_all_accounts()? {
        wtr.serialize(account)?;
    }
    let data = String::from_utf8(wtr.into_inner()?)?;
    Ok(data)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{TransactionType};
    use std::cell::RefCell;
    use std::collections::HashMap;

    // Simple in-memory mocks for DBs and accounts
    #[derive(Default, Clone)]
    struct MockClientAccount {
        pub id: u16,
        pub available: f64,
        pub held: f64,
        pub locked: bool,
    }

    impl MockClientAccount {
        fn new(id: u16) -> Self {
            Self { id, available: 0.0, held: 0.0, locked: false }
        }
        fn add_funds(&mut self, amount: f64) { self.available += amount; }
        fn withdraw_funds(&mut self, amount: f64) -> Result<(), ()> {
            if self.available >= amount { self.available -= amount; Ok(()) } else { Err(()) }
        }
        fn hold_funds(&mut self, amount: f64) -> Result<(), ()> {
            if self.available >= amount { self.available -= amount; self.held += amount; Ok(()) } else { Err(()) }
        }
        fn resolve_funds(&mut self, amount: f64) -> Result<(), ()> {
            if self.held >= amount { self.held -= amount; self.available += amount; Ok(()) } else { Err(()) }
        }
        fn withdraw_from_held(&mut self, amount: f64) -> Result<(), ()> {
            if self.held >= amount { self.held -= amount; Ok(()) } else { Err(()) }
        }
        fn lock_account(&mut self) { self.locked = true; }
        fn is_locked(&self) -> bool { self.locked }
    }

    struct MockClientAccountDB {
        accounts: RefCell<HashMap<u16, MockClientAccount>>,
    }
    impl MockClientAccountDB {
        fn new() -> Self { Self { accounts: RefCell::new(HashMap::new()) } }
        fn does_account_exist(&self, id: u16) -> Result<bool, Box<dyn Error>> {
            Ok(self.accounts.borrow().contains_key(&id))
        }
        fn include_client_account(&self, acc: &MockClientAccount) -> Result<(), Box<dyn Error>> {
            self.accounts.borrow_mut().insert(acc.id, acc.clone()); Ok(())
        }
        fn get_account(&self, id: u16) -> Result<MockClientAccount, Box<dyn Error>> {
            Ok(self.accounts.borrow().get(&id).cloned().unwrap())
        }
        fn update_client_account(&self, acc: &MockClientAccount) -> Result<(), Box<dyn Error>> {
            self.accounts.borrow_mut().insert(acc.id, acc.clone()); Ok(())
        }
    }

    struct MockTransactionDB {
        txs: RefCell<HashMap<u32, (u16, Option<f64>, bool)>>, // id -> (client_id, amount, disputed)
    }
    impl MockTransactionDB {
        fn new() -> Self { Self { txs: RefCell::new(HashMap::new()) } }
        fn include_transaction(&self, tx: &Transaction) -> Result<(), Box<dyn Error>> {
            self.txs.borrow_mut().insert(tx.id, (tx.client_id, tx.amount, false)); Ok(())
        }
        fn get_amount(&self, id: u32) -> Result<Option<f64>, Box<dyn Error>> {
            Ok(self.txs.borrow().get(&id).and_then(|(_, amt, _)| *amt))
        }
        fn mark_disputed(&self, id: u32, disputed: bool) -> Result<(), Box<dyn Error>> {
            if let Some(entry) = self.txs.borrow_mut().get_mut(&id) { entry.2 = disputed; } Ok(())
        }
    }

    fn make_tx(id: u32, client_id: u16, tx_type: TransactionType, amount: Option<f64>) -> Transaction {
        Transaction { id, client_id, transaction_type: tx_type, amount, disputed: Some(false) }
    }

    #[test]
    fn test_deposit_creates_account_and_adds_funds() {
        let db = MockTransactionDB::new();
        let acc_db = MockClientAccountDB::new();
        let tx = make_tx(1, 1, TransactionType::Deposit, Some(1.23456));
        
        if !acc_db.does_account_exist(tx.client_id).unwrap() {
            acc_db.include_client_account(&MockClientAccount::new(tx.client_id)).unwrap();
        }
        let mut acc = acc_db.get_account(tx.client_id).unwrap();
        if !acc.is_locked() {
            if let Some(amount) = tx.amount {
                acc.add_funds(amount);
                acc_db.update_client_account(&acc).unwrap();
                db.include_transaction(&tx).unwrap();
            }
        }
        let acc = acc_db.get_account(1).unwrap();
        assert_eq!(acc.available, 1.23456);
    }

    #[test]
    fn test_withdrawal_succeeds_and_fails_on_insufficient_funds() {
        let db = MockTransactionDB::new();
        let acc_db = MockClientAccountDB::new();
        let mut acc = MockClientAccount::new(1);
        acc.add_funds(2.0);
        acc_db.include_client_account(&acc).unwrap();
        let tx = make_tx(2, 1, TransactionType::Withdrawal, Some(1.0));
        let mut acc = acc_db.get_account(1).unwrap();
        if !acc.is_locked() {
            if let Some(amount) = tx.amount {
                if acc.withdraw_funds(amount).is_ok() {
                    acc_db.update_client_account(&acc).unwrap();
                }
                db.include_transaction(&tx).unwrap();
            }
        }
        let acc = acc_db.get_account(1).unwrap();
        assert_eq!(acc.available, 1.0);

        // Now try to withdraw more than available
        let tx2 = make_tx(3, 1, TransactionType::Withdrawal, Some(2.0));
        let mut acc = acc_db.get_account(1).unwrap();
        if !acc.is_locked() {
            if let Some(amount) = tx2.amount {
                if acc.withdraw_funds(amount).is_ok() {
                    acc_db.update_client_account(&acc).unwrap();
                }
                db.include_transaction(&tx2).unwrap();
            }
        }
        let acc = acc_db.get_account(1).unwrap();
        assert_eq!(acc.available, 1.0, "Should not withdraw more than available");
    }

    #[test]
    fn test_deposit_on_locked_account_is_ignored() {
        let db = MockTransactionDB::new();
        let acc_db = MockClientAccountDB::new();
        let mut acc = MockClientAccount::new(1);
        acc.lock_account();
        acc_db.include_client_account(&acc).unwrap();
        let tx = make_tx(4, 1, TransactionType::Deposit, Some(5.0));
        let mut acc = acc_db.get_account(1).unwrap();
        if !acc.is_locked() {
            if let Some(amount) = tx.amount {
                acc.add_funds(amount);
                acc_db.update_client_account(&acc).unwrap();
                db.include_transaction(&tx).unwrap();
            }
        }
        let acc = acc_db.get_account(1).unwrap();
        assert_eq!(acc.available, 0.0, "Locked account should not accept deposits");
    }

    #[test]
    fn test_dispute_moves_funds_to_held() {
        let db = MockTransactionDB::new();
        let acc_db = MockClientAccountDB::new();
        let mut acc = MockClientAccount::new(1);
        acc.add_funds(10.0);
        acc_db.include_client_account(&acc).unwrap();
        let tx = make_tx(5, 1, TransactionType::Deposit, Some(10.0));
        db.include_transaction(&tx).unwrap();
        // Dispute
        let dispute_tx = make_tx(5, 1, TransactionType::Dispute, None);
        let mut acc = acc_db.get_account(1).unwrap();
        if !acc.is_locked() {
            if let Some(amount) = db.get_amount(dispute_tx.id).unwrap() {
                if acc.hold_funds(amount).is_ok() {
                    acc_db.update_client_account(&acc).unwrap();
                    db.mark_disputed(dispute_tx.id, true).unwrap();
                }
            }
        }
        let acc = acc_db.get_account(1).unwrap();
        assert_eq!(acc.available, 0.0);
        assert_eq!(acc.held, 10.0);
    }

    #[test]
    fn test_resolve_releases_held_funds() {
        let db = MockTransactionDB::new();
        let acc_db = MockClientAccountDB::new();
        let mut acc = MockClientAccount::new(1);
        acc.held = 5.0;
        acc_db.include_client_account(&acc).unwrap();
    
        db.txs.borrow_mut().insert(6, (1, Some(5.0), true));
        // Resolve
        let resolve_tx = make_tx(6, 1, TransactionType::Resolve, None);
        let mut acc = acc_db.get_account(1).unwrap();
        if !acc.is_locked() {
            if let Some(amount) = db.get_amount(resolve_tx.id).unwrap() {
                if acc.resolve_funds(amount).is_ok() {
                    acc_db.update_client_account(&acc).unwrap();
                    db.mark_disputed(resolve_tx.id, false).unwrap();
                }
            }
        }
        let acc = acc_db.get_account(1).unwrap();
        assert_eq!(acc.held, 0.0);
        assert_eq!(acc.available, 5.0);
    }

    #[test]
    fn test_chargeback_withdraws_held_and_locks_account() {
        let db = MockTransactionDB::new();
        let acc_db = MockClientAccountDB::new();
        let mut acc = MockClientAccount::new(1);
        acc.held = 7.0;
        acc_db.include_client_account(&acc).unwrap();
    
        db.txs.borrow_mut().insert(7, (1, Some(7.0), true));
        // Chargeback
        let chargeback_tx = make_tx(7, 1, TransactionType::Chargeback, None);
        let mut acc = acc_db.get_account(1).unwrap();
        if !acc.is_locked() {
            if let Some(amount) = db.get_amount(chargeback_tx.id).unwrap() {
                if acc.withdraw_from_held(amount).is_ok() {
                    acc.lock_account();
                    acc_db.update_client_account(&acc).unwrap();
                }
            }
        }
        let acc = acc_db.get_account(1).unwrap();
        assert_eq!(acc.held, 0.0);
        assert!(acc.is_locked());
    }

    #[test]
    fn test_dispute_on_nonexistent_tx_does_nothing() {
        let db = MockTransactionDB::new();
        let acc_db = MockClientAccountDB::new();
        let acc = MockClientAccount::new(1);
        acc_db.include_client_account(&acc).unwrap();
        let dispute_tx = make_tx(999, 1, TransactionType::Dispute, None);
        let mut acc = acc_db.get_account(1).unwrap();
        if !acc.is_locked() {
            if let Some(amount) = db.get_amount(dispute_tx.id).unwrap() {
                if acc.hold_funds(amount).is_ok() {
                    acc_db.update_client_account(&acc).unwrap();
                    db.mark_disputed(dispute_tx.id, true).unwrap();
                }
            }
        }
        let acc = acc_db.get_account(1).unwrap();
        assert_eq!(acc.held, 0.0);
        assert_eq!(acc.available, 0.0);
    }

    #[test]
    fn test_resolve_on_non_disputed_tx_does_nothing() {
        let db = MockTransactionDB::new();
        let acc_db = MockClientAccountDB::new();
        let mut acc = MockClientAccount::new(1);
        acc.held = 0.0;
        acc_db.include_client_account(&acc).unwrap();

        db.txs.borrow_mut().insert(8, (1, Some(5.0), false));
        // Resolve
        let resolve_tx = make_tx(8, 1, TransactionType::Resolve, None);
        let mut acc = acc_db.get_account(1).unwrap();
        if !acc.is_locked() {
            if let Some(amount) = db.get_amount(resolve_tx.id).unwrap() {
                if acc.resolve_funds(amount).is_ok() {
                    acc_db.update_client_account(&acc).unwrap();
                    db.mark_disputed(resolve_tx.id, false).unwrap();
                }
            }
        }
        let acc = acc_db.get_account(1).unwrap();
        assert_eq!(acc.held, 0.0);
        assert_eq!(acc.available, 0.0);
    }

    #[test]
    fn test_chargeback_on_non_disputed_tx_does_nothing() {
        let db = MockTransactionDB::new();
        let acc_db = MockClientAccountDB::new();
        let mut acc = MockClientAccount::new(1);
        acc.held = 0.0;
        acc_db.include_client_account(&acc).unwrap();
    
        db.txs.borrow_mut().insert(9, (1, Some(5.0), false));
        // Chargeback
        let chargeback_tx = make_tx(9, 1, TransactionType::Chargeback, None);
        let mut acc = acc_db.get_account(1).unwrap();
        if !acc.is_locked() {
            if let Some(amount) = db.get_amount(chargeback_tx.id).unwrap() {
                if acc.withdraw_from_held(amount).is_ok() {
                    acc.lock_account();
                    acc_db.update_client_account(&acc).unwrap();
                }
            }
        }
        let acc = acc_db.get_account(1).unwrap();
        assert_eq!(acc.held, 0.0);
        assert!(!acc.is_locked());
    }
}
