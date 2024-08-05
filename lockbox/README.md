
# Summary

Implements logic for checking accounts delegated state into `AccountChainState`
Accounts are read from a `Pubkey` using `AccountProvider`

# Details

*Important symbols:*

- `DelegationRecordParser` trait
  - allows parsing a blob into a `DelegationRecord`

- `AccountChainSnapshot` struct
  - contains a `Slot` and a `AccountChainState`

- `AccountChainState` enum
  - can be NewAccount / Delegated / Undelegated / Inconsistent
  - contains the `Account` data and the delegation configuration if available

- `AccountChainSnapshotProvider` struct
  - depends on an `AccountProvider`
  - depends on a `DelegationRecordParser`
  - can read a `Pubkey` -> `Account` + `DelegationRecord` -> `AccountChainSnapshot`

# Notes

*Important dependencies:*

- Provides `AccountProvider` and `DelegationRecord`: [core](../core/README.md)
