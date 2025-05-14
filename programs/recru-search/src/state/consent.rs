use anchor_lang::prelude::*;
use mpl_token_metadata::types::DataV2;

// Define constants
pub const CONSENT_NFT_SYMBOL: &str = "CONSENT";
pub const CONSENT_NFT_URI: &str = "https://recru-search.app/metadata/consent.json";

#[account]
pub struct Consent {
    pub authority: Pubkey,
    pub study_id: Pubkey,
    pub version: String,
    pub consent_hash: String,
    pub issued_at: i64,
    pub revoked_at: Option<i64>,
    pub is_active: bool,
    pub mint: Pubkey,
    pub bump: u8,
    pub total_issued: u64,
    pub total_revoked: u64,
    // Tracking valid consent versions
    pub consent_versions: Vec<String>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ConsentMetadata {
    pub study_id: Pubkey,
    pub participant_id: Pubkey,
    pub issued_at: i64,
    pub revoked_at: Option<i64>,
    pub consent_version: String,
    pub consent_hash: String,
}

impl ConsentMetadata {
    pub fn to_data_v2(&self) -> DataV2 {
        DataV2 {
            name: format!("Consent NFT - Study {}", self.study_id),
            symbol: crate::state::consent::CONSENT_NFT_SYMBOL.to_string(),
            uri: crate::state::consent::CONSENT_NFT_URI.to_string(),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        }
    }
}

impl Consent {
    pub fn create(
        &mut self,
        authority: Pubkey,
        study_id: Pubkey,
        version: String,
        consent_hash: String,
    ) -> Result<()> {
        self.authority = authority;
        self.study_id = study_id;
        self.version = version;
        self.consent_hash = consent_hash;
        self.issued_at = Clock::get()?.unix_timestamp;
        self.revoked_at = None;
        self.is_active = true;
        Ok(())
    }
    pub fn revoke(&mut self) -> Result<()> {
        require!(self.is_active, crate::error::RecruSearchError::InvalidConsentStatus);
        self.is_active = false;
        self.revoked_at = Some(Clock::get()?.unix_timestamp);
        Ok(())
    }
    pub fn is_valid(&self) -> bool {
        self.is_active && self.revoked_at.is_none()
    }
}

#[error_code]
pub enum ConsentError {
    #[msg("Unauthorized consent issuance")]
    UnauthorizedIssuance,
    #[msg("Consent already revoked")]
    AlreadyRevoked,
    #[msg("Invalid consent version")]
    InvalidVersion,
    #[msg("Invalid consent hash")]
    InvalidHash,
    #[msg("NFT minting failed")]
    MintingFailed,
    #[msg("NFT burning failed")]
    BurningFailed,
} 