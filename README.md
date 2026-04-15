# Ktulu DLMM Vault Program

The Ktulu DLMM vault program is an open source example for building programs that have signficant CPI integrations. Specifically the dlmm-vault program integrates with both Meteora DLMM and Jupiter Aggregator v6.

## Note:

The mainnet program has been closed. Thanks for those who helped test the protocol. There were two vaults that remained open. The only token in these vaults was Sol. I have refunded the Sol in the vaults (as I of course could not forcibly close the position) plus the rent value.

```
These were the two open vaults, with the signatures of the refunds below:

Vault #1
Address: CEenGrA5xe271hZAxjrfYfiWSh87yV3wmzV7p5YiWhWc
{
  "owner": "GM5qrgMUHAAWDF7cSkFQ6DJDTqGugaQvDpaFxGLtTXxZ",
  "inPosition": true,
  "operator": "AP1t6iZxcomviEriw22wGygJWnhaPwhVp8AoVZ9H7Wtz",
  "dlmmPoolId": "BGm1tav58oGcsQJehL9WXBFXF7D27vZsKefj4xJKD5Y",
  "positionId": "DrBtEK28Z78bSxaB4JnwmKga8tLGmLHEyR4n8w7fjwB",
  "tokenXMint": "So11111111111111111111111111111111111111112",
  "tokenYMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
  "autoCompound": false,
  "autoRebalance": false,
  "useHarvestMint": true,
  "harvestBps": 10000,
  "harvestMint": "So11111111111111111111111111111111111111112",
  "feeCompoundingStrategy": {
    "conservative": {}
  },
  "binWidth": 49,
  "volatilityStrategy": {
    "spot": {}
  },
  "virtualTokenXHarvest": "0",
  "virtualTokenYHarvest": "0"
}
================================================================================
Vault #2
Address: 8rBcopxDWbbScy238JVKcFp3FrrjjQg1eErHDxypf1vr
{
  "owner": "6uBbMw3jPsA4PqSst6SqSZSRe1hZbFoiVFZagLaJ21at",
  "inPosition": true,
  "operator": "AP1t6iZxcomviEriw22wGygJWnhaPwhVp8AoVZ9H7Wtz",
  "dlmmPoolId": "BGm1tav58oGcsQJehL9WXBFXF7D27vZsKefj4xJKD5Y",
  "positionId": "3tKY9Rwm4dZn3AJ6ynwHyNrYVfrancThvWbbV7hD39er",
  "tokenXMint": "So11111111111111111111111111111111111111112",
  "tokenYMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
  "autoCompound": true,
  "autoRebalance": true,
  "useHarvestMint": true,
  "harvestBps": 10000,
  "harvestMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
  "feeCompoundingStrategy": {
    "conservative": {}
  },
  "binWidth": 41,
  "volatilityStrategy": {
    "spot": {}
  },
  "virtualTokenXHarvest": "25454701",
  "virtualTokenYHarvest": "3777893"
}
================================================================================

Open vault position balances (from Meteora DLMM):

Open Vault #1
Vault: CEenGrA5xe271hZAxjrfYfiWSh87yV3wmzV7p5YiWhWc
Pool: BGm1tav58oGcsQJehL9WXBFXF7D27vZsKefj4xJKD5Y
Position: DrBtEK28Z78bSxaB4JnwmKga8tLGmLHEyR4n8w7fjwB
Token X mint: So11111111111111111111111111111111111111112
Token Y mint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
Position totalXAmount: 79713298 (ui: 0.079713298)
Position totalYAmount: 0 (ui: 0)
Position totalXAmountExcludeTransferFee: 79713298
Position totalYAmountExcludeTransferFee: 0
--------------------------------------------------------------------------------
Open Vault #2
Vault: 8rBcopxDWbbScy238JVKcFp3FrrjjQg1eErHDxypf1vr
Pool: BGm1tav58oGcsQJehL9WXBFXF7D27vZsKefj4xJKD5Y
Position: 3tKY9Rwm4dZn3AJ6ynwHyNrYVfrancThvWbbV7hD39er
Token X mint: So11111111111111111111111111111111111111112
Token Y mint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
Position totalXAmount: 26178921 (ui: 0.026178921)
Position totalYAmount: 0 (ui: 0)
Position totalXAmountExcludeTransferFee: 26178921
Position totalYAmountExcludeTransferFee: 0
--------------------------------------------------------------------------------

➜  programs git:(main) ✗ solana transfer GM5qrgMUHAAWDF7cSkFQ6DJDTqGugaQvDpaFxGLtTXxZ 0.14
Signature: 53MYLLCyMi1qd6ArfMDFryobARdzYr4nDwEnPAYPPvZ6bsA73HvmX8Y2ZU9RtJxy4wJDXxBohZnHLhvvkEx8zXnh
➜  programs git:(main) ✗ solana transfer 6uBbMw3jPsA4PqSst6SqSZSRe1hZbFoiVFZagLaJ21at 0.086
Signature: 4UKyDicpAuLxgEg4otsrFr5ZhtEzfjApxFdgPxTkEJvGdKFQYhcAQaEocAoQtCsZeCDFffhMmGyiNfK6aLUjct8F
```

