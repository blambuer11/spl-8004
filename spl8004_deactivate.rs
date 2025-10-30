use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct DeactivateAgent<'info> {
    #[account(
        mut,
        has_one = owner @ SPL8004Error::Unauthorized,
        constraint = identity.is_active @ SPL8004Error::AgentNotActive
    )]
    pub identity: Account<'info, IdentityRegistry>,

    pub owner: Signer<'info>,
}

pub fn handler(ctx: Context<DeactivateAgent>) -> Result<()> {
    let identity = &mut ctx.accounts.identity;
    let clock = Clock::get()?;

    identity.is_active = false;
    identity.updated_at = clock.unix_timestamp;

    msg!("Agent deactivated: {}", identity.agent_id);
    msg!("Owner: {}", identity.owner);

    Ok(())
}
