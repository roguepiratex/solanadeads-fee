// programs/solanadeads_fee_router/src/lib.rs

//! ============================================================================
//! Solana Deads — Fee Router (Token-2022) with TransferFee Harvesting
//! ----------------------------------------------------------------------------
//! Adds `harvest_and_distribute()` which:
//!   1) Harvests withheld fees to the mint (from provided token accounts)
//!   2) Withdraws withheld fees from mint → router_vault (PDA ATA)
//!   3) Distributes router_vault per 65 / 17.5 / 17.5 (optionally grossed-up)
//!
//! Mint requirements (Token-2022):
//!   • TransferFeeConfig present on DEADS mint
//!   • harvest_withheld_authority = router PDA
//!   • withdraw_withheld_authority = router PDA
//! ============================================================================

use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::AccountMeta;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::associated_token::AssociatedToken;
use anchor_lang::prelude::InterfaceAccount;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};
use spl_token_2022::extension::transfer_fee::instruction as token2022_ix;
use spl_token_2022::extension::transfer_fee::TransferFeeConfig;
use spl_token_2022::extension::{BaseStateWithExtensions, StateWithExtensions};
use spl_token_2022::state::Mint as SplMint;
use std::str::FromStr;

#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

declare_id!("DEADS3ucNHjN8iz3Cw65joYxgVdguNsjytHRqCs7QvzA");

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "Solana Deads Harvester",
    project_url: "https://www.solanadeads.com",
    contacts: "email:admin@solanadeads.com,discord:solanadeads",
    policy: "https://www.solanadeads.com/security-policy",
    source_code: "https://github.com/roguepiratex/solanadeads-fee",
    preferred_languages: "en",
    auditors: "None"
}

// ------------------------------ Constants ------------------------------------

pub const SEED_NAMESPACE: &[u8] = b"solanadeads";
pub const SEED_ROUTER: &[u8] = b"fee-router-v1";  // Use v1 (already has withdraw authority)

// Hard-set **owner wallets** (cluster-agnostic). Program derives ATAs at runtime.
pub const TREASURY_OWNER: &str = "26xcb2Ygdj47BSsXTgQf4QHQw38jxMaKbENHyzwkaQA8";
pub const LP_OWNER: &str = "4zrLoUzDrTSohZ4ay6uuQM5fAPbyPSMi31hTRCaaQx7y";

// The Undertaker Rewards Vault - This is the rewards-vault ATA (Token Account)
// Vault Authority PDA: 6vLUBJaJoN9YK4jKsTwhMReWyNm9DoWw9Eee1hRZWgDV
// Vault ATA: 2SHAd8fzBFYnDvK8DBHYQkcjiVtxh2L7ondTQ1ECztFv
// This vault receives 65% of all harvested fees, which then get distributed to active pools!
// STAKERS_OWNER is the actual token account (ATA), not the authority
pub const STAKERS_OWNER: &str = "2SHAd8fzBFYnDvK8DBHYQkcjiVtxh2L7ondTQ1ECztFv";

// Token-2022 DEADS mint (mainnet & devnet)
pub const DEADS_MINT: &str = "DEADsWJZaonaiZPFkrqEEBGf43mzA5uHeHpwgy9dW666";

// Rewards Program ID (deployed on devnet/mainnet)
pub const REWARDS_PROGRAM_ID: &str = "DEADZS7SrZMW5aGgXzgUis59iaQfjgdmnXMQuJJo7uAu";

// Splits (basis points)
pub const STAKERS_BP: u16 = 6500;  // 65.00%
pub const TREASURY_BP: u16 = 1750; // 17.50%

// Optional dust guard
pub const MIN_DISTRIBUTE: u64 = 10;

// ------------------------------ Events ---------------------------------------

#[event]
pub struct FeeDistribution {
    pub stakers_amount: u64,
    pub treasury_amount: u64,
    pub lp_amount: u64,
    pub total: u64,
}

