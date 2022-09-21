use std::collections::HashMap;

use crate::{Account, ClientId, Fallible, MalipoError, Store, Transaction, TransactionId};

/// In-memory store for accounts
#[derive(Debug, Clone, Default)]
pub struct AccountsMemStore(HashMap<ClientId, Account>);

impl AccountsMemStore {
    /// Create a new accounts store
    pub fn new() -> Self {
        Self::default()
    }
}

impl Store<ClientId, Account> for AccountsMemStore {
    fn create(&mut self, item: Account) -> Fallible<()> {
        self.update(item)
    }
    fn get(&mut self, id: ClientId) -> Fallible<Account> {
        Ok(self
            .0
            .entry(id)
            .or_insert_with(|| Account::new(id))
            .to_owned())
    }
    fn delete(&mut self, id: ClientId) -> Fallible<()> {
        self.0.remove(&id);
        Ok(())
    }
    fn update(&mut self, item: Account) -> Fallible<()> {
        self.0.insert(item.client_id, item);
        Ok(())
    }
    fn iter(&self) -> Fallible<Box<dyn Iterator<Item = Account> + '_>> {
        let iter = self.0.values().copied();
        Ok(Box::new(iter))
    }
}

/// In-memory store for Transactions
#[derive(Debug, Clone, Default)]
pub struct TransactionsMemStore(HashMap<TransactionId, Transaction>);

impl TransactionsMemStore {
    /// Create a new transactions store
    pub fn new() -> Self {
        Self::default()
    }
}

impl Store<TransactionId, Transaction> for TransactionsMemStore {
    fn create(&mut self, txn: Transaction) -> Fallible<()> {
        self.update(txn)
    }

    fn delete(&mut self, id: TransactionId) -> Fallible<()> {
        self.0.remove(&id);
        Ok(())
    }

    fn get(&mut self, id: TransactionId) -> Fallible<Transaction> {
        self.0
            .get(&id)
            .copied()
            .ok_or(MalipoError::TransactionNotFound(id))
    }

    fn update(&mut self, txn: Transaction) -> Fallible<()> {
        self.0.insert(txn.id, txn);
        Ok(())
    }

    fn iter(&self) -> Fallible<Box<dyn Iterator<Item = Transaction> + '_>> {
        let iter = self.0.values().copied();
        Ok(Box::new(iter))
    }
}

/// CSV Data Reader
pub struct CsvDataReader(csv::Reader<std::fs::File>);

impl CsvDataReader {
    /// Create new reader from a path
    pub fn new(fname: &str) -> Fallible<CsvDataReader> {
        let rdr = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_path(fname)
            .map_err(MalipoError::CsvError)?;
        Ok(CsvDataReader(rdr))
    }
}

impl Iterator for CsvDataReader {
    type Item = Fallible<Transaction>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut rec = csv::StringRecord::new();
        match self.0.read_record(&mut rec) {
            Err(e) => Some(Err(MalipoError::CsvError(e))),
            Ok(rec_read) => {
                if rec_read {
                    Some(rec.deserialize(None).map_err(MalipoError::CsvError))
                } else {
                    None
                }
            }
        }
    }
}

/// CSV Data to Stdout Writer
pub struct CsvWriterStdout;

impl CsvWriterStdout {
    #[cfg(not(debug_assertions))]
    /// Write accounts to stdout
    pub fn write<W: std::io::Write>(
        accounts: Box<dyn Iterator<Item = Account> + '_>,
        wtr: Option<W>,
    ) -> Fallible<()> {
        if let Some(w) = wtr {
            let mut writer = csv::Writer::from_writer(w);
            for acc in accounts {
                writer.serialize(acc).map_err(MalipoError::CsvError)?;
            }
            writer.flush()?;
        } else {
            let mut writer = csv::Writer::from_writer(std::io::stdout());
            for acc in accounts {
                writer.serialize(acc).map_err(MalipoError::CsvError)?;
            }
            writer.flush()?;
        };

        Ok(())
    }

    #[cfg(debug_assertions)]
    /// Write accounts to stdout
    pub fn write<W: std::io::Write>(
        accounts: Box<dyn Iterator<Item = Account> + '_>,
        wtr: Option<W>,
    ) -> Fallible<()> {
        let mut accounts: Vec<_> = accounts.collect();
        accounts.sort_by_key(|acc| acc.client_id);
        if let Some(w) = wtr {
            let mut writer = csv::Writer::from_writer(w);
            for acc in accounts {
                writer.serialize(acc).map_err(MalipoError::CsvError)?;
            }
            writer.flush()?;
        } else {
            let mut writer = csv::Writer::from_writer(std::io::stdout());
            for acc in accounts {
                writer.serialize(acc).map_err(MalipoError::CsvError)?;
            }
            writer.flush()?;
        };
        Ok(())
    }
}
