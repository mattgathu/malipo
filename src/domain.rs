use crate::{Fallible, MalipoError};
use serde::{Deserialize, Serialize, Serializer};

/// Client ID
pub type ClientId = u16;
/// Transaction ID
pub type TransactionId = u32;
/// Monetary Amount
pub type Amount = f64;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Transaction Type
pub enum TransactionType {
    /// Charge back
    Chargeback,
    /// Deposit
    Deposit,
    /// Dispute
    Dispute,
    /// Resolve
    Resolve,
    /// Withdrawal
    Withdrawal,
}

#[derive(Debug, Default, Clone, Copy, Serialize)]
/// Client's Account
pub struct Account {
    #[serde(rename = "client")]
    /// Client ID
    pub client_id: ClientId,
    #[serde(serialize_with = "ser_float")]
    available: Amount,
    #[serde(serialize_with = "ser_float")]
    held: Amount,
    #[serde(serialize_with = "ser_float")]
    total: Amount,
    #[serde(default)]
    locked: bool,
}
impl Account {
    /// Create a new account
    pub fn new(client_id: ClientId) -> Self {
        Self {
            client_id,
            ..Default::default()
        }
    }

    /// Perform a chargeback on this account
    pub fn chargeback(&mut self, amount: Amount) {
        self.held -= amount;
        self.total -= amount;
        self.locked = true;
    }
    /// Deposit into this account
    pub fn deposit(&mut self, amount: Amount) {
        self.available += amount;
        self.total += amount;
    }

    /// Perform a dispute of the amount on this account
    pub fn dispute(&mut self, amount: Amount) {
        self.held += amount;
        self.available -= amount;
    }

    /// Resolve a dispute
    pub fn resolve(&mut self, amount: Amount) {
        self.held -= amount;
        self.available += amount;
    }

    /// Withdraw funds from account
    pub fn withdraw(&mut self, amount: Amount) -> Fallible<()> {
        if amount > self.available {
            return Err(MalipoError::InsufficientAccountFunds);
        }
        self.available -= amount;
        self.total -= amount;
        Ok(())
    }

    /// Check if account is frozen/locked
    pub fn is_frozen(&self) -> bool {
        self.locked
    }

    /// check that account invariants are not violated
    #[cfg(debug_assertions)]
    pub fn check_invariants(&self) {
        assert!(self.total >= self.available);
        assert!((self.total - (self.available + self.held)).abs() < f64::EPSILON);
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
/// Transaction
pub struct Transaction {
    #[serde(rename = "type")]
    /// Transaction type
    pub type_: TransactionType,
    #[serde(rename = "client")]
    /// Client ID
    pub client_id: ClientId,
    #[serde(rename = "tx")]
    /// Transaction ID
    pub id: TransactionId,
    #[serde(default)]
    /// Amount
    pub amount: Option<Amount>,
    #[serde(default)]
    #[serde(skip)]
    disputed: bool,
}
impl Transaction {
    /// Mark a transaction as disputed
    pub fn mark_as_disputed(&mut self) {
        self.disputed = true;
    }
    /// Check if a transaction is in dispute
    pub fn is_disputed(&self) -> bool {
        self.disputed
    }
    /// Resolve a dispute
    pub fn resolve_dispute(&mut self) {
        self.disputed = false;
    }
}

/// Store Interface
pub trait Store<Id, Item> {
    /// Store a new item in the store
    fn create(&mut self, item: Item) -> Fallible<()>;
    /// Delete an item using its ID
    fn delete(&mut self, id: Id) -> Fallible<()>;
    /// Get an item using its ID
    fn get(&mut self, id: Id) -> Fallible<Item>;
    /// Update an item
    fn update(&mut self, item: Item) -> Fallible<()>;
    /// An iterator over all items in the store
    fn iter(&self) -> Fallible<Box<dyn Iterator<Item = Item> + '_>>;
}

/// Serialize floats
pub fn ser_float<S: Serializer>(float: &f64, serializer: S) -> Result<S::Ok, S::Error> {
    let float_as_str = format!("{:.4}", float);
    serializer.serialize_str(&float_as_str)
}