#[event]
pub struct HarvestRun {
    pub sources: u32,
    pub vault_before: u64,
    pub distributed: u64,
    pub vault_after: u64,
}

// ------------------------------ Errors ---------------------------------------

#[error_code]
pub enum RouterError {
    #[msg("Input amount must be greater than or equal to the minimum threshold")]
    ZeroAmount,
    #[msg("Math overflow while computing splits")]
    MathOverflow,
    #[msg("Router vault has insufficient balance for requested distribution")]
    InsufficientVaultBalance,
    #[msg("Provided decimals do not match the mint's decimals")]
    DecimalsMismatch,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Sink account mint does not match the DEADS mint")]
    InvalidMintForSink,
    #[msg("Sink account is for the wrong token program")]
    WrongTokenProgramForSink,
}

// ------------------------------ State ----------------------------------------

#[account]
pub struct Router {
    pub bump: u8,
    pub authority: Pubkey,
}
impl Router {
    pub const LEN: usize = 1 + 32;
}

// ------------------------------ Accounts -------------------------------------

#[derive(Accounts)]
pub struct InitializeRouter<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Router::LEN,
        seeds = [SEED_NAMESPACE, SEED_ROUTER, mint.key().as_ref()],
        bump
    )]
    pub router: Account<'info, Router>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = Pubkey::from_str(DEADS_MINT).unwrap())]
    pub mint: InterfaceAccount<'info, Mint>,
}

#[derive(Accounts)]
pub struct DistributeFees<'info> {
    #[account(
        mut,
        seeds = [SEED_NAMESPACE, SEED_ROUTER, mint.key().as_ref()],
        bump
    )]
    pub router: Account<'info, Router>,

    #[account(address = Pubkey::from_str(DEADS_MINT).unwrap())]
    pub mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,


    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = router,
        associated_token::token_program = token_program
    )]
    pub router_vault: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: fixed owner; ATA is derived below
    #[account(address = Pubkey::from_str(TREASURY_OWNER).unwrap())]
    pub treasury_owner: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = treasury_owner,
        associated_token::token_program = token_program
    )]
    pub treasury_wallet: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: fixed owner; ATA is derived below
    #[account(address = Pubkey::from_str(LP_OWNER).unwrap())]
    pub lp_owner: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = lp_owner,
        associated_token::token_program = token_program
    )]
    pub lp_pool_wallet: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: fixed owner; ATA is derived below
    #[account(address = Pubkey::from_str(STAKERS_OWNER).unwrap())]
    pub stakers_owner: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = stakers_owner,
        associated_token::token_program = token_program
    )]
    pub stakers_wallet: InterfaceAccount<'info, TokenAccount>,
}

#[derive(Accounts)]
pub struct HarvestAndDistribute<'info> {
    #[account(
        mut,
        seeds = [SEED_NAMESPACE, SEED_ROUTER, mint.key().as_ref()],
        bump
    )]
    pub router: Account<'info, Router>,

    #[account(
        mut,
        address = Pubkey::from_str(DEADS_MINT).unwrap()
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = router,
        associated_token::token_program = token_program
    )]
    pub router_vault: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: fixed owner; ATA is derived below
    #[account(address = Pubkey::from_str(TREASURY_OWNER).unwrap())]
    pub treasury_owner: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = treasury_owner,
        associated_token::token_program = token_program
    )]
    pub treasury_wallet: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: fixed owner; ATA is derived below
    #[account(address = Pubkey::from_str(LP_OWNER).unwrap())]
    pub lp_owner: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = lp_owner,
        associated_token::token_program = token_program
    )]
    pub lp_pool_wallet: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: Rewards vault token account
    #[account(
        mut,
        address = Pubkey::from_str(STAKERS_OWNER).unwrap()
    )]
    pub stakers_wallet: InterfaceAccount<'info, TokenAccount>,
    
    // Rewards program accounts for CPI
    /// CHECK: Vault authority PDA
    pub vault_authority_pda: UncheckedAccount<'info>,
    /// CHECK: Rewards program ID
    pub rewards_program: UncheckedAccount<'info>,
    /// CHECK: Rewards program config PDA
    #[account(mut)]
    pub rewards_config: UncheckedAccount<'info>,
    /// CHECK: Rewards program pool registry PDA
    pub pool_registry: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

