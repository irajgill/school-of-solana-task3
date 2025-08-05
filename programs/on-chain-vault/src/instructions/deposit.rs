//-------------------------------------------------------------------------
///
/// TASK: Implement the deposit functionality for the on-chain vault
/// 
/// Requirements:
/// - Verify that the user has enough balance to deposit
/// - Verify that the vault is not locked
/// - Transfer lamports from user to vault using CPI (Cross-Program Invocation)
/// - Emit a deposit event after successful transfer
/// 
///-------------------------------------------------------------------------

use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use crate::state::Vault;
use crate::errors::VaultError;
use crate::events::DepositEvent;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault", vault.vault_authority.key().as_ref()],
        bump,
        constraint = !vault.locked @ VaultError::VaultLocked
    )]
    pub vault: Account<'info, Vault>,
    pub system_program: Program<'info, System>,
}

pub fn _deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    let user = &ctx.accounts.user;
    let vault = &ctx.accounts.vault;
    
    // Check if user has enough balance (minimum rent exemption + amount)
    let user_balance = user.lamports();
    let minimum_balance = Rent::get()?.minimum_balance(0); // Minimum rent for empty account
    
    require!(
        user_balance >= amount.checked_add(minimum_balance).ok_or(VaultError::Overflow)?,
        VaultError::InsufficientBalance
    );
    
    // Transfer lamports from user to vault using CPI
    let transfer_accounts = Transfer {
        from: user.to_account_info(),
        to: vault.to_account_info(),
    };
    
    let cpi_context = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        transfer_accounts,
    );
    
    transfer(cpi_context, amount)?;
    
    // Emit deposit event
    emit!(DepositEvent {
        amount,
        user: user.key(),
        vault: vault.key(),
    });
    
    Ok(())
}
