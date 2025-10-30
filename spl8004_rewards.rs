use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::state::*;
use crate::errors::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(
        has_one = owner @ SPL8004Error::Unauthorized,
        constraint = identity.is_active @ SPL8004Error::AgentNotActive
    )]
    pub identity: Account<'info, IdentityRegistry>,

    #[account(
        mut,
        seeds = [REWARD_POOL_SEED, identity.key().as_ref()],
        bump = reward_pool.bump
    )]
    pub reward_pool: Account<'info, RewardPool>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ClaimRewards>) -> Result<()> {
    let reward_pool = &mut ctx.accounts.reward_pool;
    let clock = Clock::get()?;

    require!(
        reward_pool.claimable_amount > 0,
        SPL8004Error::NoRewardsAvailable
    );

    require!(
        clock.unix_timestamp >= reward_pool.last_claim + REWARD_CLAIM_INTERVAL,
        SPL8004Error::RewardClaimTooEarly
    );

    let amount = reward_pool.claimable_amount;

    **reward_pool.to_account_info().try_borrow_mut_lamports()? = reward_pool
        .to_account_info()
        .lamports()
        .checked_sub(amount)
        .ok_or(SPL8004Error::ArithmeticOverflow)?;

    **ctx.accounts.owner.to_account_info().try_borrow_mut_lamports()? = ctx
        .accounts
        .owner
        .to_account_info()
        .lamports()
        .checked_add(amount)
        .ok_or(SPL8004Error::ArithmeticOverflow)?;

    reward_pool.total_claimed = reward_pool.total_claimed
        .checked_add(amount)
        .ok_or(SPL8004Error::ArithmeticOverflow)?;
    
    reward_pool.claimable_amount = 0;
    reward_pool.last_claim = clock.unix_timestamp;

    msg!("Rewards claimed for agent: {}", ctx.accounts.identity.agent_id);
    msg!("Amount: {} lamports ({} SOL)", amount, amount as f64 / 1e9);
    msg!("Total claimed: {} lamports", reward_pool.total_claimed);

    Ok(())
}
