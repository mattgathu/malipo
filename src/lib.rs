#![warn(missing_docs)]
//! Malipo payments engine
mod domain;
mod engine;
mod errors;
mod store;

pub use crate::domain::*;
pub use crate::errors::{Fallible, MalipoError};
pub use crate::store::{AccountsMemStore, CsvDataReader, CsvWriterStdout, TransactionsMemStore};
pub use engine::PaymentsEngine;