// ------------------------------ Program --------------------------------------

#[program]
pub mod solanadeads_fee_router {
    use super::*;

    pub fn initialize_router(ctx: Context<InitializeRouter>) -> Result<()> {
        let router = &mut ctx.accounts.router;
        router.bump = ctx.bumps.router;
        router.authority = ctx.accounts.authority.key();
        Ok(())
    }

    /// Distribute a specific amount from the router vault.
    pub fn distribute_fees(ctx: Context<DistributeFees>, amount: u64, decimals: u8) -> Result<()> {
        // Token-2022 only
        require_keys_eq!(
            ctx.accounts.token_program.key(),
            spl_token_2022::ID,
            ErrorCode::WrongTokenProgramForSink
        );
        require!(amount >= MIN_DISTRIBUTE, RouterError::ZeroAmount);

        // Ensure the vault can cover the requested amount
        require!(
            ctx.accounts.router_vault.amount >= amount,
            RouterError::InsufficientVaultBalance
        );

        // Prefer the mint's decimals (ignore or assert the arg)
        let decimals_from_mint = ctx.accounts.mint.decimals;
        let _ = decimals; // or: require!(decimals == decimals_from_mint, RouterError::DecimalsMismatch);

        let router = &ctx.accounts.router;
        let mint_key = ctx.accounts.mint.key();
        let seeds = [
            SEED_NAMESPACE.as_ref(),
            SEED_ROUTER.as_ref(),
            mint_key.as_ref(),
            &[router.bump],
        ];
        let signer = &[&seeds[..]];

        let fee_params = get_fee_params(&ctx.accounts.mint.to_account_info())?;

        distribute_now(
            &ctx.accounts.token_program,
            &ctx.accounts.router,
            &ctx.accounts.mint,
            &ctx.accounts.router_vault,
            &ctx.accounts.stakers_wallet,
            &ctx.accounts.treasury_wallet,
            &ctx.accounts.lp_pool_wallet,
            signer,
            amount,
            decimals_from_mint,
            fee_params,
        )
    }

