use crate::domain::*;
use crate::{Fallible, MalipoError};

/// Payments Engine
pub struct PaymentsEngine {
    accounts: Box<dyn Store<ClientId, Account>>,
    transactions: Box<dyn Store<TransactionId, Transaction>>,
}

impl PaymentsEngine {
    /// Creates an engine.
    pub fn new(
        accounts: Box<dyn Store<ClientId, Account>>,
        transactions: Box<dyn Store<TransactionId, Transaction>>,
    ) -> Self {
        Self {
            accounts,
            transactions,
        }
    }
    /// Execute a transaction
    pub fn execute_transaction(&mut self, txn: Transaction) -> Fallible<()> {
        match txn.type_ {
            TransactionType::Chargeback => self.chargeback(txn)?,
            TransactionType::Deposit => self.deposit(txn)?,
            TransactionType::Dispute => self.dispute(txn)?,
            TransactionType::Resolve => self.resolve(txn)?,
            TransactionType::Withdrawal => self.withdrawal(txn)?,
        }
        Ok(())
    }

    /// Get a stream if accounts from the store
    pub fn accounts(&self) -> Fallible<Box<dyn Iterator<Item = Account> + '_>> {
        self.accounts.iter()
    }

    /// A deposit is a credit to the client's asset account, meaning it should
    /// increase the available and total funds of the client account
    fn deposit(&mut self, txn: Transaction) -> Fallible<()> {
        let mut acc = self.accounts.get(txn.client_id)?;
        acc.deposit(txn.amount.unwrap());
        self.accounts.update(acc)?;
        self.transactions.create(txn)?;
        Ok(())
    }

    /// A withdraw is a debit to the client's asset account, meaning it should
    /// decrease the available and total funds of the client account
    fn withdrawal(&mut self, txn: Transaction) -> Fallible<()> {
        let mut acc = self.accounts.get(txn.client_id)?;
        match acc.withdraw(txn.amount.unwrap()) {
            Ok(_) => {}
            Err(MalipoError::InsufficientAccountFunds) => {}
            Err(e) => return Err(e),
        };
        self.accounts.update(acc)?;
        self.transactions.create(txn)?;
        Ok(())
    }

    /// A chargeback is the final state of a dispute and represents the client
    /// reversing a transaction. Funds that were held have now been withdrawn.
    /// This means that the clients held funds and total funds should decrease
    /// by the amount previously disputed. If a chargeback occurs the client's
    /// account should be immediately frozen
    fn chargeback(&mut self, txn: Transaction) -> Fallible<()> {
        match self.transactions.get(txn.id) {
            Ok(prev_txn) => {
                if prev_txn.is_disputed() {
                    let mut acc = self.accounts.get(txn.client_id)?;
                    acc.chargeback(prev_txn.amount.unwrap());
                    self.accounts.update(acc)?;
                }
            }
            Err(MalipoError::TransactionNotFound(_)) => {}
            Err(e) => return Err(e),
        }
        Ok(())
    }

    /// A dispute represents a client's claim that a transaction was erroneous
    /// and should be reversed. The transaction shouldn't be reversed yet but
    /// the associated funds should be held. This means that the clients available
    /// funds should decrease by the amount disputed, their held funds should
    /// increase by the amount disputed, while their total funds should remain the same.
    fn dispute(&mut self, txn: Transaction) -> Fallible<()> {
        match self.transactions.get(txn.id) {
            Err(MalipoError::TransactionNotFound(_)) => {}
            Err(e) => return Err(e),
            Ok(mut prev_txn) => {
                let mut acc = self.accounts.get(txn.client_id)?;
                acc.dispute(prev_txn.amount.unwrap());
                self.accounts.update(acc)?;
                prev_txn.mark_as_disputed();
                self.transactions.update(prev_txn)?;
            }
        }
        Ok(())
    }

    /// A resolve represents a resolution to a dispute, releasing the associated
    /// held funds. Funds that were previously disputed are no longer disputed.
    /// This means that the clients held funds should decrease by the amount no
    /// longer disputed, their available funds should increase by the amount no
    /// longer disputed, and their total funds should remain the same.
    fn resolve(&mut self, txn: Transaction) -> Fallible<()> {
        match self.transactions.get(txn.id) {
            Ok(mut prev_txn) => {
                if prev_txn.is_disputed() {
                    let mut acc = self.accounts.get(txn.client_id)?;
                    acc.resolve(prev_txn.amount.unwrap());
                    self.accounts.update(acc)?;
                    prev_txn.resolve_dispute();
                    self.transactions.update(prev_txn)?;
                }
            }
            Err(MalipoError::TransactionNotFound(_)) => {}
            Err(e) => return Err(e),
        }
        Ok(())
    }
}
