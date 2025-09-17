# SolanaDeads Fee Router (Token-2022)

This Anchor program routes Token-2022 transfer fees from a single mint into three sinks with fixed splits:

- 65.00% to Stakers
- 17.50% to Treasury
- 17.50% to LP Pool

It supports optional gross-up using the Token-2022 transfer fee extension so recipients net the intended amounts after fees.

## Program ID

Update `declare_id!("...")` in `programs/solanadeads_fee_router/src/lib.rs` to your deployed program ID.

## Mint and Sink Accounts

Constants in `src/lib.rs` define the supported Token-2022 mint and sink accounts:

- `DEADS_MINT`
- `TREASURY_WALLET`
- `LP_POOL_WALLET`
- `STAKERS_WALLET`

All sink accounts must be Token-2022 accounts for the same mint.

## PDA Seeds

Router PDA seeds: `[b"solanadeads", b"fee-router-v1", mint]`.

The router vault is the ATA for `(mint, router_pda)` with Token-2022 program.

## Instructions

### initialize_router

Creates the Router PDA for the given mint.

Accounts:
- [writable, pda] `router` (seeds: `[SEED_NAMESPACE, SEED_ROUTER, mint]`)
- [signer] `authority`
- `system_program`
- `mint` (InterfaceAccount<Mint>, must equal `DEADS_MINT`)

Client (TypeScript) example:
```ts
await program.methods
  .initializeRouter()
  .accounts({
    router,
    authority: wallet.publicKey,
    systemProgram: SystemProgram.programId,
    mint: deadsMint,
  })
  .rpc();
```

### distribute_fees(amount: u64, decimals: u8)

Distributes directly from the router vault per the fixed splits. Applies gross-up if the mint has an active transfer-fee config.

Accounts:
- [writable, pda] `router`
- [writable] `router_vault` (Token-2022 ATA for `(mint, router)`)
- `mint` (Token-2022 Mint)
- [writable] `stakers_wallet` (Token-2022 Account)
- [writable] `treasury_wallet` (Token-2022 Account)
- [writable] `lp_pool_wallet` (Token-2022 Account)
- `token_program` (Token-2022 ID)
- `associated_token_program`

Notes:
- The program now reads `decimals` from the mint internally; you may pass any value to the `decimals` arg and it will be ignored.

### harvest_and_distribute()

Harvests withheld fees from provided Token-2022 token accounts to the mint, withdraws withheld fees from the mint to the router vault, and then distributes the updated router vault balance per splits.

To avoid Rust lifetime issues on-chain, this instruction expects the caller to supply an ordered `remaining_accounts` slice:

1. `mint` (Token-2022 Mint)
2. `router` (Router PDA; authority for harvesting/withdrawing)
3. `token_program` (Token-2022 program ID)
4. `router_vault` (Token-2022 ATA for `(mint, router)`)
5. ... any number of fee-bearing Token-2022 Accounts (all for the same `mint`)

The program validates that entries [0..4) match the declared accounts and uses only `remaining_accounts` when performing the low-level SPL Token-2022 invokes.

Client (TypeScript) sketch:
```ts
const remainingAccounts = [
  { pubkey: mint, isWritable: false, isSigner: false },
  { pubkey: router, isWritable: false, isSigner: false },
  { pubkey: TOKEN_2022_PROGRAM_ID, isWritable: false, isSigner: false },
  { pubkey: routerVault, isWritable: true, isSigner: false },
  // fee-bearing accounts (writable)
  ...feeAccounts.map(a => ({ pubkey: a, isWritable: true, isSigner: false })),
];

await program.methods
  .harvestAndDistribute()
  .accounts({
    router,
    routerVault,
    mint,
    stakersWallet,
    treasuryWallet,
    lpPoolWallet,
    tokenProgram: TOKEN_2022_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  })
  .remainingAccounts(remainingAccounts)
  .rpc();
```

## Build and Test

- Build Rust crates:
```
cargo build
```

- Build Anchor workspace (generates the IDL, TS types):
```
anchor build
```

- Tests
  - The provided test file is a placeholder showing how to construct calls. It is marked as skipped because a full end-to-end test would need to mint a Token-2022 test mint and create several accounts.

## Notes on Token-2022

- Transfer-fee parameters are read via `StateWithExtensions::<Mint>` and `TransferFeeConfig`.
- Instruction builders come from `spl_token_2022::extension::transfer_fee::instruction`:
  - `harvest_withheld_tokens_to_mint(program, mint, signers)`
  - `withdraw_withheld_tokens_from_mint(program, mint, destination, authority, signers)`
