use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::state::*;
use crate::errors::*;
use crate::constants::*;

#[derive(Accounts)]
#[instruction(task_hash: [u8; 32])]
pub struct SubmitValidation<'info> {
    #[account(
        init,
        payer = validator,
        space = ValidationRegistry::LEN,
        seeds = [
            VALIDATION_SEED,
            agent.key().as_ref(),
            task_hash.as_ref()
        ],
        bump
    )]
    pub validation: Account<'info, ValidationRegistry>,

    #[account(
        constraint = agent.is_active @ SPL8004Error::AgentNotActive
    )]
    pub agent: Account<'info, IdentityRegistry>,

    #[account(mut)]
    pub validator: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump
    )]
    pub config: Account<'info, GlobalConfig>,

    #[account(
        mut,
        constraint = treasury.key() == config.treasury
    )]
    /// CHECK: Treasury account verified via config
    pub treasury: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<SubmitValidation>,
    task_hash: [u8; 32],
    approved: bool,
    evidence_uri: String,
) -> Result<()> {
    require!(
        evidence_uri.len() <= MAX_EVIDENCE_URI_LEN,
        SPL8004Error::EvidenceUriTooLong
    );

    let validation = &mut ctx.accounts.validation;
    let config = &mut ctx.accounts.config;
    let clock = Clock::get()?;

    validation.agent = ctx.accounts.agent.key();
    validation.validator = ctx.accounts.validator.key();
    validation.task_hash = task_hash;
    validation.approved = approved;
    validation.timestamp = clock.unix_timestamp;
    validation.evidence_uri = evidence_uri;
    validation.bump = ctx.bumps.validation;

    config.total_validations = config.total_validations
        .checked_add(1)
        .ok_or(SPL8004Error::ArithmeticOverflow)?;

    if config.commission_rate > 0 {
        let commission = (VALIDATION_FEE as u128)
            .checked_mul(config.commission_rate as u128)
            .ok_or(SPL8004Error::ArithmeticOverflow)?
            .checked_div(10000)
            .ok_or(SPL8004Error::ArithmeticOverflow)? as u64;

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.validator.to_account_info(),
                    to: ctx.accounts.treasury.to_account_info(),
                },
            ),
            commission,
        )?;

        msg!("Commission collected: {} lamports", commission);
    }

    msg!("Validation submitted for agent: {}", validation.agent);
    msg!("Task hash: {:?}", task_hash);
    msg!("Approved: {}", approved);
    msg!("Validator: {}", validation.validator);

    Ok(())
}
