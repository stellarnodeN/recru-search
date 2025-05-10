//! Consent management module for the RecruSearch program
//! 
//! This module handles all consent-related functionality including:
//! - Consent NFT management
//! - Consent tracking and verification
//! - Consent revocation
//! - Privacy-preserving consent operations

use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, Mint, Token, TokenAccount},
};
use metaplex_core::nft::{
    Nft,
    NftMetadata,
    create_metadata_accounts_v3,
    create_master_edition_v3,
};
use crate::error::RecruSearchError;

/// Constants for consent NFT metadata
pub const CONSENT_NFT_NAME: &str = "Research Consent";
pub const CONSENT_NFT_SYMBOL: &str = "RCONSENT";
pub const CONSENT_NFT_URI: &str = "https://recrusearch.io/consent-metadata";

/// Represents a consent record in the platform
#[account]
pub struct Consent {
    /// The participant's wallet authority
    pub authority: Pubkey,
    /// The study ID this consent is for
    pub study_id: Pubkey,
    /// Version of the consent form
    pub version: String,
    /// Hash of the consent document
    pub consent_hash: String,
    /// Timestamp when consent was given
    pub issued_at: i64,
    /// Timestamp when consent was revoked (if applicable)
    pub revoked_at: Option<i64>,
    /// Whether the consent is currently active
    pub is_active: bool,
}

/// Context for initializing the consent NFT
#[derive(Accounts)]
pub struct InitializeConsentNFT<'info> {
    /// The authority initializing the consent NFT
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// The mint account for the consent NFT
    #[account(
        init,
        payer = authority,
        mint::decimals = 0,
        mint::authority = authority,
    )]
    pub mint: Account<'info, Mint>,
    
    /// The metadata account for the consent NFT
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    
    /// The master edition account for the consent NFT
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    
    /// The token program
    pub token_program: Program<'info, Token>,
    
    /// The system program
    pub system_program: Program<'info, System>,
    
    /// The rent sysvar
    pub rent: Sysvar<'info, Rent>,
    
    /// The metadata program
    pub metadata_program: Program<'info, NftMetadata>,
}

/// Context for issuing a consent NFT
#[derive(Accounts)]
pub struct IssueConsentNFT<'info> {
    /// The participant receiving the consent NFT
    #[account(mut)]
    pub participant: Account<'info, super::Participant>,
    
    /// The study this consent is for
    #[account(mut)]
    pub study: Account<'info, super::Study>,
    
    /// The mint account for the consent NFT
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    /// The participant's token account
    #[account(
        init,
        payer = authority,
        token::mint = mint,
        token::authority = participant,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    /// The authority (participant) signing the transaction
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// The token program
    pub token_program: Program<'info, Token>,
    
    /// The system program
    pub system_program: Program<'info, System>,
}

/// Context for revoking consent
#[derive(Accounts)]
pub struct RevokeConsent<'info> {
    /// The participant revoking consent
    #[account(mut)]
    pub participant: Account<'info, super::Participant>,
    
    /// The study this consent is for
    #[account(mut)]
    pub study: Account<'info, super::Study>,
    
    /// The mint account for the consent NFT
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    /// The participant's token account
    #[account(
        mut,
        constraint = token_account.owner == participant.authority,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    /// The authority (participant) signing the transaction
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// The token program
    pub token_program: Program<'info, Token>,
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
            symbol: CONSENT_NFT_SYMBOL.to_string(),
            uri: CONSENT_NFT_URI.to_string(),
            seller_fee_basis_points: 0,
            creators: Some(vec![Creator {
                address: self.study_id,
                verified: true,
                share: 100,
            }]),
            collection: None,
            uses: None,
        }
    }
}

impl Consent {
    /// Creates a new consent record
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

    /// Revokes the consent
    pub fn revoke(&mut self) -> Result<()> {
        require!(self.is_active, RecruSearchError::InvalidConsentStatus);
        self.is_active = false;
        self.revoked_at = Some(Clock::get()?.unix_timestamp);
        Ok(())
    }

    /// Verifies if the consent is valid
    pub fn is_valid(&self) -> bool {
        self.is_active && self.revoked_at.is_none()
    }
}

#[error_code]
pub enum ConsentError {
    /// Attempted to issue consent without proper authorization
    #[msg("Unauthorized consent issuance")]
    UnauthorizedIssuance,
    
    /// Attempted to revoke consent that is already revoked
    #[msg("Consent already revoked")]
    AlreadyRevoked,
    
    /// Invalid consent version provided
    #[msg("Invalid consent version")]
    InvalidVersion,
    
    /// Invalid consent hash provided
    #[msg("Invalid consent hash")]
    InvalidHash,
    
    /// NFT minting failed
    #[msg("NFT minting failed")]
    MintingFailed,
    
    /// NFT burning failed
    #[msg("NFT burning failed")]
    BurningFailed,
} 