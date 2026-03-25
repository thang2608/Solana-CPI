use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use crate::state::BankInfo;
use crate::constant::{BANK_INFO_SEED, BANK_VAULT_SEED};
use staking_app::cpi::accounts::StakeToken; 
use staking_app::program::StakingApp;
use staking_app::GlobalVault; 
use anchor_spl::associated_token::AssociatedToken;

#[derive(Accounts)]
pub struct InvestToken<'info>{
    #[account(
        seeds = [BANK_INFO_SEED],
        bump,
    )]
    pub bank_info: Box<Account<'info,BankInfo>>,

    /// CHECK: Safe
    #[account(
        mut,
        seeds = [BANK_VAULT_SEED],
        bump
    )]
    pub bank_vault: UncheckedAccount<'info>,

    #[account(mut, address = bank_info.authority)]
    pub authority: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint, 
        associated_token::authority = bank_vault
    )]
    pub bank_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub staking_global_vault: Account<'info, GlobalVault>, 

    /// CHECK: Safe
    pub staking_vault_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub staking_token_vault: Account<'info, TokenAccount>,

    /// CHECK: Safe
    #[account(mut)]
    pub staking_user_info: UncheckedAccount<'info>,

    pub staking_program: Program<'info, StakingApp>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> InvestToken<'info>{
    pub fn process(ctx: Context<InvestToken>, amount: u64, is_stake: bool) -> Result<()> {
        if ctx.accounts.bank_info.is_paused{
            return Err(crate::error::BankAppError::BankAppPaused.into());
        }
        
        let bump = ctx.bumps.bank_vault;
        let bank_vault_seeds = &[BANK_VAULT_SEED, &[bump]];
        let signer = &[&bank_vault_seeds[..]];
        
        let cpi_accounts = StakeToken {
            user: ctx.accounts.bank_vault.to_account_info(),
            payer: ctx.accounts.authority.to_account_info(),
            global_vault: ctx.accounts.staking_global_vault.to_account_info(), 
            mint: ctx.accounts.mint.to_account_info(),
            user_ata: ctx.accounts.bank_ata.to_account_info(),
            vault_authority: ctx.accounts.staking_vault_authority.to_account_info(),
            token_vault: ctx.accounts.staking_token_vault.to_account_info(),
            user_token_info: ctx.accounts.staking_user_info.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.staking_program.to_account_info(), 
            cpi_accounts, 
            signer
        );
        
        staking_app::cpi::stake_token(cpi_ctx, amount, is_stake)?;
        Ok(())
    }
}