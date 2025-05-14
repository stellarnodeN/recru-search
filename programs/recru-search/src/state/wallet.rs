use anchor_lang::prelude::*;

/// Represents a participant's wallet in the RecruSearch platform.
#[account]
pub struct ParticipantWallet {
    pub participant: Pubkey,
    pub phantom_public_key: Pubkey,
    pub bump: u8,
    pub created_at: i64,
    pub last_activity: i64,
    pub total_rewards: u64,
    pub last_reward_at: i64,
    pub is_active: bool,
    pub metadata_uri: Option<String>,
}

impl ParticipantWallet {
    pub fn new(participant: Pubkey, phantom_public_key: Pubkey, bump: u8) -> Result<Self> {
        let now = Clock::get()?.unix_timestamp;
        Ok(Self {
            participant,
            phantom_public_key,
            bump,
            created_at: now,
            last_activity: now,
            total_rewards: 0,
            last_reward_at: 0,
            is_active: true,
            metadata_uri: None,
        })
    }

    pub fn add_reward(&mut self, amount: u64) -> Result<()> {
        require!(self.is_active, crate::error::RecruSearchError::InactiveWallet);
        require!(amount > 0, crate::error::RecruSearchError::InvalidTokenAmount);

        self.total_rewards = self.total_rewards
            .checked_add(amount)
            .ok_or(crate::error::RecruSearchError::RewardOverflow)?;
        self.last_reward_at = Clock::get()?.unix_timestamp;
        self.last_activity = self.last_reward_at;
        Ok(())
    }

    pub fn update_activity(&mut self) -> Result<()> {
        self.last_activity = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn toggle_active(&mut self) -> Result<()> {
        self.is_active = !self.is_active;
        self.update_activity()
    }

    pub fn update_metadata_uri(&mut self, uri: Option<String>) -> Result<()> {
        self.metadata_uri = uri;
        self.update_activity()
    }
}

pub fn get_wallet_size() -> usize {
    let mut size = 8; // discriminator
    size += 32; // participant
    size += 32; // phantom_public_key
    size += 1;  // bump
    size += 8;  // created_at
    size += 8;  // last_activity
    size += 8;  // total_rewards
    size += 8;  // last_reward_at
    size += 1;  // is_active
    size += 1 + 4 + 200; // Option<String>: discriminator + length + max URI
    size
}
