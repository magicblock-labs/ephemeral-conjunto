
# Summary

Implements logic to figure out the desired final endpoint for a given request's content

# Details

*Important symbols:*

- `GuideStrategyResolver`
  - depends on an `AccountProvider`
  - depends on a `SignatureStatusProvider`
  - Allow resolving a `GuideStrategy` into a `RequestEndpoint`
  - Allow resolving a signature into a `RequestEndpoint`
  - Allow resolving an address into a `RequestEndpoint`

# Notes

*Important dependencies:*

- Provides `AccountProvider`, `SignatureStatusProvider` and `RequestEndpoint`: [core](../core/README.md) 