    /// Harvest withheld fees, withdraw to vault, then distribute.
    /// `remaining_accounts` should be the list of **fee-bearing token accounts** to harvest from.
    pub fn harvest_and_distribute<'a, 'b, 'c, 'info>(ctx: Context<'a, 'b, 'c, 'info, HarvestAndDistribute<'info>>) -> Result<()> {
        // Token-2022 only
        require_keys_eq!(
            ctx.accounts.token_program.key(),
            spl_token_2022::ID,
            ErrorCode::WrongTokenProgramForSink
        );

        let router = &ctx.accounts.router;
        let mint_key = ctx.accounts.mint.key();
        let seeds = [
            SEED_NAMESPACE.as_ref(),
            SEED_ROUTER.as_ref(),
            mint_key.as_ref(),
            &[router.bump],
        ];
        let signer = &[&seeds[..]];

        if let Some((bps, max_fee)) = get_fee_params(&ctx.accounts.mint.to_account_info())? {
            msg!("Transfer-Fee (epoch): {} bps, max {}", bps, max_fee);
        } else {
            msg!("Transfer-Fee config not found or no epoch fee set on mint");
        }

        // Sanity-check remaining fee-bearing accounts: correct owner & mint
        // remaining_accounts are just the fee-bearing token accounts (no mint)
        // Allow empty harvests (no-op if no fees to collect)
        if ctx.remaining_accounts.is_empty() {
            msg!("No fee-bearing accounts provided - skipping harvest");
            return Ok(());
        }
        
        for acc in ctx.remaining_accounts.iter() {
            require_keys_eq!(
                *acc.owner,
                ctx.accounts.token_program.key(),
                ErrorCode::WrongTokenProgramForSink
            );
            let mut data: &[u8] = &acc.try_borrow_data()?;
            let ta = anchor_spl::token_interface::TokenAccount::try_deserialize(&mut data)?;
            require_keys_eq!(ta.mint, ctx.accounts.mint.key(), ErrorCode::InvalidMintForSink);
        }

        // Snapshot vault before
        let vault_before = ctx.accounts.router_vault.amount;

        // 1) Harvest → mint from provided fee-bearing token accounts
        let mut ix_harvest = token2022_ix::harvest_withheld_tokens_to_mint(
            &ctx.accounts.token_program.key(),
            &ctx.accounts.mint.key(),
            &[],
        )
        .unwrap();
        // Add fee accounts to the harvest instruction
        for acc in ctx.remaining_accounts.iter() {
            ix_harvest.accounts.push(AccountMeta::new(acc.key(), false));
        }
        // Create account infos: [mint, fee_accounts...]
        let mut harvest_account_infos = vec![ctx.accounts.mint.to_account_info()];
        harvest_account_infos.extend(ctx.remaining_accounts.iter().map(|acc| acc.clone()));
        invoke_signed(&ix_harvest, &harvest_account_infos, signer)?;

        // 2) Withdraw withheld → router_vault (authority = router PDA)
        let ix_withdraw = token2022_ix::withdraw_withheld_tokens_from_mint(
            &ctx.accounts.token_program.key(),
            &ctx.accounts.mint.key(),
            &ctx.accounts.router_vault.key(),
            &ctx.accounts.router.key(),
            &[],
        )
        .unwrap();
        // AccountInfos must match ix metas: [mint, destination (vault), authority (router)]
        let infos_withdraw = vec![
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.router_vault.to_account_info(),
            ctx.accounts.router.to_account_info(),
        ];
        invoke_signed(&ix_withdraw, &infos_withdraw[..], signer)?;

        // 3) Re-read vault AFTER withdraw, then distribute that fresh balance
        ctx.accounts.router_vault.reload()?;
        let amount = ctx.accounts.router_vault.amount;
        msg!("Vault balance after withdraw: {} lamports", amount);
        
        if amount < MIN_DISTRIBUTE {
            msg!("Vault balance {} < MIN_DISTRIBUTE {}, skipping distribution", amount, MIN_DISTRIBUTE);
            return Ok(());
        }

        let decimals_from_mint = ctx.accounts.mint.decimals;
        let fee_params = get_fee_params(&ctx.accounts.mint.to_account_info())?;

        distribute_now(
            &ctx.accounts.token_program,
            &ctx.accounts.router,
            &ctx.accounts.mint,
            &ctx.accounts.router_vault,
            &ctx.accounts.stakers_wallet,
            &ctx.accounts.treasury_wallet,
            &ctx.accounts.lp_pool_wallet,
            signer,
            amount,
            decimals_from_mint,
            fee_params,
        )?;

        // Re-read for `vault_after`
        ctx.accounts.router_vault.reload()?;
        let vault_after = ctx.accounts.router_vault.amount;

        // CPI: Call rewards program's sync_vault_and_distribute
        // This triggers automatic distribution of deposited rewards to active pools
        let (stakers_amount, _, _) = compute_splits(amount)?;
        if stakers_amount > 0 {
            msg!("Calling rewards program sync_vault_and_distribute via CPI");
            
            // Discriminator for sync_vault_and_distribute
            let discriminator: [u8; 8] = [8, 138, 201, 54, 235, 135, 144, 204];
            
            // Empty pool_ids - rewards program will use its own active_pool_ids from config
            let pool_ids: Vec<u8> = vec![];
            
            // Encode instruction data: discriminator + Vec<u8> (length + elements)
            let mut data = discriminator.to_vec();
            data.extend_from_slice(&(pool_ids.len() as u32).to_le_bytes());
            data.extend_from_slice(&pool_ids);
            
            // Derive rewards program PDAs
            let (rewards_config, _) = Pubkey::find_program_address(
                &[b"rewards-config", ctx.accounts.mint.key().as_ref()],
                &ctx.accounts.rewards_program.key()
            );
            
            let (pool_registry, _) = Pubkey::find_program_address(
                &[b"pool-registry-v2", ctx.accounts.mint.key().as_ref()],
                &ctx.accounts.rewards_program.key()
            );
            
            let (vault_authority_pda, _) = Pubkey::find_program_address(
                &[b"rewards-vault", ctx.accounts.mint.key().as_ref()],
                &ctx.accounts.rewards_program.key()
            );
            
            // Accounts: config, pool_registry, mint, vault_authority, vault, token_program, system_program
            let cpi_accounts = vec![
                AccountMeta::new(rewards_config, false),
                AccountMeta::new_readonly(pool_registry, false),
                AccountMeta::new_readonly(ctx.accounts.mint.key(), false),
                AccountMeta::new_readonly(vault_authority_pda, false),
                AccountMeta::new(ctx.accounts.stakers_wallet.key(), false),
                AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
                AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
            ];
            
            let cpi_instruction = anchor_lang::solana_program::instruction::Instruction {
                program_id: ctx.accounts.rewards_program.key(),
                accounts: cpi_accounts,
                data,
            };
            
            let cpi_account_infos = vec![
                ctx.accounts.rewards_config.to_account_info(),
                ctx.accounts.pool_registry.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.vault_authority_pda.to_account_info(),
                ctx.accounts.stakers_wallet.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ];
            
            anchor_lang::solana_program::program::invoke(
                &cpi_instruction,
                &cpi_account_infos,
            )?;
            
            msg!("✅ Rewards vault synced and distributed via CPI");
        }

        emit!(HarvestRun {
            sources: ctx.remaining_accounts.len() as u32,
            vault_before,
            distributed: amount,
            vault_after,
        });

        Ok(())
    }
}

