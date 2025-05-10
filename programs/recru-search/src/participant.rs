//! Participant management module for the RecruSearch program
//! 
//! This module handles all participant-related functionality including:
//! - Participant registration and profile management
//! - Study participation tracking
//! - Consent management
//! - Reward tracking

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use crate::error::RecruSearchError;

/// Represents a participant's profile information
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ParticipantProfile {
    /// Participant's age group
    pub age_group: String,
    /// Participant's gender
    pub gender: String,
    /// Participant's geographic region
    pub region: String,
    /// List of participant's interests
    pub interests: Vec<String>,
    /// Whether the participant has opted for anonymous participation
    pub is_anonymous: bool,
    pub completed_studies: Vec<CompletedStudy>,
    pub active_studies: Vec<ActiveStudy>,
    pub reward_history: Vec<RewardRecord>,
    pub reputation_score: u32,
}

/// Represents a participant in the platform
#[account]
pub struct Participant {
    /// The participant's wallet authority
    pub authority: Pubkey,
    /// Participant's profile information
    pub profile: ParticipantProfile,
    /// Hash of the participant's eligibility proof
    pub eligibility_proof: String,
    /// Timestamp when the participant registered
    pub registered_at: i64,
    /// Number of studies the participant is currently active in
    pub active_studies: u32,
    /// Number of studies the participant has completed
    pub completed_studies: u32,
    /// Whether the participant has active consent
    pub has_active_consent: bool,
    /// Timestamp when consent was issued
    pub consent_issued_at: i64,
    /// Timestamp when consent was revoked (if applicable)
    pub consent_revoked_at: Option<i64>,
}

/// Context for registering a new participant
#[derive(Accounts)]
pub struct RegisterParticipant<'info> {
    /// The participant account to be initialized
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 100 + 100 + 8 + 4 + 4 + 1 + 8 + 8
    )]
    pub participant: Account<'info, Participant>,
    
    /// The authority (participant) signing the transaction
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// The system program
    pub system_program: Program<'info, System>,
}

/// Context for updating a participant's profile
#[derive(Accounts)]
pub struct UpdateParticipantProfile<'info> {
    /// The participant account to be updated
    #[account(mut)]
    pub participant: Account<'info, Participant>,
    
    /// The authority (participant) signing the transaction
    pub authority: Signer<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CompletedStudy {
    pub study_id: Pubkey,
    pub completion_date: i64,
    pub reward_amount: u64,
    pub rating: Option<u8>,
    pub feedback: Option<String>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ActiveStudy {
    pub study_id: Pubkey,
    pub start_date: i64,
    pub progress: u8,
    pub last_update: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RewardRecord {
    pub study_id: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
    pub transaction_id: String,
}

#[derive(Accounts)]
pub struct TrackStudyProgress<'info> {
    #[account(mut)]
    pub participant: Account<'info, super::Participant>,
    #[account(mut)]
    pub study: Account<'info, super::Study>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SubmitStudyFeedback<'info> {
    #[account(mut)]
    pub participant: Account<'info, super::Participant>,
    #[account(mut)]
    pub study: Account<'info, super::Study>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateInterests<'info> {
    #[account(mut)]
    pub participant: Account<'info, super::Participant>,
    pub authority: Signer<'info>,
}

impl Participant {
    /// Creates a new participant with the specified parameters
    pub fn create(
        &mut self,
        authority: Pubkey,
        eligibility_proof: String,
    ) -> Result<()> {
        self.authority = authority;
        self.eligibility_proof = eligibility_proof;
        self.registered_at = Clock::get()?.unix_timestamp;
        self.active_studies = 0;
        self.completed_studies = 0;
        self.has_active_consent = false;
        self.consent_issued_at = 0;
        self.consent_revoked_at = None;
        Ok(())
    }

    /// Updates the participant's profile information
    pub fn update_profile(&mut self, profile: ParticipantProfile) -> Result<()> {
        self.profile = profile;
        Ok(())
    }

    /// Increments the number of active studies
    pub fn increment_active_studies(&mut self) -> Result<()> {
        self.active_studies = self.active_studies.checked_add(1)
            .ok_or(RecruSearchError::InvalidParticipantStatus)?;
        Ok(())
    }

    /// Decrements the number of active studies
    pub fn decrement_active_studies(&mut self) -> Result<()> {
        self.active_studies = self.active_studies.checked_sub(1)
            .ok_or(RecruSearchError::InvalidParticipantStatus)?;
        Ok(())
    }

    /// Increments the number of completed studies
    pub fn increment_completed_studies(&mut self) -> Result<()> {
        self.completed_studies = self.completed_studies.checked_add(1)
            .ok_or(RecruSearchError::InvalidParticipantStatus)?;
        Ok(())
    }

    /// Updates the participant's consent status
    pub fn update_consent_status(&mut self, has_consent: bool) -> Result<()> {
        self.has_active_consent = has_consent;
        if has_consent {
            self.consent_issued_at = Clock::get()?.unix_timestamp;
            self.consent_revoked_at = None;
        } else {
            self.consent_revoked_at = Some(Clock::get()?.unix_timestamp);
        }
        Ok(())
    }
}

impl ParticipantProfile {
    pub fn calculate_reputation_score(&self) -> u32 {
        let base_score = self.completed_studies.len() as u32 * 10;
        let rating_bonus: u32 = self.completed_studies
            .iter()
            .filter_map(|study| study.rating)
            .map(|rating| rating as u32)
            .sum();
        
        base_score + rating_bonus
    }

    pub fn is_eligible_for_study(&self, study: &super::Study) -> bool {
        // Check if participant has completed similar studies
        let similar_studies = self.completed_studies
            .iter()
            .filter(|completed| completed.study_id == study.key())
            .count();

        // Check if participant has active studies
        let active_studies = self.active_studies.len();

        // Check if participant meets minimum reputation requirement
        let meets_reputation = self.reputation_score >= study.min_reputation_score;

        similar_studies == 0 && active_studies < 3 && meets_reputation
    }
}

#[error_code]
pub enum ParticipantError {
    #[msg("Invalid progress update")]
    InvalidProgress,
    #[msg("Study not found in active studies")]
    StudyNotFound,
    #[msg("Invalid rating value")]
    InvalidRating,
    #[msg("Feedback too long")]
    FeedbackTooLong,
    #[msg("Maximum active studies reached")]
    MaxActiveStudiesReached,
    #[msg("Insufficient reputation score")]
    InsufficientReputation,
} 