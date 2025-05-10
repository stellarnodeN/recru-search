//! Study management module for the RecruSearch program
//! 
//! This module handles all study-related functionality including:
//! - Study creation and management
//! - Participant tracking
//! - Study completion and reward distribution
//! - Study analytics and feedback

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use crate::error::RecruSearchError;
use mpl_token_metadata::state::Metadata as MplMetadata;
use mpl_token_metadata::instruction::{create_metadata_accounts_v3, create_master_edition_v3};
use mpl_token_metadata::{
    instruction as mpl_instruction,
    state::{Creator, DataV2},
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct StudyFilter {
    pub category: Option<String>,
    pub min_reward: Option<u64>,
    pub max_participants: Option<u32>,
    pub is_active: Option<bool>,
    pub created_after: Option<i64>,
    pub study_type: Option<StudyType>,
    pub duration: Option<StudyDuration>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum StudyType {
    Survey,
    Interview,
    Clinical,
    Observational,
    Experimental,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct StudyDuration {
    pub min_days: u32,
    pub max_days: u32,
}

/// Represents a research study in the platform
#[account]
pub struct Study {
    /// The researcher's authority who created the study
    pub authority: Pubkey,
    /// Title of the study
    pub title: String,
    /// Detailed description of the study
    pub description: String,
    /// Hash of the study's eligibility criteria
    pub criteria_hash: String,
    /// Amount of tokens awarded to participants
    pub reward_amount: u64,
    /// Maximum number of participants allowed
    pub max_participants: u32,
    /// Current number of participants
    pub current_participants: u32,
    /// Number of participants who completed the study
    pub completed_participants: u32,
    /// Whether the study is currently active
    pub is_active: bool,
    /// Timestamp when the study was created
    pub created_at: i64,
    /// Type of the study
    pub study_type: StudyType,
    /// Analytics data for the study
    pub analytics: StudyAnalytics,
}

/// Analytics data for tracking study performance
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct StudyAnalytics {
    /// Percentage of participants who completed the study
    pub completion_rate: f32,
    /// Total number of participants who joined the study
    pub total_participants: u32,
    /// Average rating given by participants
    pub average_rating: f32,
}

/// Context for creating a new study
#[derive(Accounts)]
pub struct CreateStudy<'info> {
    /// The study account to be initialized
    #[account(
        init,
        payer = researcher,
        space = 8 + 32 + 100 + 500 + 100 + 8 + 4 + 4 + 4 + 1 + 8 + 1 + 16
    )]
    pub study: Account<'info, Study>,
    
    /// The researcher creating the study
    #[account(mut)]
    pub researcher: Account<'info, Researcher>,
    
    /// The authority signing the transaction
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// The system program
    pub system_program: Program<'info, System>,
}

/// Context for completing a study
#[derive(Accounts)]
pub struct CompleteStudy<'info> {
    /// The study being completed
    #[account(mut)]
    pub study: Account<'info, Study>,
    
    /// The participant completing the study
    #[account(mut)]
    pub participant: Account<'info, Participant>,
    
    /// The researcher who created the study
    #[account(mut)]
    pub researcher: Account<'info, Researcher>,
    
    /// The researcher's token account for distributing rewards
    #[account(mut)]
    pub researcher_token_account: Account<'info, TokenAccount>,
    
    /// The participant's token account for receiving rewards
    #[account(mut)]
    pub participant_token_account: Account<'info, TokenAccount>,
    
    /// The token program
    pub token_program: Program<'info, Token>,
    
    /// The system program
    pub system_program: Program<'info, System>,
}

/// Context for tracking study progress
#[derive(Accounts)]
pub struct TrackStudyProgress<'info> {
    /// The study being tracked
    #[account(mut)]
    pub study: Account<'info, Study>,
    
    /// The participant whose progress is being tracked
    #[account(mut)]
    pub participant: Account<'info, Participant>,
    
    /// The authority (participant) signing the transaction
    pub authority: Signer<'info>,
}

/// Context for submitting study feedback
#[derive(Accounts)]
pub struct SubmitStudyFeedback<'info> {
    /// The study receiving feedback
    #[account(mut)]
    pub study: Account<'info, Study>,
    
    /// The participant submitting feedback
    #[account(mut)]
    pub participant: Account<'info, Participant>,
    
    /// The authority (participant) signing the transaction
    pub authority: Signer<'info>,
}

impl Study {
    /// Creates a new study with the specified parameters
    pub fn create(
        &mut self,
        authority: Pubkey,
        title: String,
        description: String,
        criteria_hash: String,
        reward_amount: u64,
        max_participants: u32,
        study_type: StudyType,
    ) -> Result<()> {
        self.authority = authority;
        self.title = title;
        self.description = description;
        self.criteria_hash = criteria_hash;
        self.reward_amount = reward_amount;
        self.max_participants = max_participants;
        self.current_participants = 0;
        self.is_active = true;
        self.created_at = Clock::get()?.unix_timestamp;
        self.completed_participants = 0;
        self.analytics = StudyAnalytics {
            completion_rate: 0.0,
            total_participants: 0,
            average_rating: 0.0,
        };
        self.study_type = study_type;
        Ok(())
    }

