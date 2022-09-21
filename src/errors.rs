use thiserror::Error;

use crate::domain::{ClientId, TransactionId};

/// A result where the error channel is MalipoError
pub type Fallible<T> = Result<T, MalipoError>;

#[derive(Debug, Error)]
/// All possible Malipo errors
pub enum MalipoError {
    /// Missing Account
    #[error("Acount not found for client id: {0}")]
    AccountNotFound(ClientId),

    /// Missing Transaction
    #[error("Transaction not found for id: {0}")]
    TransactionNotFound(TransactionId),

    /// Insufficient Funds
    #[error("Insufficient funds in account")]
    InsufficientAccountFunds,

    /// CSV Data Error
    #[error("Error when processing CSV data: {0}")]
    CsvError(csv::Error),

    /// IO Errors
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// String Parsing error
    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),
}
