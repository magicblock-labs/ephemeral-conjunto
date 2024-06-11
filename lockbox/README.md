
# Summary

Implements logic for checking accounts delegated state into `AccountChainState`
Accounts are read from a `Pubkey` using `AccountProvider`

# Details

*Important symbols:*

- `DelegationAccount` enum
  - can be Valid or Invalid
  - contains `DelegationRecord` (from core) if valid

- `DelegationRecordParser` trait
  - allows parsing a blob into a `DelegationRecord`

- `AccountChainState` enum
  - can be NewAccount / Delegated / Undelegated / Inconsistent
  - contains the `Account` data and the delegation configuration if available

- `AccountChainStateProvider` struct
  - depends on an `AccountProvider`
  - depends on a `DelegationRecordParser`
  - can read a `Pubkey` -> `Account` + `DelegationAccount` -> `AccountChainState`

# Notes

*Important dependencies:*

- Provides `AccountProvider` and `DelegationRecord`: [core](../core/README.md)
