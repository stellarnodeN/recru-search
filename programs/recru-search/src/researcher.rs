//! Researcher management module for the RecruSearch program
//! 
//! This module handles all researcher-related functionality including:
//! - Researcher registration and verification
//! - Study creation and management
//! - Participant interaction
//! - Research credentials management

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use crate::error::RecruSearchError;

/// Represents a researcher in the platform
#[account]
pub struct Researcher {
    /// The researcher's wallet authority
    pub authority: Pubkey,
    /// Institution or organization the researcher belongs to
    pub institution: String,
    /// Hash of the researcher's credentials for verification
    pub credentials_hash: String,
    /// Whether the researcher has been verified by an admin
    pub is_verified: bool,
    /// Timestamp when the researcher registered
    pub registered_at: i64,
    /// Number of studies created by the researcher
    pub studies_created: u32,
    /// Number of active studies
    pub active_studies: u32,
    /// Total number of participants across all studies
    pub total_participants: u32,
    /// Researcher's reputation score
    pub reputation_score: u32,
}

/// Context for registering a new researcher
#[derive(Accounts)]
pub struct RegisterResearcher<'info> {
    /// The researcher account to be initialized
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 100 + 100 + 1 + 8 + 4 + 4 + 4 + 4
    )]
    pub researcher: Account<'info, Researcher>,
    
    /// The authority (researcher) signing the transaction
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// The system program
    pub system_program: Program<'info, System>,
}

