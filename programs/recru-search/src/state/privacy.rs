use anchor_lang::prelude::*;
use std::collections::HashMap;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PrivacyManager {
    pub participant: Pubkey,
    pub master_key: [u8; 32],
    pub data_keys: HashMap<Pubkey, DataKey>,
    pub consent_tokens: Vec<ConsentToken>,
    pub pseudonym: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct DataKey {
    pub encrypted_key: [u8; 32],
    pub access_level: DataAccessLevel,
    pub created_at: i64,
    pub last_used: i64,
    pub is_revoked: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ConsentToken {
    pub study_id: Pubkey,
    pub token_id: String,
    pub access_level: DataAccessLevel,
    pub granted_at: i64,
    pub expires_at: Option<i64>,
    pub is_revoked: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum DataAccessLevel {
    Basic,
    Extended,
    Full,
}

impl PrivacyManager {
    pub fn generate_pseudonym(&mut self) -> String {
        let bytes = &self.participant.to_bytes()[0..8];
        let hex = bytes.iter().fold(String::new(), |mut output, byte| {
            output.push_str(&format!("{:02x}", byte));
            output
        });
        format!("P{}", hex)
    }
    pub fn grant_access(&mut self, study_id: Pubkey, _researcher_pubkey: Pubkey, access_level: DataAccessLevel) -> Result<()> {
        let data_key = DataKey {
            encrypted_key: [0; 32],
            access_level,
            created_at: Clock::get()?.unix_timestamp,
            last_used: Clock::get()?.unix_timestamp,
            is_revoked: false,
        };
        self.data_keys.insert(study_id, data_key);
        let consent_token = ConsentToken {
            study_id,
            token_id: {  
                let bytes = &study_id.to_bytes()[0..8];
                let hex = bytes.iter().fold(String::new(), |mut output, byte| {
                    output.push_str(&format!("{:02x}", byte));
                    output
                });
                hex
            },
            access_level: access_level, // Fix the moved value issue with access_level
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
            // Since we've added Copy trait to DataAccessLevel, we can now use it directly
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