// ------------------------------ Helpers --------------------------------------

// ------------------------------ Helpers --------------------------------------

fn compute_splits(amount: u64) -> Result<(u64, u64, u64)> {
    let amount_u128 = amount as u128;

    let stakers = amount_u128
        .checked_mul(STAKERS_BP as u128)
        .and_then(|v| v.checked_div(10_000))
        .ok_or(RouterError::MathOverflow)?;

    let treasury = amount_u128
        .checked_mul(TREASURY_BP as u128)
        .and_then(|v| v.checked_div(10_000))
        .ok_or(RouterError::MathOverflow)?;

    let lp = amount_u128
        .checked_sub(stakers)
        .and_then(|v| v.checked_sub(treasury))
        .ok_or(RouterError::MathOverflow)?;

    Ok((stakers as u64, treasury as u64, lp as u64))
}

fn ceil_div_u128(n: u128, d: u128) -> u128 {
    (n + d - 1) / d
}

/// Compute a gross amount so the **net** is `target_net`.
fn gross_up(target_net: u64, bps: u16, max_fee: u64) -> Result<u64> {
    if target_net == 0 {
        return Ok(0);
    }
    let rate_num: u128 = bps as u128;
    let denom: u128 = 10_000;

    let net_u: u128 = target_net as u128;
    let denom_minus = denom
        .checked_sub(rate_num)
        .ok_or(RouterError::MathOverflow)?;
    require!(denom_minus > 0, RouterError::MathOverflow);

    let gross_uncapped_u =
        ceil_div_u128(net_u.checked_mul(denom).ok_or(RouterError::MathOverflow)?, denom_minus);
    let fee_uncapped_u = gross_uncapped_u
        .checked_mul(rate_num)
        .ok_or(RouterError::MathOverflow)?
        / denom;

    if fee_uncapped_u <= (max_fee as u128) {
        let gross_u64 = u64::try_from(gross_uncapped_u).map_err(|_| RouterError::MathOverflow)?;
        return Ok(gross_u64);
    }

    let gross_capped_u = net_u
        .checked_add(max_fee as u128)
        .ok_or(RouterError::MathOverflow)?;
    let gross_capped = u64::try_from(gross_capped_u).map_err(|_| RouterError::MathOverflow)?;
    Ok(gross_capped)
}

