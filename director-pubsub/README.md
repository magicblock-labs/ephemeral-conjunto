
# Summary

Start a TCP PubSub service on 127.0.0.1:9900.
Dynamically route requests either to the Chain PubSub or the Ephemeral PubSub.
Requests are routed transparently based on their content and context.

# Details

Uses websocket implemented using `tokio_tungstenite` open source crate.

Any request from the client is conditionally routed to either:

- the websocket of the "chain" (Solana)
- the websocket of the "ephem" (Validator)
- Sometimes both

This routing is done using some "guide" logic implemented in this crate.

Any response from "chain" or "ephem" is sent directly back to the client

*Important symbols:*

- `accept_connection` function
  - Basically the main loop function for the service (using tokio)
  - Takes in parameter `DirectorPubsub` and tcps/websockets
  - Read from all streams and write to appropriate stream for each messages
  - Uses the `DirectorPubsub` for routing requests and simple forward for responses

- `DirectorPubsub` struct
  - depends on a `GuideStrategyResolver`
  - can convert `Message` -> `GuideStrategy` -> `RequestEndpoint`
  - using `guide_strategy_from_pubsub_msg`

- `ParsedClientMessage` enum
  - Parsed representation of a raw websocket message
  - Can be parsed from a raw message string
  - Uses serde to parse the JSON messages

- `guide_strategy_from_pubsub_msg` function
  - Takes in parameter a message, parses it to a `ParsedClientMessage`
  - Compute the expected `GuideStrategy` based off of the message content

# Notes

*Important dependencies:*

- Provides `GuideStrategyResolver`: [guidepoint](../guidepoint/README.md)
- Provides `GuideStrategy` and `RequestEndpoint`: [core](../core/README.md)
