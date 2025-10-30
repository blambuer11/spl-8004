use anchor_lang::prelude::*;

#[error_code]
pub enum SPL8004Error {
    #[msg("Agent ID exceeds maximum length of 64 characters")]
    AgentIdTooLong,
    
    #[msg("Metadata URI exceeds maximum length of 200 characters")]
    MetadataUriTooLong,
    
    #[msg("Evidence URI exceeds maximum length of 200 characters")]
    EvidenceUriTooLong,
    
    #[msg("Agent is not active")]
    AgentNotActive,
    
    #[msg("Unauthorized: caller is not the agent owner")]
    Unauthorized,
    
    #[msg("Invalid reputation score")]
    InvalidReputationScore,
    
    #[msg("Validation already exists for this task hash")]
    ValidationAlreadyExists,
    
    #[msg("Insufficient reputation score for this action")]
    InsufficientReputation,
    
    #[msg("Commission rate exceeds maximum allowed (10%)")]
    InvalidCommissionRate,
    
    #[msg("Reward claim too early, must wait 24 hours")]
    RewardClaimTooEarly,
    
    #[msg("No rewards available to claim")]
    NoRewardsAvailable,
    
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    
    #[msg("Agent already registered")]
    AgentAlreadyRegistered,
}
