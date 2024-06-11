
# Summary

Implements logic for checking accounts delegated state into `AccountLockState`
Accounts are read from a `Pubkey` using `AccountProvider`

# Details

*Important symbols:*

- `DelegationAccount` enum
  - can be Valid or Invalid
  - contains `DelegationRecord` (from core) if valid

- `DelegationRecordParser` trait
  - allows parsing a blob into a `DelegationRecord`

- `AccountLockState` enum
  - can be NewAccount / Delegated / Undelegated / Inconsistent

- `AccountLockStateProvider` struct
  - depends on an `AccountProvider`
  - depends on a `DelegationRecordParser`
  - can read a `Pubkey` -> `DelegationAccount` -> `AccountLockState`

# Notes

*Important dependencies:*

- Provides `AccountProvider` and `DelegationRecord`: [core](../core/README.md)
