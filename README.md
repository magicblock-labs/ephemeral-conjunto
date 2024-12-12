# conjunto

Orchestrating MagicBlock's machinery to abstract away underlying ephemeral validator, write
back and proxying.

## Director

The director solves the following problem for all RPC requests that come in and solves it as
follows:

- it decides if this request is better served via the ephemeral validator or the chain
- to make those decisions it considers the accounts involved in the transaction
- i.e. for `sendTransaction` it makes sure that:
  - if none of them are delegated, transaction is sent to chain
  - if all of them are delegated, transaction is sent to ephemeral
  - in the case of a mix it declares the request as _unroutable_
- other RPC methods need to be handled similarly, see `director-rpc/src/rpc/passthrough.rs`

For Pubsub methods it solves this in a different way:

- all messages from the chain and ephemeral backends are routed into the single client socket
- when the client subscribes those subscriptions are forwarded to the correct backend or both
depending on the accounts/signatures involved.