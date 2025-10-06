# DLMM Vault

This vault program allows the auto-compounding, auto-rebalancing and auto-harvesting of Meteora DLMM positions.

## Auto-Compounding

If only auto-compounding is enabled, the user is responsible for monitoring their position pricing, but the operator will claim and compound fees into the active position when economically viable to do so. The compounding strategy determines how the claimed fees are applied into the position. If the conservative strategy is used (which it is by default), then then all tokens on the wider side of liquidity from the active bin will be consumed, and appropriately ratiod against the narrower side. This is best illustrated with an example:

- The position in vault was opened with a width of 20, and the active bin ID on opening was 0.
- Upon claiming, the active bin ID is 5. All of token Y fees claimed will be distributed between bin ID -10 and 5. Half of the token X fees claimed will be distributed between bin ID 5 - 10. This approach looks to minimise impermament loss when compounding fees during volatile price action.

The aggressive compounding strategy distributes all claimed fees in the available bins, so to the above example, 100% of token X fees claimed would be distributed between bin ID 5 - 10.

## Auto-Rebalancing

Auto-rebalancing is facilitated through jupiter swaps against the vault ATAs for token X and token Y. Assuming a target position ratio of 50% token X, 50% token Y, if a position is closed and the ratio is currently 100% token X, 0% token Y, the operator will swap 50% of token X into token Y and re-open the position.

Auto-rebalancing must be enabled on the vault for the operator to perform this action.

## Auto-harvesting

The vault provides auto-harvesting behaviour, and optionally allows the user to specify a different mint to harvest fees into. For example, if LPing the FARM/SOL pool, the user may wish to harvest fees into USDC. To enable this, users must set their harvest value to >0. 10_000bps represents harvesting all fees, 0bps represents harvesting none. The alterate harvest mint is optional. It is possible to simply harvest fees into the vault and not have that portion of token re-added back into the liquidity pools.

## Fees

Ktulu charge the following operator fees:
 - 50bps of the fee collected on token Y when claiming fees, which happens when auto-compounding, auto-rebalancing and auto-harvesting.
 - 50bps when harvesting fees to a target mint
 - 5bps when executing rebalance operations.

In practice this means:
 - ~0.25% of fees earned by your positions are paid to Ktulu
 - ~0.025% of the position value is paid to Ktulu when rebalancing positions
 - An additional ~0.5% of fees earned are paid to Ktulu if auto-harvesting to an alternate mint is enabled

## Known Limitations

### Transfer Hooks are Unsupported

`Remaining accounts` are explicitly not forwards to the CPI token transfers. This must be added to support transfer hooks. Attempting to use transfer hook dependant Token2022 tokens will fail at the `deposit` step, as the program will fail to move funds into the vault.

### Rewards Pools are Untested

Rewards are rare for DLMM pools. The CPI call has been written but is not yet tested.