## About the Program

This vault program allows the auto-compounding, auto-rebalancing and auto-harvesting of Meteora DLMM positions.

### Auto-Compounding

If only auto-compounding is enabled, the user is responsible for monitoring their position pricing, but the operator will claim and compound fees into the active position when economically viable to do so. The compounding strategy determines how the claimed fees are applied into the position. If the conservative strategy is used (which it is by default), then fee compounding only occurs if the active bin is within the middle range of bins for the position. To clarify with an example:

- The position in vault was opened with a width of 20, and the active bin ID on opening was 0. The middle 50% range for the position is considered bins -5 - 5.
- Enough fees have accumulated to trigger an auto-compound, hower the active bin is currently -7, so no action is taken.
- Upon re-evaluation after X time, the active bin is now -3, which will trigger an autocompounding of fees back into the position.

The aggressive compounding strategy distributes all claimed fees into the position regardless of active bin.

### Auto-Rebalancing

Auto-rebalancing is facilitated through jupiter swaps against the vault ATAs for token X and token Y. Assuming a target position ratio of 50% token X, 50% token Y, if a position is closed and the ratio is currently 100% token X, 0% token Y, the operator will swap 50% of token X into token Y and re-open the position.

Auto-rebalancing must be enabled on the vault for the operator to perform this action.

### Auto-harvesting

The vault provides auto-harvesting behaviour, and optionally allows the user to specify a different mint to harvest fees into. For example, if LPing the FARM/SOL pool, the user may wish to harvest fees into USDC. To enable this, users must set their harvest value to >0. 10_000bps represents harvesting all fees, 0bps represents harvesting none. The alterate harvest mint is optional. It is possible to simply harvest fees into the vault and not have that portion of token re-added back into the liquidity pools.

### Fees

Ktulu charge the following operator fees:
 - 50bps of the fee collected on token Y when claiming fees, which happens when auto-compounding, auto-rebalancing and auto-harvesting.
 - 50bps when harvesting fees to a target mint
 - 5bps when executing rebalance operations.

In practice this means:
 - ~0.25% of fees earned by your positions are paid to Ktulu
 - ~0.025% of the position value is paid to Ktulu when rebalancing positions
 - An additional ~0.5% of fees earned are paid to Ktulu if auto-harvesting to an alternate mint is enabled

### Known Limitations

#### Transfer Hooks are Unsupported

`Remaining accounts` are explicitly not forwards to the CPI token transfers. This must be added to support transfer hooks. Attempting to use transfer hook dependant Token2022 tokens will fail at the `deposit` step, as the program will fail to move funds into the vault.

#### Rewards Pools are Untested

Rewards are rare for DLMM pools. The CPI call has been written but is not yet tested.
