# View Facade

The `view-facade` contract is a read-only registry that lets dashboards,
wallets, and indexers discover Grainlify contract addresses through one
query surface.

## Query API

- `list_contracts()`: returns all registered contracts in registration order.
- `contract_count()`: returns the current registry length.
- `get_contract(address)`: returns the first matching registration for a
  known address, or `None` when the address is absent.

## Query Examples

Typical dashboard flow:

1. Call `contract_count()` to estimate the size of the registry snapshot.
2. Call `list_contracts()` to render the current list view.
3. Call `get_contract(address)` when refreshing details for one selected
   contract card.

## Limits

`get_contract` performs an `O(n)` scan over the in-contract registry, and
`list_contracts` returns the full registry in one response. This is
acceptable for the intended small, curated set of Grainlify contracts, but
the facade should not be treated as a general-purpose index for unbounded
datasets.

## Security Notes

- The facade is read-only and does not custody funds.
- Query methods do not perform cross-contract calls.
- Registry data is useful discovery metadata, but downstream consumers should
  not use it as an authorization primitive.
