use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
declare_id!("6Zy6mwXfPtPMJ5PfB5q6w54Wpgckp2Z1rkuxHunbWWM6");

pub mod transfer_helper;

#[program]
pub mod staking_app {
    use transfer_helper::{sol_transfer_from_pda, sol_transfer_from_user};
    use super::*;

    pub fn stake(ctx: Context<Stake>, amount: u64, is_stake: bool) -> Result<()> {
        require!(amount > 0, MyError::InvalidAmount);
        let global_vault = &mut ctx.accounts.global_vault;
        let user_info = &mut ctx.accounts.user_info;

        if is_stake {
            let share_to_mint = if global_vault.total_shares == 0  {
                amount
            } else {
                (amount as u128)
                    .checked_mul(global_vault.total_shares as u128).unwrap()
                    .checked_div(global_vault.total_assets as u128).unwrap() as u64
            };
            sol_transfer_from_user(
                &ctx.accounts.user,
                ctx.accounts.staking_vault.to_account_info(),
                &ctx.accounts.system_program,
                amount,
            )?;
            user_info.shares = user_info.shares.checked_add(share_to_mint).ok_or(MyError::InvalidAmount)?;
            global_vault.total_assets = global_vault.total_assets.checked_add(amount).ok_or(MyError::MathOverflow)?;
            global_vault.total_shares = global_vault.total_shares.checked_add(share_to_mint).ok_or(MyError::MathOverflow)?;
        } else {
            require!(amount <= user_info.shares && global_vault.total_shares != 0, MyError::InvalidAmount);
            let sol_to_return = (amount as u128)
                .checked_mul(global_vault.total_assets as u128).unwrap()
                .checked_div(global_vault.total_shares as u128).unwrap() as u64;

            let bump = ctx.bumps.staking_vault;
            let seeds: &[&[u8]] = &[b"STAKING_VAULT", &[bump]];
            let pda_seeds = &[&seeds[..]];
            sol_transfer_from_pda(
                ctx.accounts.staking_vault.to_account_info(),
                ctx.accounts.user.to_account_info(),
                &ctx.accounts.system_program,
                pda_seeds,
                sol_to_return,
            )?;
            user_info.shares = user_info.shares.checked_sub(amount).ok_or(MyError::InvalidAmount)?;
            global_vault.total_assets = global_vault.total_assets.checked_sub(sol_to_return).ok_or(MyError::MathOverflow)?;
            global_vault.total_shares = global_vault.total_shares.checked_sub(amount).ok_or(MyError::MathOverflow)?;
            
        }
        Ok(())
    }
    pub fn stake_token(ctx: Context<StakeToken>, amount: u64, is_stake: bool) -> Result<()> {
        msg!("Hàm stake_token đã được gọi thông qua CPI!");
        Ok(())
    }
    pub fn add_rewards(ctx: Context<AddRewards>, amount: u64) -> Result<()> {
        require!(amount > 0, MyError::InvalidAmount);
        sol_transfer_from_user(
            &ctx.accounts.admin,
            ctx.accounts.staking_vault.to_account_info(),
            &ctx.accounts.system_program,
            amount
        )?;
        let global_vault = &mut ctx.accounts.global_vault;
        global_vault.total_assets = global_vault.total_assets.checked_add(amount).ok_or(MyError::MathOverflow)?;
        Ok(())
    }
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let global_vault = &mut ctx.accounts.global_vault;
    global_vault.admin = ctx.accounts.admin.key();
    global_vault.total_assets = 0;
    global_vault.total_shares = 0;
    Ok(())
}
}

#[derive(Accounts)]
pub struct Stake<'info> {
    ///CHECK: PDA này chỉ dùng để giữ Native SOL, an toàn không cần check.
    #[account(mut , seeds = [b"STAKING_VAULT"], bump)]
    pub staking_vault: UncheckedAccount<'info>,
    
    #[account(mut, seeds = [b"GLOBAL_STATE"], bump)]
    pub global_vault: Box<Account<'info, GlobalVault>>,
    
    #[account(init_if_needed, payer = payer, seeds = [b"USER_INFO", user.key().as_ref()], bump, space = 8 + 8)]
    pub user_info: Box<Account<'info, UserInfo>>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct AddRewards<'info>{
    #[account(
        mut,
        seeds = [b"GLOBAL_STATE"],
        bump,
        has_one = admin @MyError::Unauthorized
    )]
    pub global_vault: Box<Account<'info,GlobalVault>>,
    ///CHECK: PDA này chỉ dùng để giữ Native SOL, an toàn không cần check.
    #[account(mut, seeds = [b"STAKING_VAULT"], bump)]
    pub staking_vault: AccountInfo<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info,System>,
}
#[account]
#[derive(Default)]
pub struct UserInfo {
    pub shares: u64,
}

#[account]
#[derive(Default)]
pub struct GlobalVault {
    pub admin: Pubkey,
    pub total_assets: u64,
    pub total_shares: u64,
}
#[derive(Accounts)]
pub struct StakeToken<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub global_vault: Box<Account<'info, GlobalVault>>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub user_ata: Account<'info, TokenAccount>,
    /// CHECK: PDA Authority
    pub vault_authority: AccountInfo<'info>,
    #[account(mut)]
    pub token_vault: Account<'info, TokenAccount>,
    /// CHECK: User token state
    #[account(mut)]
    pub user_token_info: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init, 
        payer = admin, 
        seeds = [b"GLOBAL_STATE"], 
        bump, 
        space = 8 + 32 + 8 + 8 
    )]
    pub global_vault: Account<'info, GlobalVault>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}
#[error_code]
pub enum MyError {
    #[msg("Not enough shares")]
    NotEnoughShares,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Math overflow")]
    MathOverflow,
}