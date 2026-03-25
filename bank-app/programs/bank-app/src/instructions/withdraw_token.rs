//Your code here
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use anchor_lang::prelude::*;
use crate::{
    constant::{BANK_INFO_SEED, BANK_VAULT_SEED, USER_RESERVE_SEED},
    error::BankAppError,
    state::{BankInfo, UserReserve},
};

#[derive(Accounts)]
pub struct WithdrawToken<'info>{
    #[account(
        mut,
        seeds = [BANK_INFO_SEED],
        bump
    )]
    pub bank_info: Box<Account<'info, BankInfo>>,
    
    #[account(
        mut,
        seeds = [
            USER_RESERVE_SEED, 
            user.key().as_ref(), 
            mint.key().as_ref()
        ],
        bump,
    )]
    pub user_reserve: Box<Account<'info, UserReserve>>,

    #[account(mut)]
    pub user: Signer<'info>,
    
    pub system_program: Program<'info,System>,
    pub mint: Account<'info,Mint>,
    
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = mint,          
        associated_token::authority = bank_info, 
    )]
    pub bank_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> WithdrawToken<'info>{
    pub fn process(ctx: Context<WithdrawToken>, withdraw_amount: u64) -> Result<()> {
        if ctx.accounts.bank_info.is_paused{
            return Err(BankAppError::BankAppPaused.into());
        }
        if ctx.accounts.user_reserve.deposited_amount < withdraw_amount {
            return Err(BankAppError::InsufficientBalance.into());
        }
        
        ctx.accounts.user_reserve.deposited_amount -= withdraw_amount;
        
        let pda_seeds: &[&[&[u8]]] = &[&[BANK_INFO_SEED, &[ctx.accounts.bank_info.bump]]];
        
        crate::transfer_helper::token_transfer_from_pda(
            ctx.accounts.bank_token_account.to_account_info(),
            ctx.accounts.bank_info.to_account_info(),
            ctx.accounts.user_token_account.to_account_info(),
            &ctx.accounts.token_program,
            pda_seeds,
            withdraw_amount,
        )?;
        Ok(())
    }
}