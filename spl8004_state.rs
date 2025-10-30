use anchor_lang::prelude::*;

#[account]
pub struct GlobalConfig {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub commission_rate: u16,
    pub total_agents: u64,
    pub total_validations: u64,
    pub bump: u8,
}

impl GlobalConfig {
    pub const LEN: usize = 8 + 32 + 32 + 2 + 8 + 8 + 1;
}

#[account]
pub struct IdentityRegistry {
    pub owner: Pubkey,
    pub agent_id: String,
    pub metadata_uri: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub is_active: bool,
    pub bump: u8,
}

impl IdentityRegistry {
    pub const LEN: usize = 8 + 32 + (4 + 64) + (4 + 200) + 8 + 8 + 1 + 1;
}

#[account]
pub struct ReputationRegistry {
    pub agent: Pubkey,
    pub score: u64,
    pub total_tasks: u64,
    pub successful_tasks: u64,
    pub failed_tasks: u64,
    pub last_updated: i64,
    pub stake_amount: u64,
    pub bump: u8,
}

impl ReputationRegistry {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 1;

    pub fn success_rate(&self) -> u8 {
        if self.total_tasks == 0 {
            return 100;
        }
        ((self.successful_tasks * 100) / self.total_tasks) as u8
    }

    pub fn calculate_score_change(&self, approved: bool) -> i64 {
        let success_rate = self.success_rate();
        
        if approved {
            match success_rate {
                90..=100 => 100,
                80..=89 => 75,
                70..=79 => 50,
                _ => 25,
            }
        } else {
            match success_rate {
                0..=50 => -150,
                51..=70 => -100,
                _ => -50,
            }
        }
    }
}

#[account]
pub struct ValidationRegistry {
    pub agent: Pubkey,
    pub validator: Pubkey,
    pub task_hash: [u8; 32],
    pub approved: bool,
    pub timestamp: i64,
    pub evidence_uri: String,
    pub bump: u8,
}

impl ValidationRegistry {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 1 + 8 + (4 + 200) + 1;
}

#[account]
pub struct RewardPool {
    pub agent: Pubkey,
    pub claimable_amount: u64,
    pub last_claim: i64,
    pub total_claimed: u64,
    pub bump: u8,
}

impl RewardPool {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 8 + 1;
}
