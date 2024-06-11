
# Summary

Starts an HTTP server on 127.0.0.1:9899.
Dynamically route requests either to the Chain RPC or the Ephemeral RPC.
Requests are routed transparently based on type (and content).

# Details

Uses a HTTP RPC server implemented using `jsonrpsee` open-source crate.

Any request from the client is conditionally routed to either:
- the RPC of the "chain" (Solana)
- the RPC of the "ephem" (Validator)

The routing is done using `Transwise` logic.

Any response from "chain" or "ephem" is sent directly back to the client

*Important symbols:*

- `DirectorRpc` struct
  - depends on a `Transwise`
  - contains `HttpClient` for both "chain" and "ephem"

- `register_passthrough_methods` function
  - Register HTTP routes on the `DirectorRpc`'s `RpcModule` that can be passthrough
  - All those routes defined as passthrough simply proxy all requests to the chain's RPC

- `register_guide_methods` function
  - Define the RPC's method that needs to be routed (guided) dynamically
  - For those methods, parse the received message, then do the routing
  - for the "sendTransaction" method specifically, we decode the transaction then route it by asking `Transwise` where to send it
  - Send the same message to the desired RPC (either Chain or Ephemeral)

# Notes

*Important dependencies:*

- Provides `Transwise`: [transwise](../transwise/README.md)
