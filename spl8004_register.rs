use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::constants::*;

#[derive(Accounts)]
#[instruction(agent_id: String)]
pub struct RegisterAgent<'info> {
    #[account(
        init,
        payer = owner,
        space = IdentityRegistry::LEN,
        seeds = [IDENTITY_SEED, agent_id.as_bytes()],
        bump
    )]
    pub identity: Account<'info, IdentityRegistry>,

    #[account(
        init,
        payer = owner,
        space = ReputationRegistry::LEN,
        seeds = [REPUTATION_SEED, identity.key().as_ref()],
        bump
    )]
    pub reputation: Account<'info, ReputationRegistry>,

    #[account(
        init,
        payer = owner,
        space = RewardPool::LEN,
        seeds = [REWARD_POOL_SEED, identity.key().as_ref()],
        bump
    )]
    pub reward_pool: Account<'info, RewardPool>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump
    )]
    pub config: Account<'info, GlobalConfig>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<RegisterAgent>,
    agent_id: String,
    metadata_uri: String,
) -> Result<()> {
    require!(
        agent_id.len() <= MAX_AGENT_ID_LEN,
        SPL8004Error::AgentIdTooLong
    );
    require!(
        metadata_uri.len() <= MAX_METADATA_URI_LEN,
        SPL8004Error::MetadataUriTooLong
    );

    let identity = &mut ctx.accounts.identity;
    let reputation = &mut ctx.accounts.reputation;
    let reward_pool = &mut ctx.accounts.reward_pool;
    let config = &mut ctx.accounts.config;
    let clock = Clock::get()?;

    identity.owner = ctx.accounts.owner.key();
    identity.agent_id = agent_id.clone();
    identity.metadata_uri = metadata_uri;
    identity.created_at = clock.unix_timestamp;
    identity.updated_at = clock.unix_timestamp;
    identity.is_active = true;
    identity.bump = ctx.bumps.identity;

    reputation.agent = identity.key();
    reputation.score = INITIAL_REPUTATION_SCORE;
    reputation.total_tasks = 0;
    reputation.successful_tasks = 0;
    reputation.failed_tasks = 0;
    reputation.last_updated = clock.unix_timestamp;
    reputation.stake_amount = 0;
    reputation.bump = ctx.bumps.reputation;

    reward_pool.agent = identity.key();
    reward_pool.claimable_amount = 0;
    reward_pool.last_claim = clock.unix_timestamp;
    reward_pool.total_claimed = 0;
    reward_pool.bump = ctx.bumps.reward_pool;

    config.total_agents = config.total_agents
        .checked_add(1)
        .ok_or(SPL8004Error::ArithmeticOverflow)?;

    msg!("Agent registered successfully!");
    msg!("Agent ID: {}", agent_id);
    msg!("Owner: {}", identity.owner);
    msg!("Initial reputation score: {}", reputation.score);

    Ok(())
}
