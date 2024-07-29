
# Summary

The main purpose is to process a transaction and pull out information for each account used in it.
Also provides account validation implementation.

# Details

It is used by the validator to fetch information for if/how to clone accounts via `ValidatedAccounts`.
Internally uses an intermediary representation for the accounts: `TransactionAccountMetas`.
Help the director route a transaction properly by computing an `Endpoint`.

*Important symbols:*

- `TransactionAccountsHolder` struct
  - Parsed transaction pubkey Vecs

- `TransactionAccountsExtractor` trait
  - allow conversion from solana transactions to `TransactionAccountsHolder`

- `ValidatedAccounts` struct
  - classified accounts with meta info and delegation state

- `ValidatedAccountsProvider` trait
  - Computes `TransactionAccountsHolder` -> `TransactionAccountMetas` -> `ValidatedAccounts`

- `TransactionAccountMeta` struct
  - enum of Writable or Readable
  - contains `AccountChainSnapshot` chain account data and delegation state

- `TransactionAccountMetas` struct
  - vec of `TransactionAccountMeta`

- `Endpoint` enum
  - enum Chain or Ephemeral or Unroutable

- `Transwise` struct
  - implements `ValidatedAccountsProvider`
  - depends on an `AccountChainSnapshotProvider`
  - Computes solana transaction -> `TransactionAccountMetas` -> `Endpoint`

# Notes

*Important dependencies:*

- Provides `AccountChainSnapshotProvider`: [lockbox](../lockbox/README.md)
