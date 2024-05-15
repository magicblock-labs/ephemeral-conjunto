# conjunto

Orchestrating MagicBlock's machinery to abstract away underlying ephemeral validator, write
back and proxying.

## Director

The director solves the following problem for all RPC requests that come in and solves it as
follows:

- it decides if this request is better served via the ephemeral validator or the chain one
- to make those decisions it considers the accounts involved
- i.e. for `sendTransaction` it makes sure that all writable accounts are either locked
- if none of them are it sends to chain, if all of them are locked it sends to the ephemeral
  validator
- in the case of a mix it declares the request as _unroutable_
- other RPC methods need to be handled similarly, see `director-rpc/src/rpc/passthrough.rs`

For Pubsub methods it solves this in a different way:

- all messages from the chain and ephemeral backends are routed into the single client socket
- when the client subscribes those subscriptions are forwarded to the correct backend or both
depending on the accounts/signatures involved

## Current Status

### RPC Server

- uses jsonrpsee which causes a bunch of issues, see `director-rpc/src/lib.rs`
- it also maybe too highlevel and we should consider replacing with a lower level
  implementation

#### Working Methods

- most methods are just passed through to chain for now, however I noted which strategy they
should _actually_ use, see `director-rpc/src/rpc/passthrough.rs`.
- once we decide if to use jsonrpsee or not we need to implement them
- they should make use of existing code, i.e. the `guidepoint` crate as much of possible, i.e.
  if the strategy is based on an account being present in the ephemeral validator

### Cors Support

- tried to add CORS support but ran into jsonrpsee issue, not worth investing more time until
we decide if we keep that module or go lower level

### Pubsub Server

- correctly guides all subscriptions by looking at method and accounts/signatures involved
- I'd consider this done at this point and am happy with the fairly low level approach
- CORS support may need to be added if we see issues in the browser