    /// Updates study progress for a participant
    pub fn update_progress(&mut self, progress: u8) -> Result<()> {
        require!(progress <= 100, RecruSearchError::InvalidProgress);
        // Update study progress logic here
        Ok(())
    }

    /// Submits feedback for a study
    pub fn submit_feedback(&mut self, rating: u8, feedback: Option<String>) -> Result<()> {
        require!(rating <= 5, RecruSearchError::InvalidRating);
        if let Some(feedback) = feedback {
            require!(feedback.len() <= 500, RecruSearchError::FeedbackTooLong);
        }
        // Update study feedback logic here
        Ok(())
    }

    /// Checks if the study can accept more participants
    pub fn can_accept_participants(&self) -> Result<()> {
        require!(self.is_active, RecruSearchError::StudyInactive);
        require!(
            self.current_participants < self.max_participants,
            RecruSearchError::StudyFull
        );
        Ok(())
    }

    /// Adds a participant to the study
    pub fn add_participant(&mut self) -> Result<()> {
        self.can_accept_participants()?;
        self.current_participants = self.current_participants.checked_add(1)
            .ok_or(RecruSearchError::MaxParticipantsExceeded)?;
        Ok(())
    }

    /// Removes a participant from the study
    pub fn remove_participant(&mut self) -> Result<()> {
        self.current_participants = self.current_participants.checked_sub(1)
            .ok_or(RecruSearchError::InvalidParticipantStatus)?;
        Ok(())
    }

    /// Marks a participant as completed
    pub fn complete_participant(&mut self) -> Result<()> {
        self.completed_participants = self.completed_participants.checked_add(1)
            .ok_or(RecruSearchError::InvalidParticipantStatus)?;
        Ok(())
    }

    /// Increments consent count
    pub fn increment_consent(&mut self) -> Result<()> {
        // Implementation of increment_consent method
        Ok(())
    }

    /// Decrements consent count
    pub fn decrement_consent(&mut self) -> Result<()> {
        // Implementation of decrement_consent method
        Ok(())
    }
}

#[derive(Accounts)]
pub struct UpdateStudyAnalytics<'info> {
    #[account(mut)]
    pub study: Account<'info, Study>,
    #[account(mut)]
    pub researcher: Account<'info, super::Researcher>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ReportStudy<'info> {
    #[account(mut)]
    pub study: Account<'info, Study>,
    #[account(mut)]
    pub participant: Account<'info, super::Participant>,
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl StudyFilter {
    pub fn matches(&self, study: &Study) -> bool {
        if let Some(category) = &self.category {
            if !study.description.contains(category) {
                return false;
            }
        }

        if let Some(min_reward) = self.min_reward {
            if study.reward_amount < min_reward {
                return false;
            }
        }

        if let Some(max_participants) = self.max_participants {
            if study.max_participants > max_participants {
                return false;
            }
        }

        if let Some(is_active) = self.is_active {
            if study.is_active != is_active {
                return false;
            }
        }

        if let Some(created_after) = self.created_after {
            if study.created_at < created_after {
                return false;
            }
        }

        if let Some(study_type) = &self.study_type {
            if study.study_type != *study_type {
                return false;
            }
        }

        if let Some(duration) = &self.duration {
            let study_duration = Clock::get().unwrap().unix_timestamp - study.created_at;
            let min_duration = duration.min_days as i64 * 24 * 60 * 60;
            let max_duration = duration.max_days as i64 * 24 * 60 * 60;
            if study_duration < min_duration || study_duration > max_duration {
                return false;
            }
        }

        true
    }
}

#[error_code]
pub enum StudyError {
    #[msg("Invalid study category")]
    InvalidCategory,
    #[msg("Invalid filter criteria")]
    InvalidFilterCriteria,
    #[msg("Study report already exists")]
    ReportExists,
    #[msg("Invalid study duration")]
    InvalidDuration,
    #[msg("Study is not active")]
    StudyInactive,
    #[msg("Participant not enrolled")]
    ParticipantNotEnrolled,
    #[msg("Invalid progress update")]
    InvalidProgressUpdate,
}

pub fn create_study(
    ctx: Context<CreateStudy>,
    title: String,
    description: String,
    criteria_hash: String,
    reward_amount: u64,
    max_participants: u32,
    study_type: StudyType,
) -> Result<()> {
    let study = &mut ctx.accounts.study;
    study.authority = ctx.accounts.researcher.key();
    study.title = title;
    study.description = description;
    study.criteria_hash = criteria_hash;
    study.reward_amount = reward_amount;
    study.max_participants = max_participants;
    study.current_participants = 0;
    study.completed_participants = 0;
    study.is_active = true;
    study.created_at = Clock::get()?.unix_timestamp;
    study.study_type = study_type;
    study.analytics = StudyAnalytics {
        completion_rate: 0.0,
        total_participants: 0,
        average_rating: 0.0,
    };
    Ok(())
}

create_metadata_accounts_v3(
    cpi_context,
    CONSENT_NFT_NAME.to_string(),
    CONSENT_NFT_SYMBOL.to_string(),
    CONSENT_NFT_URI.to_string(),
    None,
    0,
    true,
    true,
    None,
    None,
    None,
)?;

create_master_edition_v3(
    master_edition_cpi_context,
    Some(0), // Max supply of 0 means unlimited
)?;

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