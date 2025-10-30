use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(
        init,
        payer = authority,
        space = GlobalConfig::LEN,
        seeds = [CONFIG_SEED],
        bump
    )]
    pub config: Account<'info, GlobalConfig>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeConfig>,
    commission_rate: u16,
    treasury: Pubkey,
) -> Result<()> {
    require!(
        commission_rate <= MAX_COMMISSION_RATE,
        SPL8004Error::InvalidCommissionRate
    );

    let config = &mut ctx.accounts.config;
    
    config.authority = ctx.accounts.authority.key();
    config.treasury = treasury;
    config.commission_rate = commission_rate;
    config.total_agents = 0;
    config.total_validations = 0;
    config.bump = ctx.bumps.config;

    msg!("Global config initialized");
    msg!("Authority: {}", config.authority);
    msg!("Treasury: {}", config.treasury);
    msg!("Commission rate: {}%", commission_rate as f64 / 100.0);

    Ok(())
}
