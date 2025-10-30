use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct UpdateMetadata<'info> {
    #[account(
        mut,
        has_one = owner @ SPL8004Error::Unauthorized,
        constraint = identity.is_active @ SPL8004Error::AgentNotActive
    )]
    pub identity: Account<'info, IdentityRegistry>,

    pub owner: Signer<'info>,
}

pub fn handler(
    ctx: Context<UpdateMetadata>,
    new_metadata_uri: String,
) -> Result<()> {
    require!(
        new_metadata_uri.len() <= MAX_METADATA_URI_LEN,
        SPL8004Error::MetadataUriTooLong
    );

    let identity = &mut ctx.accounts.identity;
    let clock = Clock::get()?;

    let old_uri = identity.metadata_uri.clone();
    identity.metadata_uri = new_metadata_uri.clone();
    identity.updated_at = clock.unix_timestamp;

    msg!("Metadata updated for agent: {}", identity.agent_id);
    msg!("Old URI: {}", old_uri);
    msg!("New URI: {}", new_metadata_uri);

    Ok(())
}
