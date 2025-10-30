use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct UpdateReputation<'info> {
    #[account(
        mut,
        seeds = [REPUTATION_SEED, agent.key().as_ref()],
        bump = reputation.bump,
        constraint = reputation.agent == agent.key()
    )]
    pub reputation: Account<'info, ReputationRegistry>,

    #[account(
        constraint = agent.is_active @ SPL8004Error::AgentNotActive
    )]
    pub agent: Account<'info, IdentityRegistry>,

    #[account(
        constraint = validation.agent == agent.key()
    )]
    pub validation: Account<'info, ValidationRegistry>,

    #[account(
        mut,
        seeds = [REWARD_POOL_SEED, agent.key().as_ref()],
        bump = reward_pool.bump
    )]
    pub reward_pool: Account<'info, RewardPool>,
}

pub fn handler(ctx: Context<UpdateReputation>) -> Result<()> {
    let reputation = &mut ctx.accounts.reputation;
    let validation = &ctx.accounts.validation;
    let reward_pool = &mut ctx.accounts.reward_pool;
    let clock = Clock::get()?;

    reputation.total_tasks = reputation.total_tasks
        .checked_add(1)
        .ok_or(SPL8004Error::ArithmeticOverflow)?;

    let score_change = reputation.calculate_score_change(validation.approved);
    
    if validation.approved {
        reputation.successful_tasks = reputation.successful_tasks
            .checked_add(1)
            .ok_or(SPL8004Error::ArithmeticOverflow)?;

        let new_score = (reputation.score as i64)
            .checked_add(score_change)
            .ok_or(SPL8004Error::ArithmeticOverflow)?;
        
        reputation.score = std::cmp::min(new_score as u64, MAX_REPUTATION_SCORE);

        let reward = calculate_reward(reputation.score, score_change as u64);
        reward_pool.claimable_amount = reward_pool.claimable_amount
            .checked_add(reward)
            .ok_or(SPL8004Error::ArithmeticOverflow)?;

        msg!("Task approved! Score increased by {}", score_change);
        msg!("Reward added: {} lamports", reward);
    } else {
        reputation.failed_tasks = reputation.failed_tasks
            .checked_add(1)
            .ok_or(SPL8004Error::ArithmeticOverflow)?;

        let new_score = (reputation.score as i64)
            .checked_add(score_change)
            .ok_or(SPL8004Error::ArithmeticOverflow)?;
        
        reputation.score = std::cmp::max(new_score as u64, MIN_REPUTATION_SCORE);

        msg!("Task failed! Score decreased by {}", score_change.abs());
    }

    reputation.last_updated = clock.unix_timestamp;

    msg!("Reputation updated for agent: {}", reputation.agent);
    msg!("New score: {}/10000", reputation.score);
    msg!("Success rate: {}%", reputation.success_rate());
    msg!("Total tasks: {}", reputation.total_tasks);

    Ok(())
}

fn calculate_reward(current_score: u64, score_increase: u64) -> u64 {
    let base_reward = 100_000; // 0.0001 SOL base
    
    let score_multiplier = match current_score {
        9000..=10000 => 5,
        8000..=8999 => 4,
        7000..=7999 => 3,
        6000..=6999 => 2,
        _ => 1,
    };

    base_reward * score_multiplier * score_increase / 100
}