fn maybe_gross_up_splits(
    stakers: u64,
    treasury: u64,
    lp: u64,
    fee_params: Option<(u16, u64)>,
) -> Result<(u64, u64, u64)> {
    if let Some((bps, max_fee)) = fee_params {
        // Skip gross-up if fee is >= 100% (mathematically impossible)
        if bps >= 10000 {
            return Ok((stakers, treasury, lp));
        }
        let s = gross_up(stakers, bps, max_fee)?;
        let t = gross_up(treasury, bps, max_fee)?;
        let l = gross_up(lp, bps, max_fee)?;
        Ok((s, t, l))
    } else {
        Ok((stakers, treasury, lp))
    }
}

fn distribute_now<'info>(
    token_program: &Interface<'info, TokenInterface>,
    router: &Account<'info, Router>,
    mint: &InterfaceAccount<'info, Mint>,
    router_vault: &InterfaceAccount<'info, TokenAccount>,
    stakers_wallet: &InterfaceAccount<'info, TokenAccount>,
    treasury_wallet: &InterfaceAccount<'info, TokenAccount>,
    lp_pool_wallet: &InterfaceAccount<'info, TokenAccount>,
    signer: &[&[&[u8]]],
    amount: u64,
    decimals: u8,
    fee_params: Option<(u16, u64)>,
) -> Result<()> {
    let (stakers_target, treasury_target, lp_target) = compute_splits(amount)?;
    let (mut s_amt, mut t_amt, mut l_amt) =
        maybe_gross_up_splits(stakers_target, treasury_target, lp_target, fee_params)?;

    // Safety: if gross-up exceeds available amount, fall back to raw targets
    let total_gross = s_amt
        .checked_add(t_amt)
        .ok_or(RouterError::MathOverflow)?
        .checked_add(l_amt)
        .ok_or(RouterError::MathOverflow)?;
    if total_gross > amount {
        s_amt = stakers_target;
        t_amt = treasury_target;
        l_amt = lp_target;
    }

    // Stakers
    transfer_checked(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            TransferChecked {
                from: router_vault.to_account_info(),
                mint: mint.to_account_info(),
                to: stakers_wallet.to_account_info(),
                authority: router.to_account_info(),
            },
            signer,
        ),
        s_amt,
        decimals,
    )?;

    // Treasury
    transfer_checked(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            TransferChecked {
                from: router_vault.to_account_info(),
                mint: mint.to_account_info(),
                to: treasury_wallet.to_account_info(),
                authority: router.to_account_info(),
            },
            signer,
        ),
        t_amt,
        decimals,
    )?;

    // LP
    transfer_checked(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            TransferChecked {
                from: router_vault.to_account_info(),
                mint: mint.to_account_info(),
                to: lp_pool_wallet.to_account_info(),
                authority: router.to_account_info(),
            },
            signer,
        ),
        l_amt,
        decimals,
    )?;

    emit!(FeeDistribution {
        stakers_amount: stakers_target, // report intended net targets
        treasury_amount: treasury_target,
        lp_amount: lp_target,
        total: amount,
    });

    Ok(())
}

/// Read Token-2022 transfer-fee parameters for the current epoch.
/// Returns (basis_points, maximum_fee) if present.
fn get_fee_params(mint_ai: &AccountInfo) -> Result<Option<(u16, u64)>> {
    let data = mint_ai.try_borrow_data()?;
    let Ok(state) = StateWithExtensions::<SplMint>::unpack(&data) else {
        return Ok(None);
    };
    let Ok(cfg) = state.get_extension::<TransferFeeConfig>() else {
        return Ok(None);
    };

    let epoch = Clock::get()?.epoch;
    let epoch_fee = cfg.get_epoch_fee(epoch);
    Ok(Some((
        epoch_fee.transfer_fee_basis_points.into(),
        epoch_fee.maximum_fee.into(),
    )))
}
