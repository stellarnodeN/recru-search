use anchor_lang::prelude::*;

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

// Import the StudyStatus enum
use crate::state::admin::StudyStatus;

#[account]
pub struct Study {
    pub authority: Pubkey,
    pub status: StudyStatus,
    pub title: String,
    pub description: String,
    pub criteria_hash: String,
    pub reward_amount: u64,
    pub max_participants: u32,
    pub current_participants: u32,
    pub completed_participants: u32,
    pub is_active: bool,
    pub created_at: i64,
    pub study_type: StudyType,
    pub analytics: StudyAnalytics,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct StudyAnalytics {
    pub completion_rate: f32,
    pub total_participants: u32,
    pub average_rating: f32,
}

impl Study {
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
    pub fn update_progress(&mut self, progress: u8) -> Result<()> {
        require!(progress <= 100, crate::error::RecruSearchError::InvalidProgress);
        Ok(())
    }
    pub fn submit_feedback(&mut self, rating: u8, feedback: Option<String>) -> Result<()> {
        require!(rating <= 5, crate::error::RecruSearchError::InvalidRating);
        if let Some(feedback) = feedback {
            require!(feedback.len() <= 500, crate::error::RecruSearchError::FeedbackTooLong);
        }
        Ok(())
    }
    pub fn can_accept_participants(&self) -> Result<()> {
        require!(self.is_active, crate::error::RecruSearchError::StudyInactive);
        require!(
            self.current_participants < self.max_participants,
            crate::error::RecruSearchError::StudyFull
        );
        Ok(())
    }
    pub fn add_participant(&mut self) -> Result<()> {
        self.can_accept_participants()?;
        self.current_participants = self.current_participants.checked_add(1)
            .ok_or(crate::error::RecruSearchError::MaxParticipantsExceeded)?;
        Ok(())
    }
    pub fn remove_participant(&mut self) -> Result<()> {
        self.current_participants = self.current_participants.checked_sub(1)
            .ok_or(crate::error::RecruSearchError::InvalidParticipantStatus)?;
        Ok(())
    }
    pub fn complete_participant(&mut self) -> Result<()> {
        self.completed_participants = self.completed_participants.checked_add(1)
            .ok_or(crate::error::RecruSearchError::InvalidParticipantStatus)?;
        Ok(())
    }
    pub fn increment_consent(&mut self) -> Result<()> {
        Ok(())
    }
    pub fn decrement_consent(&mut self) -> Result<()> {
        Ok(())
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