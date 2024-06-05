
# Summary

Responsible for initializing and instantiating both the RPC and PUBSUB service.

# Details

Actual code for both services can be found in separate crates.
This crate is just a wrapper that initialize both services.

# Notes

*Important dependencies:*

 - Actual PUBSUB implementation: [director-pubsub](../director-pubsub/README.md)
 - Actual RPC implementation: [director-rpc](../director-rpc/README.md)
