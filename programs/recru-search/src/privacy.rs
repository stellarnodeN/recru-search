use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use mpl_token_metadata::state::Metadata;
use std::collections::HashMap;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PrivacyManager {
    pub participant: Pubkey,
    pub master_key: [u8; 32],  // Encrypted master key
    pub data_keys: HashMap<Pubkey, DataKey>,  // Study-specific data keys
    pub consent_tokens: Vec<ConsentToken>,
    pub pseudonym: String,     // Participant's pseudonym
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct DataKey {
    pub encrypted_key: [u8; 32],  // Encrypted with researcher's public key
    pub access_level: DataAccessLevel,
    pub created_at: i64,
    pub last_used: i64,
    pub is_revoked: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ConsentToken {
    pub study_id: Pubkey,
    pub token_id: String,      // Unique token for off-chain data access
    pub access_level: DataAccessLevel,
    pub granted_at: i64,
    pub expires_at: Option<i64>,
    pub is_revoked: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum DataAccessLevel {
    Basic,      // Only study-specific data
    Extended,   // Including demographic data
    Full,       // All participant data
}

#[derive(Accounts)]
pub struct InitializePrivacyManager<'info> {
    #[account(mut)]
    pub participant: Account<'info, super::Participant>,
    #[account(mut)]
    pub privacy_manager: Account<'info, PrivacyManager>,
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GrantDataAccess<'info> {
    #[account(mut)]
    pub privacy_manager: Account<'info, PrivacyManager>,
    #[account(mut)]
    pub study: Account<'info, super::Study>,
    pub researcher: Account<'info, super::Researcher>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct RevokeDataAccess<'info> {
    #[account(mut)]
    pub privacy_manager: Account<'info, PrivacyManager>,
    #[account(mut)]
    pub study: Account<'info, super::Study>,
    pub authority: Signer<'info>,
}

impl PrivacyManager {
    pub fn generate_pseudonym(&mut self) -> String {
        // Generate a unique pseudonym that can't be linked to the participant
        // This would typically use a cryptographic hash of some participant data
        // combined with a random salt
        format!("P{:x}", self.participant.to_bytes()[0..8].to_vec())
    }

    pub fn grant_access(&mut self, study_id: Pubkey, researcher_pubkey: Pubkey, access_level: DataAccessLevel) -> Result<()> {
        // Generate a new data key for this study
        let data_key = DataKey {
            encrypted_key: [0; 32], // This would be encrypted with researcher's public key
            access_level,
            created_at: Clock::get()?.unix_timestamp,
            last_used: Clock::get()?.unix_timestamp,
            is_revoked: false,
        };

        // Store the encrypted data key
        self.data_keys.insert(study_id, data_key);

        // Generate a consent token for off-chain data access
        let consent_token = ConsentToken {
            study_id,
            token_id: format!("{:x}", study_id.to_bytes()[0..8].to_vec()),
            access_level,
            granted_at: Clock::get()?.unix_timestamp,
            expires_at: None,
            is_revoked: false,
        };

        self.consent_tokens.push(consent_token);
        Ok(())
    }

    pub fn revoke_access(&mut self, study_id: Pubkey) -> Result<()> {
        if let Some(data_key) = self.data_keys.get_mut(&study_id) {
            data_key.is_revoked = true;
        }

        if let Some(consent_token) = self.consent_tokens.iter_mut()
            .find(|token| token.study_id == study_id) {
            consent_token.is_revoked = true;
        }

        Ok(())
    }

    pub fn verify_access(&self, study_id: Pubkey, required_level: DataAccessLevel) -> bool {
        if let Some(data_key) = self.data_keys.get(&study_id) {
            if data_key.is_revoked {
                return false;
            }

            match (data_key.access_level, required_level) {
                (DataAccessLevel::Full, _) => true,
                (DataAccessLevel::Extended, DataAccessLevel::Extended) => true,
                (DataAccessLevel::Extended, DataAccessLevel::Basic) => true,
                (DataAccessLevel::Basic, DataAccessLevel::Basic) => true,
                _ => false,
            }
        } else {
            false
        }
    }
}

#[error_code]
pub enum PrivacyError {
    #[msg("Privacy manager not initialized")]
    NotInitialized,
    #[msg("Access already granted")]
    AccessAlreadyGranted,
    #[msg("Access not found")]
    AccessNotFound,
    #[msg("Access already revoked")]
    AccessAlreadyRevoked,
    #[msg("Invalid access level")]
    InvalidAccessLevel,
    #[msg("Encryption failed")]
    EncryptionFailed,
    #[msg("Decryption failed")]
    DecryptionFailed,
} 