/// Context for verifying a researcher
#[derive(Accounts)]
pub struct VerifyResearcher<'info> {
    /// The researcher account to be verified
    #[account(mut)]
    pub researcher: Account<'info, Researcher>,
    
    /// The admin account performing the verification
    pub admin: Account<'info, super::Admin>,
    
    /// The authority (admin) signing the transaction
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ResearcherProfile {
    pub institution: String,
    pub credentials: Vec<Credential>,
    pub is_verified: bool,
    pub created_studies: Vec<Pubkey>,
    pub active_studies: Vec<Pubkey>,
    pub completed_studies: Vec<Pubkey>,
    pub reputation_score: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Credential {
    pub credential_type: String,
    pub issuer: String,
    pub issue_date: i64,
    pub expiry_date: Option<i64>,
    pub verification_status: VerificationStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum VerificationStatus {
    Pending,
    Verified,
    Rejected,
}

#[derive(Accounts)]
pub struct CreateStudy<'info> {
    #[account(mut)]
    pub researcher: Account<'info, super::Researcher>,
    #[account(mut)]
    pub study: Account<'info, super::Study>,
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SelectParticipants<'info> {
    #[account(mut)]
    pub researcher: Account<'info, super::Researcher>,
    #[account(mut)]
    pub study: Account<'info, super::Study>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct DistributeRewards<'info> {
    #[account(mut)]
    pub researcher: Account<'info, super::Researcher>,
    #[account(mut)]
    pub study: Account<'info, super::Study>,
    #[account(mut)]
    pub researcher_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateStudyResults<'info> {
    #[account(mut)]
    pub researcher: Account<'info, super::Researcher>,
    #[account(mut)]
    pub study: Account<'info, super::Study>,
    pub authority: Signer<'info>,
}

impl Researcher {
    /// Creates a new researcher with the specified parameters
    pub fn create(
        &mut self,
        authority: Pubkey,
        institution: String,
        credentials_hash: String,
    ) -> Result<()> {
        require!(institution.len() > 0, RecruSearchError::InvalidInstitutionName);
        require!(credentials_hash.len() > 0, RecruSearchError::InvalidCredentials);
        
        self.authority = authority;
        self.institution = institution;
        self.credentials_hash = credentials_hash;
        self.is_verified = false;
        self.registered_at = Clock::get()?.unix_timestamp;
        self.studies_created = 0;
        self.active_studies = 0;
        self.total_participants = 0;
        self.reputation_score = 0;
        
        // Create initial verification status
        self.verification_status = VerificationStatus::Pending;
        self.verification_requested_at = Clock::get()?.unix_timestamp;
        
        Ok(())
    }

    /// Verifies the researcher's credentials
    pub fn verify(&mut self, admin: &Account<Admin>) -> Result<()> {
        require!(admin.authority == self.authority, RecruSearchError::UnauthorizedAccess);
        require!(self.verification_status == VerificationStatus::Pending, 
            RecruSearchError::AlreadyVerified);
        
        self.is_verified = true;
        self.verification_status = VerificationStatus::Verified;
        self.verified_at = Clock::get()?.unix_timestamp;
        self.reputation_score = 100; // Initial reputation score for verified researchers
        
        admin.dashboard.total_researchers += 1;
        admin.dashboard.last_updated = Clock::get()?.unix_timestamp;
        
        Ok(())
    }

    /// Increments the number of studies created
    pub fn increment_studies_created(&mut self) -> Result<()> {
        self.studies_created = self.studies_created.checked_add(1)
            .ok_or(RecruSearchError::InvalidStudyParameters)?;
        Ok(())
    }

    /// Updates the researcher's active studies count
    pub fn update_active_studies(&mut self, delta: i32) -> Result<()> {
        require!(self.is_verified, RecruSearchError::NotVerified);
        
        if delta > 0 {
            self.active_studies = self.active_studies.checked_add(delta as u32)
                .ok_or(RecruSearchError::InvalidStudyParameters)?;
        } else {
            self.active_studies = self.active_studies.checked_sub((-delta) as u32)
                .ok_or(RecruSearchError::InvalidStudyParameters)?;
        }
        
        // Update reputation score based on study activity
        self.update_reputation_score(
            self.reputation_score + (delta.abs() as u32 * 10) // +10 points per study
        )?;
        
        Ok(())
    }

    /// Updates the total number of participants
    pub fn update_total_participants(&mut self, delta: i32) -> Result<()> {
        if delta > 0 {
            self.total_participants = self.total_participants.checked_add(delta as u32)
                .ok_or(RecruSearchError::InvalidStudyParameters)?;
        } else {
            self.total_participants = self.total_participants.checked_sub((-delta) as u32)
                .ok_or(RecruSearchError::InvalidStudyParameters)?;
        }
        Ok(())
    }

    /// Updates the researcher's reputation score
    pub fn update_reputation_score(&mut self, score: u32) -> Result<()> {
        self.reputation_score = score;
        Ok(())
    }
}

impl ResearcherProfile {
    pub fn can_create_study(&self) -> bool {
        self.is_verified && self.reputation_score >= 50
    }

    pub fn update_reputation(&mut self, study: &super::Study) {
        let base_score = 10;
        let completion_bonus = (study.analytics.completion_rate * 20.0) as u32;
        let rating_bonus = (study.analytics.average_rating * 10.0) as u32;
        
        self.reputation_score += base_score + completion_bonus + rating_bonus;
    }

    pub fn verify_credentials(&mut self) -> bool {
        let valid_credentials = self.credentials.iter()
            .filter(|cred| {
                if let Some(expiry) = cred.expiry_date {
                    let now = Clock::get().unwrap().unix_timestamp;
                    now <= expiry
                } else {
                    true
                }
            })
            .count();

        self.is_verified = valid_credentials >= 2;
        self.is_verified
    }
}

#[error_code]
pub enum ResearcherError {
    /// Attempted to create a study without verified researcher status
    #[msg("Researcher not verified")]
    NotVerified,
    
    /// Attempted to exceed maximum allowed studies
    #[msg("Maximum studies limit reached")]
    MaxStudiesReached,
    
    /// Invalid institution information provided
    #[msg("Invalid institution information")]
    InvalidInstitution,
    
    /// Invalid credentials provided
    #[msg("Invalid credentials")]
    InvalidCredentials,
    
    /// Researcher already registered
    #[msg("Researcher already registered")]
    AlreadyRegistered,
} 