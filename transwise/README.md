
# Summary

The main purpose of this crate is to process transactions accounts information.

# Details

It is used by the validator to check if a transaction is valid using `TransactionAccountsValidator`.
Internally uses an intermediary representation for the transactions accounts: `TransactionAccountsSnapshot`.
Help the director route a transaction properly by computing an `Endpoint`.

*Important symbols:*

- `TransactionAccountsHolder` struct
  - Parsed transaction pubkey Vecs

- `TransactionAccountsExtractor` trait
  - allow conversion from solana transactions to `TransactionAccountsHolder`

- `TransactionAccountsSnapshot` struct
  - readonly and writable vecs of `AccountChainSnapshot`
  - can be fetched from a `TransactionAccountsHolder` using a `AccountChainSnapshotProvider`

- `TransactionAccountsValidator` trait
  - takes a `TransactionAccountsSnapshot` and check if it can be a valid ephemeral transaction

- `Endpoint` enum
  - enum Chain or Ephemeral or Unroutable
  - can be created from a `TransactionAccountsSnapshot`

- `Transwise` struct
  - Internally uses an `AccountChainSnapshotProvider`
  - Also allows conversion from solana transaction -> `Endpoint`

# Notes

*Important dependencies:*

- Provides `AccountChainSnapshot` and `AccountChainSnapshotProvider`: [lockbox](../lockbox/README.md)
