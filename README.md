# Malipo Payments Engine

A simple toy payments engine.

Reads a series of transactions from a CSV, updates client accounts, handles disputes and chargebacks, and then outputs the state of clients accounts as a CSV.

## Code Layout
--

The implementation uses a component based approach.

The core module defines common types and component interaces that are used in the actual
component implementations.

This approach allows for different implementations of components to be used with the payment engine
without requiring the engine to change. Components can be swapped out.

Project Layout:
- `src/domain.rs` : common data types and components traits.
- `src/engine.rs` : payment transactions processor.
- `src/errors.rs` : errors enumerations.
- `src/main.rs` : Command Line Interace.
- `src/store.rs` : data storage implementation.

## Design Decisions
--
* a trait is used to find the data storage interface.

This allows for different storage implementations to used e.g. in-memory stores can be used for
testing and actual production-ready stores used when code is deployed.

The trait is self is kept simple to allow multiple implementations.

* core logic is implemented by domain objects

The domain logic is implemented alongside the domain objects themselves i.e. accounts and
transactions. This means other components such as the data stores do not need to be concerned with
domain specific logic.

* engine expects traits instead of concrete types

This allows for different trait implementations to be used without having to rewrite the engine
code.

## Error Handling
--
All possible errors are modeled into a single enum `MalipoError` which
guarantees a single error type and allows easy error propagation using the `?` syntax.

Expected errors such as insufficient funds are handled by pattern matching
over the errors enum.

An assumption made is that the input data is always well formed (e.g. a deposit transaction with always
have an amount) and unwrapping the amount is considered safe.

## Testing
--
There are end-to-end test scenarios covering various transactions sequences.
The tests are implemented as a module of the binary `main.rs`.

Running tests:
- `cargo test`

## Executing
--
- `cargo run -- transactions.csv > accounts.csv`


## Further Work
--
* Swap out the In Memory Store if a more robust data storage engine.
