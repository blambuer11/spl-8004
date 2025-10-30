use anchor_lang::prelude::*;

pub const CONFIG_SEED: &[u8] = b"config";
pub const IDENTITY_SEED: &[u8] = b"identity";
pub const REPUTATION_SEED: &[u8] = b"reputation";
pub const VALIDATION_SEED: &[u8] = b"validation";
pub const REWARD_POOL_SEED: &[u8] = b"reward_pool";

pub const MAX_AGENT_ID_LEN: usize = 64;
pub const MAX_METADATA_URI_LEN: usize = 200;
pub const MAX_EVIDENCE_URI_LEN: usize = 200;

pub const INITIAL_REPUTATION_SCORE: u64 = 5000;
pub const MAX_REPUTATION_SCORE: u64 = 10000;
pub const MIN_REPUTATION_SCORE: u64 = 0;

pub const VALIDATION_FEE: u64 = 1_000_000; // 0.001 SOL
pub const REGISTRATION_FEE: u64 = 5_000_000; // 0.005 SOL

pub const DEFAULT_COMMISSION_RATE: u16 = 300; // 3%
pub const MAX_COMMISSION_RATE: u16 = 1000; // 10%

pub const REWARD_CLAIM_INTERVAL: i64 = 86400; // 24 hours in seconds
