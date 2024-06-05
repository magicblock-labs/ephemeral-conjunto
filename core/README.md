
# Summary

This crate declares types and traits used across the whole codebase.
Doesn't contain any logic directly.

# Details

*Important symbols:*

- `GuideStrategy`/`RequestEndpoint` enums
  - Which endpoint to propagate a request to
  - can be Chain/Ephemeral/Both

- `DelegationRecord` struct
  - Account owner's pubkey
  - `CommitFrequency` frequency at which the account's state is commited to chain

- `AccountsHolder` trait
  - Writable/Readonly/Payer store for Pubkeys (those accounts are pull out of the transaction)

- `AccountProvider` trait
  - get_account(Pubkey) -> Account

- `SignatureStatusProvider` trait
  - get_signature_status(Signature) -> Result

# Notes

This crate is supposed to be importable from everywhere.
It is not supposed to have any dependency.

This crate does not provide concrete implementation for any of the traits,
so that they can also be stubbed in unit tests without using the production implementation.
