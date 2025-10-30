use anchor_lang::prelude::*;

declare_id!("SPL8wVx7ZqKNxJk5H2bF8QyGvM4tN3rP9WdE6fU5Kc2");

pub mod state;
pub mod instructions;
pub mod errors;
pub mod constants;

use instructions::*;

#[program]
pub mod spl_8004 {
    use super::*;

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        commission_rate: u16,
        treasury: Pubkey,
    ) -> Result<()> {
        instructions::initialize_config::handler(ctx, commission_rate, treasury)
    }

    pub fn register_agent(
        ctx: Context<RegisterAgent>,
        agent_id: String,
        metadata_uri: String,
    ) -> Result<()> {
        instructions::register_agent::handler(ctx, agent_id, metadata_uri)
    }

    pub fn update_metadata(
        ctx: Context<UpdateMetadata>,
        new_metadata_uri: String,
    ) -> Result<()> {
        instructions::update_metadata::handler(ctx, new_metadata_uri)
    }

    pub fn submit_validation(
        ctx: Context<SubmitValidation>,
        task_hash: [u8; 32],
        approved: bool,
        evidence_uri: String,
    ) -> Result<()> {
        instructions::submit_validation::handler(ctx, task_hash, approved, evidence_uri)
    }

    pub fn update_reputation(
        ctx: Context<UpdateReputation>,
    ) -> Result<()> {
        instructions::update_reputation::handler(ctx)
    }

    pub fn deactivate_agent(
        ctx: Context<DeactivateAgent>,
    ) -> Result<()> {
        instructions::deactivate_agent::handler(ctx)
    }

    pub fn claim_rewards(
        ctx: Context<ClaimRewards>,
    ) -> Result<()> {
        instructions::claim_rewards::handler(ctx)
    }
}
