# DLMM Vault

This vault program allows the auto-compounding and auto-rebalancing of Meteora DLMM positions.

If only auto-compounding is enabled, the user is responsible for monitoring their position pricing, but the operator will claim and compound fees into the active position at least once an hour. The compounding strategy determines how the claimed fees are applied into the position. If the conservative strategy is used (which it is by default), then then all tokens on the wider side of liquidity from the active bin will be consumed, and appropriately ratiod against the narrower side. This is best illustrated with an example:

- The position in vault was opened with a width of 20, and the active bin ID on opening was 0.
- Upon claiming, the active bin ID is 5. All of token Y fees claimed will be distributed between bin ID -10 and 5. Half of the token X fees claimed will be distributed between bin ID 5 - 10. This approach looks to minimise impermament loss when compounding fees during volatile price action.

The aggressive compounding strategy distributes all claimed fees in the available bins, so to the above example, 100% of token X fees claimed would be distributed between bin ID 5 - 10.

Auto-rebalancing is facilitated through jupiter swaps against the vault ATAs for token X and token Y.

## Known Limitations

### Transfer Hooks are Unsupported

TODO

### Rewards Pools are Untested

TODO

## Misc / Dev Helpers

### Proc Macro2 IDL issue

Workaround: https://stackoverflow.com/questions/79582055/why-is-the-method-source-file-not-found-for-proc-macro2span

TLDR; `anchor build --no-idl` to build program, then use nightly toolchain to build IDLs when required

### Sync program ids
`anchor keys sync`